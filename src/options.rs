//! Per-request options.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Options for a single request, passed to a method's `*_with_options`
/// variant (every mutating method has one, e.g.
/// [`Payouts::create_evm_with_options`](crate::Payouts::create_evm_with_options)).
///
/// Options apply to that call only; they never change the shared client.
///
/// ```
/// use std::time::Duration;
/// use blindpay::RequestOptions;
///
/// let options = RequestOptions::new()
///     .idempotency_key("payout-2024-0001")
///     .timeout(Duration::from_secs(30));
/// ```
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RequestOptions {
    /// The `Idempotency-Key` header to send (1–255 characters). When unset and
    /// retries are enabled, the SDK generates a UUIDv4 key before the first
    /// attempt of every non-multipart mutation, so retries can't execute it
    /// twice. Unrelated to the ToS `idempotency_key` body field.
    pub idempotency_key: Option<String>,
    /// Per-attempt timeout, overriding the client's
    /// [`timeout`](crate::BlindPayBuilder::timeout).
    pub timeout: Option<Duration>,
    /// Extra headers to send. `Authorization`, `Idempotency-Key`,
    /// `Content-Type` and `User-Agent` are set by the SDK and ignored here.
    pub headers: HeaderMap,
}

impl RequestOptions {
    /// Creates empty options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `Idempotency-Key` to send. An empty key, or one longer than
    /// 255 characters, is rejected with [`Error::Config`](crate::Error::Config)
    /// before anything is sent (the API silently ignores an empty key).
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Sets the per-attempt timeout for this request.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Adds an extra header.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.append(name, value);
        self
    }
}
