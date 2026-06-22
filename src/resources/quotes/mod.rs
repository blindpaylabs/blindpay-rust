//! The `quotes` resource: payout quotes and FX-rate lookups.
//!
//! A created [`Quote`]'s `id` is the `quote_id` consumed when creating a payout.

mod types;

pub use types::{
    CreateQuoteInput, GetFxRateInput, Quote, QuoteContract, QuoteContractNetwork, QuoteFx,
};

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::error::Result;

/// Handle for the `quotes` resource.
///
/// Obtained from the `quotes` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Quotes {
    client: Arc<Inner>,
}

impl Quotes {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Creates a payout quote. The returned [`Quote::id`] is used to create a
    /// payout.
    ///
    /// `POST /instances/{instance_id}/quotes`
    pub async fn create(&self, input: &CreateQuoteInput) -> Result<Quote> {
        let path = format!("/instances/{}/quotes", self.client.instance_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(input))
            .await
    }

    /// Returns an FX-rate quote without creating a payout quote.
    ///
    /// `POST /instances/{instance_id}/quotes/fx`
    pub async fn get_fx_rate(&self, input: &GetFxRateInput) -> Result<QuoteFx> {
        let path = format!("/instances/{}/quotes/fx", self.client.instance_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(input))
            .await
    }
}
