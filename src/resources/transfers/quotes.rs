//! The `transfers.quotes` sub-resource: transfer-quote creation.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::error::Result;
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
        let path = format!("/instances/{}/transfer-quotes", self.client.instance_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(input))
            .await
    }
}
