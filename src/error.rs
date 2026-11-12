//! Error and result types for the BlindPay SDK.

use std::fmt;
use std::time::Duration;

pub use crate::error_code::ErrorCode;

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
        /// The (2xx) HTTP status of the response: the request succeeded.
        status: u16,
        /// The `Idempotency-Key` the request carried, if any. Resending with it
        /// replays the stored response instead of running the request again.
        idempotency_key: Option<String>,
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
            // A request that couldn't even be built will fail the same way again.
            Error::Http { source, .. } => !source.is_builder(),
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
            Error::Decode {
                idempotency_key, ..
            } => idempotency_key.as_deref(),
            _ => None,
        }
    }

    /// The HTTP status of an [`Error::Api`] or [`Error::Decode`].
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api(e) => Some(e.status),
            Error::Decode { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// The typed error code of an [`Error::Api`] (see
    /// [`ApiError::error_code`]); `None` for other variants.
    pub fn error_code(&self) -> Option<ErrorCode> {
        match self {
            Error::Api(e) => e.error_code(),
            _ => None,
        }
    }

    /// The documented retry class of an [`Error::Api`]'s code (see
    /// [`ApiError::retry_class`]); [`RetryClass::Unknown`] for other variants.
    pub fn retry_class(&self) -> RetryClass {
        match self {
            Error::Api(e) => e.retry_class(),
            _ => RetryClass::Unknown,
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

impl ApiError {
    /// The body's `code` as an [`ErrorCode`], or `None` when the body has no
    /// `code` (idempotency errors, some legacy paths: match on
    /// [`message`](Self::message) there). A code this SDK version doesn't
    /// know is [`ErrorCode::Unknown`] with the raw string.
    ///
    /// ```
    /// # fn handle(err: &blindpay::ApiError) {
    /// use blindpay::ErrorCode;
    /// if err.error_code() == Some(ErrorCode::TransfersSwapAmountBelowMinimum) {
    ///     // Swaps between different tokens need at least 100 USDC.
    /// }
    /// # }
    /// ```
    pub fn error_code(&self) -> Option<ErrorCode> {
        self.code.as_deref().map(ErrorCode::from)
    }

    /// What the API documents about retrying this error's code:
    /// [`RetryClass::Unknown`] when there is no code or the code is unknown.
    ///
    /// This is catalogue metadata for your own handling. It doesn't change
    /// what the SDK resends, and it can differ from
    /// [`retryable`](Self::retryable), which describes this request (method,
    /// idempotency key, status).
    pub fn retry_class(&self) -> RetryClass {
        self.error_code()
            .and_then(|code| code.info())
            .map_or(RetryClass::Unknown, |info| info.retry)
    }
}

/// How the API documents retrying an error code (its error table's "Retry"
/// column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RetryClass {
    /// The same request can be retried as is.
    Retryable,
    /// Don't retry.
    Terminal,
    /// The request succeeds only after you fix something (fund a balance,
    /// correct bank details, ...).
    Actionable,
    /// No code, or a code this SDK version doesn't know.
    Unknown,
}

impl RetryClass {
    /// The table's value: `retryable`, `terminal`, `actionable` or `unknown`.
    pub fn as_str(&self) -> &'static str {
        match self {
            RetryClass::Retryable => "retryable",
            RetryClass::Terminal => "terminal",
            RetryClass::Actionable => "actionable",
            RetryClass::Unknown => "unknown",
        }
    }
}

impl fmt::Display for RetryClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What the API documents about an [`ErrorCode`]; see [`ErrorCode::info`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ErrorCodeInfo {
    /// The HTTP status the code typically comes with (an endpoint may use
    /// another).
    pub status: u16,
    /// How to retry it.
    pub retry: RetryClass,
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
