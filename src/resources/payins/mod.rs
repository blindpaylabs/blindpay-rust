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
use crate::common::ListResponse;
use crate::error::Result;
use crate::internal::encode_path_segment;

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
        let path = format!("/instances/{}/payins/evm", self.client.instance_id);
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Lists payins for the instance.
    ///
    /// `GET /instances/{instance_id}/payins`
    pub async fn list(&self, params: &ListPayinsParams) -> Result<ListResponse<Payin>> {
        let path = format!("/instances/{}/payins", self.client.instance_id);
        self.client.get_query(&path, params).await
    }

    /// Retrieves a single payin by id.
    ///
    /// `GET /instances/{instance_id}/payins/{id}`
    pub async fn get(&self, payin_id: impl AsRef<str>) -> Result<Payin> {
        let path = format!(
            "/instances/{}/payins/{}",
            self.client.instance_id,
            encode_path_segment(payin_id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Retrieves a payin via its public, unauthenticated tracking endpoint.
    ///
    /// `GET /e/payins/{id}`
    ///
    /// This `/e/` route is the obfuscated, end-customer tracking endpoint: it is
    /// served without instance scoping and is not gated by the API key (the SDK
    /// still sends the bearer header). Use [`get`](Payins::get) for the
    /// authenticated, instance-scoped lookup.
    pub async fn get_track(&self, payin_id: impl AsRef<str>) -> Result<Payin> {
        let path = format!("/e/payins/{}", encode_path_segment(payin_id.as_ref()));
        self.client.get(&path).await
    }
}
