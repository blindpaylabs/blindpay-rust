//! The `payins` resource: inbound (fiat → stablecoin) payments and their quotes.

mod quotes;
mod types;

pub use quotes::PayinQuotes;
pub use types::{
    BlindpayBankAccount, BlindpayBankDetails, BlindpayBankParty, CreatePayinInput,
    CreatePayinQuoteInput, CreatePayinResponse, ListPayinsParams, PayerRules, Payin, PayinQuote,
    PayinQuoteFx, PayinQuoteFxInput, PayinTrackingComplete, PayinTrackingPartnerFee,
    PayinTrackingPayment, PayinTrackingTransaction, PayinTransactionStatus, PseDocumentType,
    PseInstruction, SwiftReceivingBank, TedInstruction, TransfersInstruction,
};

use std::sync::Arc;

use crate::client::Inner;
use crate::common::{ListResponse, Pager};
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

/// Handle for the `payins` resource.
///
/// Obtained from the `payins` field of a [`BlindPay`](crate::BlindPay) client. The `quotes` sub-handle
/// builds payin quotes and FX previews.
#[derive(Clone)]
pub struct Payins {
    client: Arc<Inner>,
    /// Payin quotes (`payin-quotes`) sub-resource.
    pub quotes: PayinQuotes,
}

impl Payins {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self {
            quotes: PayinQuotes::new(Arc::clone(&client)),
            client,
        }
    }

    /// Creates an EVM payin from a previously created payin quote.
    ///
    /// `POST /instances/{instance_id}/payins/evm`
    pub async fn create_evm(&self, input: &CreatePayinInput) -> Result<CreatePayinResponse> {
        self.create_evm_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_evm`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payins/evm`
    pub async fn create_evm_with_options(
        &self,
        input: &CreatePayinInput,
        options: &RequestOptions,
    ) -> Result<CreatePayinResponse> {
        let path = self.client.instance_path("/payins/evm");
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Lists payins for the instance.
    ///
    /// `GET /instances/{instance_id}/payins`
    pub async fn list(&self, params: &ListPayinsParams) -> Result<ListResponse<Payin>> {
        let path = self.client.instance_path("/payins");
        self.client.get_query(&path, params).await
    }

    /// Iterates every payin matching `params`, page by page (see [`Pager`]).
    /// Sends `limit` (100 unless set) and follows `pagination.next_page`.
    ///
    /// `GET /instances/{instance_id}/payins`
    pub fn list_auto_paging(&self, params: &ListPayinsParams) -> Pager<Payin> {
        let path = self.client.instance_path("/payins");
        Pager::new(Arc::clone(&self.client), path, params)
    }

    /// Retrieves a single payin by id.
    ///
    /// `GET /instances/{instance_id}/payins/{id}`
    pub async fn get(&self, payin_id: impl AsRef<str>) -> Result<Payin> {
        let path = self.client.instance_path(&format!(
            "/payins/{}",
            encode_path_segment(payin_id.as_ref())
        ));
        self.client.get(&path).await
    }

    /// Retrieves a payin via its public tracking endpoint. `checkout_token`
    /// is the hosted-checkout token, when the payin has one.
    ///
    /// `GET /e/payins/{id}`
    ///
    /// Public tracking endpoint for BlindPay's web and iOS apps: server-side
    /// calls get **401 in production** (it only admits the app origin).
    /// Use [`get`](Payins::get) instead. Kept for completeness.
    pub async fn get_track(
        &self,
        payin_id: impl AsRef<str>,
        checkout_token: Option<&str>,
    ) -> Result<Payin> {
        #[derive(serde::Serialize)]
        struct Query<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            checkout_token: Option<&'a str>,
        }
        let path = format!("/e/payins/{}", encode_path_segment(payin_id.as_ref()));
        self.client
            .get_query(&path, &Query { checkout_token })
            .await
    }
}
