//! The `transfers` resource: stablecoin transfers between blockchain wallets,
//! plus the `transfers.quotes` sub-resource for creating transfer quotes.

mod quotes;
mod types;

pub use quotes::TransferQuotes;
pub use types::{
    CreateTransferInput, CreateTransferQuoteInput, CreateTransferResponse, Transfer, TransferQuote,
    TransferTrackingStep, TransferTrackingTransactionMonitoring,
};

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::{ListResponse, PaginationParams};
use crate::error::{Error, Result};
use crate::internal::encode_path_segment;

/// Handle for the `transfers` resource.
///
/// Obtained from the `transfers` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Transfers {
    client: Arc<Inner>,
    /// Sub-resource for creating transfer quotes.
    pub quotes: TransferQuotes,
}

impl Transfers {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self {
            quotes: TransferQuotes::new(Arc::clone(&client)),
            client,
        }
    }

    /// Creates a transfer from a previously created transfer quote.
    ///
    /// `POST /instances/{instance_id}/transfers`
    pub async fn create(&self, input: &CreateTransferInput) -> Result<CreateTransferResponse> {
        let path = format!("/instances/{}/transfers", self.client.instance_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(input))
            .await
    }

    /// Lists transfers for the instance. Always paginated.
    ///
    /// `GET /instances/{instance_id}/transfers`
    pub async fn list(&self, params: &PaginationParams) -> Result<ListResponse<Transfer>> {
        let path = format!("/instances/{}/transfers", self.client.instance_id);
        self.client.get_query(&path, params).await
    }

    /// Retrieves a single transfer by ID.
    ///
    /// `GET /instances/{instance_id}/transfers/{id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if `id` is empty.
    pub async fn get(&self, id: impl AsRef<str>) -> Result<Transfer> {
        let id = id.as_ref().trim();
        if id.is_empty() {
            return Err(Error::Config("transfer id cannot be empty".to_string()));
        }
        let path = format!(
            "/instances/{}/transfers/{}",
            self.client.instance_id,
            encode_path_segment(id)
        );
        self.client.get(&path).await
    }

    /// Retrieves a transfer through the unauthenticated tracking endpoint.
    ///
    /// `GET /e/transfers/{id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if `id` is empty.
    pub async fn get_track(&self, id: impl AsRef<str>) -> Result<Transfer> {
        let id = id.as_ref().trim();
        if id.is_empty() {
            return Err(Error::Config("transfer id cannot be empty".to_string()));
        }
        let path = format!("/e/transfers/{}", encode_path_segment(id));
        self.client.get(&path).await
    }
}
