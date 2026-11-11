//! Internal HTTP transport shared by all resources: headers, idempotency
//! keys, the retry policy and error parsing.

use std::time::Duration;

use reqwest::header::{self, HeaderMap};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::Inner;
use crate::error::{ApiError, Error, Result};
use crate::options::RequestOptions;

/// 5xx codes the spec's error table classifies as terminal: never retried.
const TERMINAL_5XX_CODES: &[&str] = &["PAYINS_CARD_PAYMENT_PENDING"];

/// The longest `Retry-After` the SDK waits for; longer values fall back to
/// exponential backoff (an in-flight 409 usually says 61–300 s).
const MAX_RETRY_AFTER: Duration = Duration::from_secs(60);

/// Longest idempotency key the API accepts.
const MAX_IDEMPOTENCY_KEY_LEN: usize = 255;

/// Headers the SDK owns; extra headers with these names are ignored.
const RESERVED_HEADERS: &[&str] = &[
    "authorization",
    "idempotency-key",
    "content-type",
    "user-agent",
];

/// A received response: status, headers and the full body.
struct Raw {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

/// The result of one attempt.
enum Attempt {
    /// A status line arrived and the body was read in full.
    Received(Raw),
    /// A status line arrived but reading the body failed (e.g. the per-attempt
    /// timeout fired mid-body). The status still decides what happens next.
    BodyFailed(Raw, reqwest::Error),
    /// No status was received.
    NoResponse(reqwest::Error),
}

impl Inner {
    /// Performs a `GET` request with no query parameters.
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request::<T, (), ()>(Method::GET, path, None, None)
            .await
    }

    /// Performs a `GET` request whose query string is the serialized `query`.
    ///
    /// Pass a `#[derive(Serialize)]` params struct (use
    /// `skip_serializing_if = "Option::is_none"` for optional params).
    pub(crate) async fn get_query<T, Q>(&self, path: &str, query: &Q) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        self.request::<T, Q, ()>(Method::GET, path, Some(query), None)
            .await
    }

    /// [`Inner::request_with`] with default options.
    pub(crate) async fn request<T, Q, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        self.request_with(method, path, query, body, &RequestOptions::default())
            .await
    }

    /// Performs a JSON request (with retries) and deserializes the response.
    ///
    /// `path` is appended to the configured base URL (e.g. `/available/rails`).
    /// The body is serialized once and the same bytes are resent on every
    /// attempt, because the API hashes the raw body for idempotency.
    pub(crate) async fn request_with<T, Q, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
        options: &RequestOptions,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let body = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|e| Error::Config(format!("failed to serialize request body: {e}")))?;
        let mutating = method != Method::GET && method != Method::HEAD;
        let key = self.resolve_idempotency_key(mutating, options)?;
        let timeout = options.timeout.unwrap_or(self.timeout);
        let url = format!("{}{}", self.base_url, path);

        let mut attempt: u32 = 0;
        loop {
            let mut req = self.base_request(method.clone(), &url, options, timeout);
            if let Some(key) = &key {
                req = req.header("Idempotency-Key", key);
            }
            if let Some(query) = query {
                req = req.query(query);
            }
            if let Some(body) = &body {
                req = req
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(body.clone());
            }

            let can_retry = attempt < self.max_retries && (!mutating || key.is_some());
            let raw = match receive(req).await {
                // Only a request that provably got no answer (it never
                // connected, or timed out before a status line) is resent.
                Attempt::NoResponse(e) if can_retry && (e.is_connect() || e.is_timeout()) => {
                    self.sleep(backoff(attempt)).await;
                    attempt += 1;
                    continue;
                }
                Attempt::NoResponse(source) => {
                    return Err(Error::Http {
                        source,
                        idempotency_key: key,
                    });
                }
                // The request ran; the body was lost. Never resend it.
                Attempt::BodyFailed(raw, source) if raw.status.is_success() => {
                    return Err(Error::Http {
                        source,
                        idempotency_key: key,
                    });
                }
                Attempt::BodyFailed(raw, _) | Attempt::Received(raw) => raw,
            };
            if raw.status.is_success() {
                return decode(raw);
            }
            let mut err = parse_api_error(&raw, key.clone());
            let (retryable, auto_retry) = classify(mutating, &err);
            err.retryable = retryable;
            if !(auto_retry && can_retry) {
                return Err(err.into());
            }
            let delay = match err.retry_after {
                Some(d) if d <= MAX_RETRY_AFTER => d,
                _ => backoff(attempt),
            };
            self.sleep(delay).await;
            attempt += 1;
        }
    }

    /// Performs a multipart `POST` and deserializes the JSON response.
    ///
    /// Multipart requests get no idempotency protection from the API and a
    /// `Form` can't be replayed, so they are sent exactly once. reqwest sets the
    /// `Content-Type`/boundary.
    pub(crate) async fn post_multipart_with<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        form: reqwest::multipart::Form,
        options: &RequestOptions,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let timeout = options.timeout.unwrap_or(self.timeout);
        let req = self
            .base_request(Method::POST, &url, options, timeout)
            .query(query)
            .multipart(form);
        let raw = match receive(req).await {
            Attempt::NoResponse(source) => {
                return Err(Error::Http {
                    source,
                    idempotency_key: None,
                });
            }
            Attempt::BodyFailed(raw, source) if raw.status.is_success() => {
                return Err(Error::Http {
                    source,
                    idempotency_key: None,
                });
            }
            Attempt::BodyFailed(raw, _) | Attempt::Received(raw) => raw,
        };
        if raw.status.is_success() {
            return decode(raw);
        }
        let mut err = parse_api_error(&raw, None);
        err.retryable = classify(true, &err).0;
        Err(err.into())
    }

    /// A request with the headers every call carries.
    fn base_request(
        &self,
        method: Method,
        url: &str,
        options: &RequestOptions,
        timeout: Duration,
    ) -> reqwest::RequestBuilder {
        let mut req = self.http.request(method, url).timeout(timeout);
        for (name, value) in &options.headers {
            if !RESERVED_HEADERS.contains(&name.as_str()) {
                req = req.header(name, value);
            }
        }
        req.bearer_auth(&self.api_key)
            .header(header::USER_AGENT, &self.user_agent)
            .header(header::ACCEPT, "application/json")
    }

    /// The caller's key (validated), or a fresh UUIDv4 for a mutation when
    /// retries are on. Generated before the first attempt: that attempt may
    /// already execute, so a key minted at retry time would protect nothing.
    fn resolve_idempotency_key(
        &self,
        mutating: bool,
        options: &RequestOptions,
    ) -> Result<Option<String>> {
        if !mutating {
            return Ok(None);
        }
        match &options.idempotency_key {
            Some(key) if key.is_empty() => Err(Error::Config(
                "idempotency key must not be empty (the API would silently ignore it)".into(),
            )),
            Some(key) if key.len() > MAX_IDEMPOTENCY_KEY_LEN => Err(Error::Config(format!(
                "idempotency key must be at most {MAX_IDEMPOTENCY_KEY_LEN} characters"
            ))),
            // Visible ASCII only: always a legal header value, and never
            // altered by proxies (no whitespace, control or non-ASCII bytes).
            Some(key) if !key.bytes().all(|b| b.is_ascii_graphic()) => Err(Error::Config(
                "idempotency key must be visible ASCII (no spaces or control characters)".into(),
            )),
            Some(key) => Ok(Some(key.clone())),
            None if self.max_retries > 0 => Ok(Some(uuid::Uuid::new_v4().to_string())),
            None => Ok(None),
        }
    }

    async fn sleep(&self, delay: Duration) {
        (self.sleep)(delay).await;
    }
}

/// Sends `req` and reads the whole body.
async fn receive(req: reqwest::RequestBuilder) -> Attempt {
    let response = match req.send().await {
        Ok(response) => response,
        Err(e) => return Attempt::NoResponse(e),
    };
    let status = response.status();
    let headers = response.headers().clone();
    match response.bytes().await {
        Ok(body) => Attempt::Received(Raw {
            status,
            headers,
            body: body.to_vec(),
        }),
        Err(e) => Attempt::BodyFailed(
            Raw {
                status,
                headers,
                body: Vec::new(),
            },
            e,
        ),
    }
}

/// Decodes a 2xx body. An empty body (e.g. 204) decodes as `null`, then `[]`,
/// so `()`/`Option` and list return types all succeed.
fn decode<T: DeserializeOwned>(raw: Raw) -> Result<T> {
    if raw.status == StatusCode::NO_CONTENT || raw.body.is_empty() {
        return serde_json::from_slice(b"null")
            .or_else(|_| serde_json::from_slice(b"[]"))
            .map_err(|source| Error::Decode {
                source,
                body: String::new(),
            });
    }
    serde_json::from_slice(&raw.body).map_err(|source| Error::Decode {
        source,
        body: String::from_utf8_lossy(&raw.body).into_owned(),
    })
}

/// The retry decision for an HTTP error: `(retryable flag, auto-retry?)`.
///
/// - 429 and an in-flight idempotency 409: retried (the handler never ran or
///   is still running under the same key).
/// - A 5xx with a terminal error code: neither.
/// - A mutation with any other 5xx: flagged retryable but **never** resent
///   automatically, because the API releases the idempotency key on 5xx and a
///   resend would run the handler again.
/// - A `GET` with 500/502/503/504 or 408: retried. Other 5xx (501, 505, …)
///   and other 4xx: neither.
fn classify(mutating: bool, err: &ApiError) -> (bool, bool) {
    let terminal_code = err
        .code
        .as_deref()
        .is_some_and(|c| TERMINAL_5XX_CODES.contains(&c));
    match err.status {
        409 if err.message == "idempotency_key_in_flight" => (true, true),
        429 => (true, true),
        500..=599 if terminal_code => (false, false),
        500..=599 if mutating => (true, false),
        408 if mutating => (true, false),
        408 | 500 | 502 | 503 | 504 => (true, true),
        _ => (false, false),
    }
}

/// `min(8s, 0.5s × 2^attempt) × U[0.5, 1.0)`.
fn backoff(attempt: u32) -> Duration {
    let base = (0.5 * 2f64.powi(attempt.min(16) as i32)).min(8.0);
    // Jitter from a v4 UUID's random bits: no extra RNG dependency.
    let random = (uuid::Uuid::new_v4().as_u128() >> 75) as f64 / (1u64 << 53) as f64;
    Duration::from_secs_f64(base * (0.5 + 0.5 * random))
}

/// Builds an [`ApiError`] from a non-2xx response. Every body field is
/// optional; a non-JSON body (e.g. a proxy's HTML 502) yields
/// `HTTP <status> <reason>`.
fn parse_api_error(raw: &Raw, idempotency_key: Option<String>) -> ApiError {
    #[derive(serde::Deserialize, Default)]
    struct Body {
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        errors: Option<serde_json::Value>,
        #[serde(default)]
        trace_id: Option<String>,
    }

    let body: Body = serde_json::from_slice(&raw.body).unwrap_or_default();
    let message = body.message.unwrap_or_else(|| {
        format!(
            "HTTP {} {}",
            raw.status.as_u16(),
            raw.status.canonical_reason().unwrap_or("error")
        )
    });
    let errors = match body.errors {
        Some(serde_json::Value::Array(items)) => items,
        Some(serde_json::Value::Null) | None => Vec::new(),
        Some(other) => vec![other],
    };
    let header_str = |name: &str| {
        raw.headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let retry_after = header_str("retry-after")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs);

    ApiError {
        status: raw.status.as_u16(),
        code: body.code,
        message,
        description: body.description,
        errors,
        trace_id: body.trace_id,
        request_id: header_str("x-request-id"),
        idempotency_key,
        retryable: false,
        retry_after,
        raw_body: String::from_utf8_lossy(&raw.body).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_error(status: u16, code: Option<&str>, message: &str) -> ApiError {
        ApiError {
            status,
            code: code.map(str::to_string),
            message: message.to_string(),
            description: None,
            errors: Vec::new(),
            trace_id: None,
            request_id: None,
            idempotency_key: None,
            retryable: false,
            retry_after: None,
            raw_body: String::new(),
        }
    }

    #[test]
    fn classify_follows_the_a2_table() {
        let in_flight = api_error(409, None, "idempotency_key_in_flight");
        assert_eq!(classify(true, &in_flight), (true, true));
        assert_eq!(
            classify(true, &api_error(409, None, "conflict")),
            (false, false)
        );
        assert_eq!(classify(true, &api_error(429, None, "x")), (true, true));
        assert_eq!(classify(false, &api_error(429, None, "x")), (true, true));
        // Mutation 5xx (any): flagged retryable, never auto-retried.
        for status in [500, 501, 502, 503, 504, 505, 507] {
            assert_eq!(classify(true, &api_error(status, None, "x")), (true, false));
        }
        for status in [500, 502, 503, 504] {
            assert_eq!(classify(false, &api_error(status, None, "x")), (true, true));
        }
        for status in [501, 505, 507] {
            assert_eq!(
                classify(false, &api_error(status, None, "x")),
                (false, false)
            );
        }
        let terminal = api_error(502, Some("PAYINS_CARD_PAYMENT_PENDING"), "x");
        assert_eq!(classify(false, &terminal), (false, false));
        assert_eq!(classify(false, &api_error(408, None, "x")), (true, true));
        assert_eq!(classify(true, &api_error(408, None, "x")), (true, false));
        for status in [400, 401, 404, 422] {
            assert_eq!(
                classify(false, &api_error(status, None, "x")),
                (false, false)
            );
        }
    }

    #[test]
    fn backoff_is_capped_and_jittered() {
        for attempt in 0..20 {
            let d = backoff(attempt).as_secs_f64();
            let base = (0.5 * 2f64.powi(attempt as i32)).min(8.0);
            assert!(d >= base * 0.5 && d < base, "attempt {attempt}: {d}");
        }
    }
}
