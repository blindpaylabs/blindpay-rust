//! The `payouts` resource: outbound (stablecoin → fiat) payments.
//!
//! Payouts have no generic `create`: they are chain-specific
//! ([`create_evm`](Payouts::create_evm), [`create_solana`](Payouts::create_solana),
//! [`create_stellar`](Payouts::create_stellar)) and each consumes a `quote_id`
//! produced by the `quotes` resource.

mod types;

pub use types::{
    AuthorizeStellarTokenInput, AuthorizeStellarTokenResponse, CreateEvmPayoutInput,
    CreatePayoutResponse, CreateSolanaPayoutInput, CreateStellarPayoutInput,
    EstimatedTimeOfArrival, ListPayoutsParams, Payout, PayoutCompleteStatus, PayoutDocumentsStatus,
    PayoutLiquidityProviderStatus, PayoutPaymentProviderStatus, PayoutTrackingComplete,
    PayoutTrackingDocuments, PayoutTrackingLiquidity, PayoutTrackingPartnerFee,
    PayoutTrackingPayment, PayoutTrackingStep, PayoutTrackingTransaction, PayoutTransactionStatus,
    ProviderName, SubmitPayoutDocumentsInput, SubmitPayoutDocumentsResponse,
    TransactionDocumentType,
};

use std::sync::Arc;

use crate::client::Inner;
use crate::common::ListResponse;
use crate::error::Result;
use crate::internal::encode_path_segment;

/// Handle for the `payouts` resource.
///
/// Obtained from the `payouts` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Payouts {
    client: Arc<Inner>,
}

impl Payouts {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Creates a payout on an EVM chain from a previously created quote.
    ///
    /// `POST /instances/{instance_id}/payouts/evm`
    pub async fn create_evm(&self, input: &CreateEvmPayoutInput) -> Result<CreatePayoutResponse> {
        let path = format!("/instances/{}/payouts/evm", self.client.instance_id);
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Creates a payout on Solana from a previously created quote.
    ///
    /// `POST /instances/{instance_id}/payouts/solana`
    pub async fn create_solana(
        &self,
        input: &CreateSolanaPayoutInput,
    ) -> Result<CreatePayoutResponse> {
        let path = format!("/instances/{}/payouts/solana", self.client.instance_id);
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Creates a payout on Stellar from a previously created quote.
    ///
    /// `POST /instances/{instance_id}/payouts/stellar`
    pub async fn create_stellar(
        &self,
        input: &CreateStellarPayoutInput,
    ) -> Result<CreatePayoutResponse> {
        let path = format!("/instances/{}/payouts/stellar", self.client.instance_id);
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Authorizes the Stellar token (trustline) for a payout.
    ///
    /// `POST /instances/{instance_id}/payouts/stellar/authorize`
    pub async fn authorize_stellar_token(
        &self,
        input: &AuthorizeStellarTokenInput,
    ) -> Result<AuthorizeStellarTokenResponse> {
        let path = format!(
            "/instances/{}/payouts/stellar/authorize",
            self.client.instance_id
        );
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Submits supporting documents for a payout (e.g. for SWIFT compliance).
    ///
    /// `POST /instances/{instance_id}/payouts/{id}/documents`
    pub async fn submit_documents(
        &self,
        payout_id: impl AsRef<str>,
        input: &SubmitPayoutDocumentsInput,
    ) -> Result<SubmitPayoutDocumentsResponse> {
        let path = format!(
            "/instances/{}/payouts/{}/documents",
            self.client.instance_id,
            encode_path_segment(payout_id.as_ref())
        );
        self.client
            .request::<_, (), _>(reqwest::Method::POST, &path, None, Some(input))
            .await
    }

    /// Lists payouts for the instance.
    ///
    /// `GET /instances/{instance_id}/payouts`
    pub async fn list(&self, params: &ListPayoutsParams) -> Result<ListResponse<Payout>> {
        let path = format!("/instances/{}/payouts", self.client.instance_id);
        self.client.get_query(&path, params).await
    }

    /// Retrieves a single payout by id.
    ///
    /// `GET /instances/{instance_id}/payouts/{id}`
    pub async fn get(&self, payout_id: impl AsRef<str>) -> Result<Payout> {
        let path = format!(
            "/instances/{}/payouts/{}",
            self.client.instance_id,
            encode_path_segment(payout_id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Retrieves a payout via its public, unauthenticated tracking endpoint.
    ///
    /// `GET /e/payouts/{id}`
    ///
    /// This `/e/` route is the end-customer tracking endpoint: it is served
    /// without instance scoping and is not gated by the API key (the SDK still
    /// sends the bearer header). Use [`get`](Payouts::get) for the
    /// authenticated, instance-scoped lookup.
    pub async fn get_track(&self, payout_id: impl AsRef<str>) -> Result<Payout> {
        let path = format!("/e/payouts/{}", encode_path_segment(payout_id.as_ref()));
        self.client.get(&path).await
    }
}
