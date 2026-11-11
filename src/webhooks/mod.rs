//! Webhook signature verification and typed webhook events.
//!
//! BlindPay signs webhook deliveries with the [Svix](https://www.svix.com)
//! scheme. Each delivery carries `svix-id`, `svix-timestamp` and
//! `svix-signature` headers which, with the **raw** request body and your
//! endpoint's signing secret (`whsec_…`), prove the payload's authenticity and
//! freshness. Verification is native (HMAC-SHA256); it needs no API key or
//! client.
//!
//! ```no_run
//! # fn handler(headers: &http::HeaderMap, raw_body: &[u8], secret: &str)
//! #     -> Result<(), blindpay::WebhookVerificationError> {
//! use blindpay::webhooks::{construct_webhook_event, WebhookEventPayload};
//!
//! match construct_webhook_event(raw_body, headers, secret)? {
//!     WebhookEventPayload::PayoutComplete(payout) => {
//!         // Failures arrive on payout.complete *and* payout.update: branch on status.
//!         println!("payout {} is {}", payout.id, payout.status);
//!     }
//!     WebhookEventPayload::Unknown(raw) => println!("new event type: {raw}"),
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Verify against the bytes you received, not re-serialized JSON, dedupe
//! deliveries on `svix-id`, and never log the `whsec_` secret.

mod events;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub use events::{CustomerDeleteEvent, TosAcceptEvent, WalletInboundEvent, WebhookEventPayload};

/// How far a delivery's timestamp may be from now (5 minutes).
pub const DEFAULT_TOLERANCE: Duration = Duration::from_secs(300);

/// An error returned by [`verify_webhook_signature`] and
/// [`construct_webhook_event`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WebhookVerificationError {
    /// The signing secret could not be parsed (not base64 after `whsec_`).
    #[error("invalid webhook signing secret")]
    InvalidSecret,

    /// One of the supplied header values is not a valid HTTP header value
    /// (e.g. contains control characters).
    #[error("invalid value for webhook header `{0}`")]
    InvalidHeaderValue(&'static str),

    /// The signature did not match, the timestamp was outside the tolerance
    /// window, or a required header was missing.
    #[error("webhook signature verification failed: {0}")]
    Verification(String),

    /// The signature was valid but the payload isn't a JSON webhook event.
    #[error("webhook payload could not be decoded: {0}")]
    Decode(#[source] serde_json::Error),
}

/// Returns `true` when `payload` carries a valid Svix signature for `secret`
/// and its timestamp is within [`DEFAULT_TOLERANCE`] of the system clock.
///
/// Header names are matched case-insensitively; the `webhook-*` aliases are
/// accepted too. Never panics or errors: anything malformed is `false`.
pub fn verify(payload: &[u8], headers: &http::HeaderMap, secret: &str) -> bool {
    verify_at(payload, headers, secret, DEFAULT_TOLERANCE, unix_now())
}

/// Like [`verify`], with an explicit tolerance and clock (`now` in Unix
/// seconds). Useful for tests.
pub fn verify_at(
    payload: &[u8],
    headers: &http::HeaderMap,
    secret: &str,
    tolerance: Duration,
    now: i64,
) -> bool {
    check(payload, headers, secret, tolerance, now).is_ok()
}

/// Verifies `payload` (see [`verify`]) and decodes it into a typed event.
/// Unknown event names decode to [`WebhookEventPayload::Unknown`]; a known
/// event whose payload doesn't match its type is a `Decode` error.
///
/// # Errors
///
/// - [`WebhookVerificationError::Verification`] or
///   [`WebhookVerificationError::InvalidSecret`]: the delivery isn't
///   authentic. Reject it (e.g. 400).
/// - [`WebhookVerificationError::Decode`]: the signature **was valid** but the
///   body didn't match the event's type. Log it and still acknowledge the
///   delivery (2xx); rejecting it makes Svix redeliver the same body
///   indefinitely.
pub fn construct_webhook_event(
    payload: &[u8],
    headers: &http::HeaderMap,
    secret: &str,
) -> Result<WebhookEventPayload, WebhookVerificationError> {
    construct_webhook_event_at(payload, headers, secret, DEFAULT_TOLERANCE, unix_now())
}

/// Like [`construct_webhook_event`], with an explicit tolerance and clock.
///
/// # Errors
///
/// See [`construct_webhook_event`].
pub fn construct_webhook_event_at(
    payload: &[u8],
    headers: &http::HeaderMap,
    secret: &str,
    tolerance: Duration,
    now: i64,
) -> Result<WebhookEventPayload, WebhookVerificationError> {
    check(payload, headers, secret, tolerance, now)?;
    serde_json::from_slice(payload).map_err(WebhookVerificationError::Decode)
}

/// Verifies a Svix webhook signature from the three header values.
///
/// `secret` is the endpoint's signing secret (`whsec_…`, prefix optional);
/// `id`, `timestamp` and `signature` are the `svix-id` / `svix-timestamp` /
/// `svix-signature` header values; `payload` is the **raw** body.
///
/// # Errors
///
/// - [`WebhookVerificationError::InvalidSecret`] if `secret` is malformed.
/// - [`WebhookVerificationError::InvalidHeaderValue`] if a value can't be an
///   HTTP header value.
/// - [`WebhookVerificationError::Verification`] if the signature is invalid or
///   the timestamp is outside [`DEFAULT_TOLERANCE`].
pub fn verify_webhook_signature(
    secret: &str,
    id: &str,
    timestamp: &str,
    payload: &[u8],
    signature: &str,
) -> Result<(), WebhookVerificationError> {
    let mut headers = http::HeaderMap::new();
    for (name, value) in [
        ("svix-id", id),
        ("svix-timestamp", timestamp),
        ("svix-signature", signature),
    ] {
        let value = http::HeaderValue::from_str(value)
            .map_err(|_| WebhookVerificationError::InvalidHeaderValue(name))?;
        headers.insert(name, value);
    }
    check(payload, &headers, secret, DEFAULT_TOLERANCE, unix_now())
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn header<'a>(headers: &'a http::HeaderMap, names: [&str; 2]) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| headers.get(*name).and_then(|v| v.to_str().ok()))
}

/// The Svix verification algorithm.
fn check(
    payload: &[u8],
    headers: &http::HeaderMap,
    secret: &str,
    tolerance: Duration,
    now: i64,
) -> Result<(), WebhookVerificationError> {
    let fail = |why: &str| Err(WebhookVerificationError::Verification(why.to_string()));
    let (Some(id), Some(timestamp), Some(signature)) = (
        header(headers, ["svix-id", "webhook-id"]),
        header(headers, ["svix-timestamp", "webhook-timestamp"]),
        header(headers, ["svix-signature", "webhook-signature"]),
    ) else {
        return fail("missing svix-id, svix-timestamp or svix-signature header");
    };
    if timestamp.is_empty() || !timestamp.bytes().all(|b| b.is_ascii_digit()) {
        return fail("non-numeric timestamp");
    }
    let Ok(ts) = timestamp.parse::<i64>() else {
        return fail("timestamp out of range");
    };
    if now.abs_diff(ts) > tolerance.as_secs() {
        return fail("timestamp outside the tolerance window");
    }
    let key = STANDARD
        .decode(secret.strip_prefix("whsec_").unwrap_or(secret))
        .map_err(|_| WebhookVerificationError::InvalidSecret)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&key)
        .map_err(|_| WebhookVerificationError::InvalidSecret)?;
    mac.update(id.as_bytes());
    mac.update(b".");
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(payload);

    let matched = signature.split(' ').any(|part| {
        let Some(("v1", value)) = part.split_once(',') else {
            return false;
        };
        // `verify_slice` compares in constant time and checks the length.
        STANDARD
            .decode(value)
            .is_ok_and(|sig| mac.clone().verify_slice(&sig).is_ok())
    });
    if matched {
        Ok(())
    } else {
        fail("no matching v1 signature")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "whsec_MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw";

    fn sign(secret: &str, id: &str, ts: i64, payload: &[u8]) -> String {
        let key = STANDARD
            .decode(secret.trim_start_matches("whsec_"))
            .unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).unwrap();
        mac.update(format!("{id}.{ts}.").as_bytes());
        mac.update(payload);
        format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()))
    }

    #[test]
    fn signature_wrapper_accepts_valid_and_rejects_bad_input() {
        let ts = unix_now();
        let sig = sign(SECRET, "msg_1", ts, b"{}");
        verify_webhook_signature(SECRET, "msg_1", &ts.to_string(), b"{}", &sig).unwrap();
        let bare = SECRET.strip_prefix("whsec_").unwrap();
        verify_webhook_signature(bare, "msg_1", &ts.to_string(), b"{}", &sig).unwrap();

        let stale = (ts - 3600).to_string();
        let err = verify_webhook_signature(SECRET, "msg_1", &stale, b"{}", &sig).unwrap_err();
        assert!(matches!(err, WebhookVerificationError::Verification(_)));

        let err =
            verify_webhook_signature("whsec_not!!base64", "msg_1", &ts.to_string(), b"{}", &sig)
                .unwrap_err();
        assert!(matches!(err, WebhookVerificationError::InvalidSecret));

        let err =
            verify_webhook_signature(SECRET, "bad\nid", &ts.to_string(), b"{}", &sig).unwrap_err();
        assert!(matches!(
            err,
            WebhookVerificationError::InvalidHeaderValue("svix-id")
        ));
    }

    #[test]
    fn accepts_webhook_header_aliases_case_insensitively() {
        let ts = 1_762_790_400;
        let mut headers = http::HeaderMap::new();
        headers.insert("Webhook-Id", "msg_1".parse().unwrap());
        headers.insert("WEBHOOK-TIMESTAMP", ts.to_string().parse().unwrap());
        headers.insert(
            "webhook-signature",
            sign(SECRET, "msg_1", ts, b"x").parse().unwrap(),
        );
        assert!(verify_at(b"x", &headers, SECRET, DEFAULT_TOLERANCE, ts));
    }
}
