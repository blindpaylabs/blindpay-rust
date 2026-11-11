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
use crate::options::RequestOptions;

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
        self.create_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/quotes`
    pub async fn create_with_options(
        &self,
        input: &CreateQuoteInput,
        options: &RequestOptions,
    ) -> Result<Quote> {
        let path = format!("/instances/{}/quotes", self.client.instance_id);
        self.client
            .request_with(Method::POST, &path, None::<&()>, Some(input), options)
            .await
    }

    /// Returns an FX-rate quote without creating a payout quote.
    ///
    /// `POST /instances/{instance_id}/quotes/fx`
    pub async fn get_fx_rate(&self, input: &GetFxRateInput) -> Result<QuoteFx> {
        self.get_fx_rate_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::get_fx_rate`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/quotes/fx`
    pub async fn get_fx_rate_with_options(
        &self,
        input: &GetFxRateInput,
        options: &RequestOptions,
    ) -> Result<QuoteFx> {
        let path = format!("/instances/{}/quotes/fx", self.client.instance_id);
        self.client
            .request_with(Method::POST, &path, None::<&()>, Some(input), options)
            .await
    }
}
