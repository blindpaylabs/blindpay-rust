//! Error and result types for the BlindPay SDK.

use std::fmt;
use std::time::Duration;

/// A specialized [`Result`](std::result::Result) for fallible BlindPay operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the BlindPay SDK.
///
/// Match on [`Error::Api`] for responses the API rejected and on
/// [`Error::Http`] for transport failures where no response arrived. Use
/// [`Error::is_retryable`] and [`Error::idempotency_key`] to resend a mutation
/// safely: resend with the **same** key after checking the resource's state.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The client was built with invalid configuration, or a method was called
    /// with an invalid argument (for example an empty API key, an empty SWIFT
    /// code, or an empty or over-255-character idempotency key). Nothing was
    /// sent.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// The BlindPay API returned a non-success (non-2xx) HTTP status. Boxed
    /// to keep `Result` small; fields are reachable through `Deref`.
    #[error(transparent)]
    Api(Box<ApiError>),

    /// The response body could not be deserialized into the expected type.
    #[error("failed to decode response body: {source}")]
    Decode {
        /// The underlying deserialization error.
        #[source]
        source: serde_json::Error,
        /// The raw response body that failed to decode, retained for debugging.
        body: String,
    },

    /// A transport error occurred (DNS, connection, TLS, timeout, ...): no API
    /// response was received. A mutation may or may not have executed; resend
    /// it with [`Error::idempotency_key`] to get the stored result instead of a
    /// duplicate.
    #[error("http request failed: {source}")]
    Http {
        /// The underlying transport error.
        #[source]
        source: reqwest::Error,
        /// The `Idempotency-Key` the request carried (caller-supplied or
        /// auto-generated), if any.
        idempotency_key: Option<String>,
    },
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Error::Api(Box::new(err))
    }
}

impl Error {
    /// Whether resending the request may succeed: a transport failure, or an
    /// [`ApiError`] flagged [`retryable`](ApiError::retryable). The SDK has
    /// already retried what it safely could; mutations that failed with a 5xx
    /// are **not** auto-retried, so look the resource up before resending.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Http { .. } => true,
            Error::Api(e) => e.retryable,
            _ => false,
        }
    }

    /// The `Idempotency-Key` the failed request carried, if any. Resend with
    /// this key to resume after a timeout or a 5xx.
    pub fn idempotency_key(&self) -> Option<&str> {
        match self {
            Error::Http {
                idempotency_key, ..
            } => idempotency_key.as_deref(),
            Error::Api(e) => e.idempotency_key.as_deref(),
            _ => None,
        }
    }

    /// The HTTP status of an [`Error::Api`], if this is one.
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api(e) => Some(e.status),
            _ => None,
        }
    }
}

/// A structured error returned by the BlindPay API on a non-2xx response.
///
/// Only `status` and `message` are always present: auth and rate-limit errors
/// carry no `trace_id`, internal errors carry no `errors`, and idempotency
/// errors carry no `code`. Its `Display` never includes request bodies or
/// credentials.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiError {
    /// The HTTP status code of the response.
    pub status: u16,
    /// The machine-readable error code (UPPERCASE, e.g.
    /// `PAYOUTS_INSUFFICIENT_BALANCE`), when the body has one. This is the
    /// stable key to match on.
    pub code: Option<String>,
    /// A short message parsed from the body (e.g. `not_enough_balance`), or a
    /// synthesized `HTTP <status> <reason>` when the body has none or isn't
    /// JSON.
    pub message: String,
    /// A human-readable explanation, when the body has one.
    pub description: Option<String>,
    /// The body's raw `errors` array (validation details); empty when absent.
    pub errors: Vec<serde_json::Value>,
    /// The server trace id, when the body has one. Quote it to support.
    pub trace_id: Option<String>,
    /// The `X-Request-Id` response header.
    pub request_id: Option<String>,
    /// The `Idempotency-Key` the request carried (caller-supplied or
    /// auto-generated), if any.
    pub idempotency_key: Option<String>,
    /// Whether resending may succeed (429, an in-flight idempotency 409, 408,
    /// or a 5xx other than a terminal code). For a mutation that hit a 5xx this
    /// is `true` but the SDK did **not** retry it: the server releases the
    /// idempotency key on 5xx, so check state before resending with
    /// [`idempotency_key`](Self::idempotency_key).
    pub retryable: bool,
    /// The parsed `Retry-After` header (integer seconds), if present.
    pub retry_after: Option<Duration>,
    /// The raw, unparsed response body. Useful for debugging schema drift.
    pub raw_body: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blindpay api error (status {}", self.status)?;
        if let Some(code) = &self.code {
            write!(f, ", code {code}")?;
        }
        write!(f, "): {}", self.message)?;
        if let Some(description) = &self.description {
            write!(f, " ({description})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}
