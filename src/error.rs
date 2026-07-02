//! Error and result types for the BlindPay SDK.

use std::fmt;

/// A specialized [`Result`](std::result::Result) for fallible BlindPay operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the BlindPay SDK.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The client was built with invalid configuration, or a method was called
    /// with an invalid argument (for example an empty API key or SWIFT code).
    #[error("invalid configuration: {0}")]
    Config(String),

    /// The BlindPay API returned a non-success (non-2xx) HTTP status.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// The response body could not be deserialized into the expected type.
    #[error("failed to decode response body: {source}")]
    Decode {
        /// The underlying deserialization error.
        #[source]
        source: serde_json::Error,
        /// The raw response body that failed to decode, retained for debugging.
        body: String,
    },

    /// An HTTP transport error occurred (DNS, connection, TLS, timeout, ...).
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
}

/// A structured error returned by the BlindPay API on a non-2xx response.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiError {
    /// The HTTP status code of the response.
    pub status: u16,
    /// A human-readable message parsed from the response body, or a synthesized
    /// message when the body has no `message` field.
    pub message: String,
    /// The raw, unparsed response body. Useful for debugging schema drift.
    pub raw_body: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "blindpay api error (status {}): {}",
            self.status, self.message
        )
    }
}

impl std::error::Error for ApiError {}
