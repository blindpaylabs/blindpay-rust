//! The `payins.quotes` sub-resource: payin quotes and FX-rate previews.

use std::sync::Arc;

use crate::client::Inner;
use crate::error::Result;
use crate::options::RequestOptions;

use super::types::{CreatePayinQuoteInput, PayinQuote, PayinQuoteFx, PayinQuoteFxInput};

/// Handle for the `payins.quotes` sub-resource.
///
/// Obtained from the `quotes` field of a [`Payins`](super::Payins) handle. Cheap
/// to clone — its state is reference-counted.
#[derive(Clone)]
pub struct PayinQuotes {
    client: Arc<Inner>,
}

impl PayinQuotes {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Creates a payin quote.
    ///
    /// `POST /instances/{instance_id}/payin-quotes`
    pub async fn create(&self, input: &CreatePayinQuoteInput) -> Result<PayinQuote> {
        self.create_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payin-quotes`
    pub async fn create_with_options(
        &self,
        input: &CreatePayinQuoteInput,
        options: &RequestOptions,
    ) -> Result<PayinQuote> {
        let path = self.client.instance_path("/payin-quotes");
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Previews the FX rate for a prospective payin.
    ///
    /// `POST /instances/{instance_id}/payin-quotes/fx`
    pub async fn get_fx_rate(&self, input: &PayinQuoteFxInput) -> Result<PayinQuoteFx> {
        self.get_fx_rate_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::get_fx_rate`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payin-quotes/fx`
    pub async fn get_fx_rate_with_options(
        &self,
        input: &PayinQuoteFxInput,
        options: &RequestOptions,
    ) -> Result<PayinQuoteFx> {
        let path = self.client.instance_path("/payin-quotes/fx");
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }
}
