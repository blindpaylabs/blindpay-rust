//! The `transfers.quotes` sub-resource: transfer-quote creation.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::error::Result;
use crate::options::RequestOptions;
use crate::resources::transfers::types::{CreateTransferQuoteInput, TransferQuote};

/// Handle for the `transfers.quotes` sub-resource.
///
/// Obtained from the `quotes` field of a [`Transfers`](super::Transfers) handle.
#[derive(Clone)]
pub struct TransferQuotes {
    client: Arc<Inner>,
}

impl TransferQuotes {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Creates a transfer quote.
    ///
    /// `POST /instances/{instance_id}/transfer-quotes`
    pub async fn create(&self, input: &CreateTransferQuoteInput) -> Result<TransferQuote> {
        self.create_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/transfer-quotes`
    pub async fn create_with_options(
        &self,
        input: &CreateTransferQuoteInput,
        options: &RequestOptions,
    ) -> Result<TransferQuote> {
        let path = self.client.instance_path("/transfer-quotes");
        self.client
            .request_with(Method::POST, &path, None::<&()>, Some(input), options)
            .await
    }
}
