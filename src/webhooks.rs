//! Webhook signature verification.
//!
//! BlindPay signs webhook deliveries with [Svix](https://www.svix.com), the
//! same scheme used by the other BlindPay SDKs. Each delivery carries three
//! headers — an id, a timestamp, and a signature — which together with the raw
//! request body and your endpoint's signing secret (`whsec_…`) prove the
//! payload's authenticity and freshness.
//!
//! Verify a delivery before trusting it:
//!
//! ```no_run
//! # fn handler(secret: &str, id: &str, ts: &str, body: &[u8], sig: &str)
//! #     -> Result<(), blindpay::WebhookVerificationError> {
//! blindpay::verify_webhook_signature(secret, id, ts, body, sig)?;
//! // signature is valid — safe to parse `body`
//! # Ok(())
//! # }
//! ```
//!
//! The header names accepted by the underlying Svix verifier are the
//! `Svix-Id` / `Svix-Timestamp` / `Svix-Signature` triplet and their
//! Standard-Webhooks aliases `Webhook-Id` / `Webhook-Timestamp` /
//! `Webhook-Signature`. Pass the values, not the header names — this function
//! builds the header map for you.

use std::str::FromStr;

use svix::webhooks::Webhook;

/// An error returned by [`verify_webhook_signature`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WebhookVerificationError {
    /// The signing secret could not be parsed (not a valid `whsec_…` value).
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
}

/// Verifies a Svix webhook signature against the raw request body.
///
/// `secret` is the endpoint's signing secret (`whsec_…`); `id`, `timestamp`,
/// and `signature` are the values of the `Svix-Id` / `Svix-Timestamp` /
/// `Svix-Signature` headers (the `Webhook-*` aliases work too); `payload` is
/// the **raw** request body bytes, exactly as received — re-serializing parsed
/// JSON will change the bytes and break verification.
///
/// Returns `Ok(())` when the signature is valid and the timestamp is within
/// the Svix tolerance window (±5 minutes).
///
/// # Errors
///
/// - [`WebhookVerificationError::InvalidSecret`] if `secret` is malformed.
/// - [`WebhookVerificationError::InvalidHeaderValue`] if a header value cannot
///   be represented as an HTTP header.
/// - [`WebhookVerificationError::Verification`] if the signature is invalid,
///   the timestamp is stale, or a header is missing.
pub fn verify_webhook_signature(
    secret: &str,
    id: &str,
    timestamp: &str,
    payload: &[u8],
    signature: &str,
) -> Result<(), WebhookVerificationError> {
    let webhook = Webhook::new(secret).map_err(|_| WebhookVerificationError::InvalidSecret)?;

    let mut headers = http::HeaderMap::new();
    insert_header(&mut headers, "svix-id", id)?;
    insert_header(&mut headers, "svix-timestamp", timestamp)?;
    insert_header(&mut headers, "svix-signature", signature)?;

    webhook
        .verify(payload, &headers)
        .map_err(|err| WebhookVerificationError::Verification(err.to_string()))
}

fn insert_header(
    headers: &mut http::HeaderMap,
    name: &'static str,
    value: &str,
) -> Result<(), WebhookVerificationError> {
    let header_name = http::HeaderName::from_str(name)
        .map_err(|_| WebhookVerificationError::InvalidHeaderValue(name))?;
    let header_value = http::HeaderValue::from_str(value)
        .map_err(|_| WebhookVerificationError::InvalidHeaderValue(name))?;
    headers.insert(header_name, header_value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "whsec_C2FVsBQIhrscChlQIMV+b5sSYspob7oD";
    const MSG_ID: &str = "msg_27UH4WbU6Z5A5EzD8u03UvzRbpk";
    const PAYLOAD: &[u8] = br#"{"email":"test@example.com","username":"test_user"}"#;

    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap()
    }

    fn sign(timestamp: i64) -> String {
        Webhook::new(SECRET)
            .unwrap()
            .sign(MSG_ID, timestamp, PAYLOAD)
            .unwrap()
    }

    #[test]
    fn accepts_a_valid_signature() {
        let ts = now();
        let signature = sign(ts);

        verify_webhook_signature(SECRET, MSG_ID, &ts.to_string(), PAYLOAD, &signature).unwrap();
    }

    #[test]
    fn accepts_the_webhook_alias_secret_without_prefix() {
        // `Webhook::new` strips the `whsec_` prefix, so a bare secret verifies too.
        let ts = now();
        let signature = sign(ts);
        let bare = SECRET.strip_prefix("whsec_").unwrap();

        verify_webhook_signature(bare, MSG_ID, &ts.to_string(), PAYLOAD, &signature).unwrap();
    }

    #[test]
    fn rejects_a_tampered_signature() {
        let ts = now();
        // Signature for a different payload — must not validate against PAYLOAD.
        let signature = Webhook::new(SECRET)
            .unwrap()
            .sign(MSG_ID, ts, br#"{"email":"attacker@example.com"}"#)
            .unwrap();

        let err = verify_webhook_signature(SECRET, MSG_ID, &ts.to_string(), PAYLOAD, &signature)
            .unwrap_err();
        assert!(matches!(err, WebhookVerificationError::Verification(_)));
    }

    #[test]
    fn rejects_a_payload_mutated_after_signing() {
        let ts = now();
        let signature = sign(ts);

        let err = verify_webhook_signature(
            SECRET,
            MSG_ID,
            &ts.to_string(),
            br#"{"email":"test@example.com","username":"changed"}"#,
            &signature,
        )
        .unwrap_err();
        assert!(matches!(err, WebhookVerificationError::Verification(_)));
    }

    #[test]
    fn rejects_a_stale_timestamp() {
        // Outside the ±5 minute tolerance window.
        let ts = now() - 60 * 60;
        let signature = sign(ts);

        let err = verify_webhook_signature(SECRET, MSG_ID, &ts.to_string(), PAYLOAD, &signature)
            .unwrap_err();
        assert!(matches!(err, WebhookVerificationError::Verification(_)));
    }

    #[test]
    fn rejects_a_malformed_secret() {
        let ts = now();
        let signature = sign(ts);

        // Not valid base64 after the prefix.
        let err = verify_webhook_signature(
            "whsec_not!!base64",
            MSG_ID,
            &ts.to_string(),
            PAYLOAD,
            &signature,
        )
        .unwrap_err();
        assert!(matches!(err, WebhookVerificationError::InvalidSecret));
    }

    #[test]
    fn rejects_a_header_value_with_control_characters() {
        let ts = now();
        let signature = sign(ts);

        let err = verify_webhook_signature(SECRET, "bad\nid", &ts.to_string(), PAYLOAD, &signature)
            .unwrap_err();
        assert!(matches!(
            err,
            WebhookVerificationError::InvalidHeaderValue("svix-id")
        ));
    }
}
