//! The `payins.quotes` sub-resource: payin quotes and FX-rate previews.

use std::sync::Arc;

use crate::client::Inner;
use crate::error::Result;

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
        let path = format!("/instances/{}/payin-quotes", self.client.instance_id);
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Previews the FX rate for a prospective payin.
    ///
    /// `POST /instances/{instance_id}/payin-quotes/fx`
    pub async fn get_fx_rate(&self, input: &PayinQuoteFxInput) -> Result<PayinQuoteFx> {
        let path = format!("/instances/{}/payin-quotes/fx", self.client.instance_id);
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }
}
