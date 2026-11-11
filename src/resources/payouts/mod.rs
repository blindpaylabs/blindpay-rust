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
    SubmitPayoutDocumentsInput, SubmitPayoutDocumentsResponse, TransactionDocumentType,
};

use std::sync::Arc;

use crate::client::Inner;
use crate::common::{ListResponse, Pager};
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

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
        self.create_evm_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_evm`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payouts/evm`
    pub async fn create_evm_with_options(
        &self,
        input: &CreateEvmPayoutInput,
        options: &RequestOptions,
    ) -> Result<CreatePayoutResponse> {
        let path = format!("/instances/{}/payouts/evm", self.client.instance_id);
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Creates a payout on Solana from a previously created quote.
    ///
    /// `POST /instances/{instance_id}/payouts/solana`
    pub async fn create_solana(
        &self,
        input: &CreateSolanaPayoutInput,
    ) -> Result<CreatePayoutResponse> {
        self.create_solana_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_solana`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payouts/solana`
    pub async fn create_solana_with_options(
        &self,
        input: &CreateSolanaPayoutInput,
        options: &RequestOptions,
    ) -> Result<CreatePayoutResponse> {
        let path = format!("/instances/{}/payouts/solana", self.client.instance_id);
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Creates a payout on Stellar from a previously created quote.
    ///
    /// `POST /instances/{instance_id}/payouts/stellar`
    pub async fn create_stellar(
        &self,
        input: &CreateStellarPayoutInput,
    ) -> Result<CreatePayoutResponse> {
        self.create_stellar_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_stellar`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payouts/stellar`
    pub async fn create_stellar_with_options(
        &self,
        input: &CreateStellarPayoutInput,
        options: &RequestOptions,
    ) -> Result<CreatePayoutResponse> {
        let path = format!("/instances/{}/payouts/stellar", self.client.instance_id);
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Authorizes the Stellar token (trustline) for a payout.
    ///
    /// `POST /instances/{instance_id}/payouts/stellar/authorize`
    pub async fn authorize_stellar_token(
        &self,
        input: &AuthorizeStellarTokenInput,
    ) -> Result<AuthorizeStellarTokenResponse> {
        self.authorize_stellar_token_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::authorize_stellar_token`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payouts/stellar/authorize`
    pub async fn authorize_stellar_token_with_options(
        &self,
        input: &AuthorizeStellarTokenInput,
        options: &RequestOptions,
    ) -> Result<AuthorizeStellarTokenResponse> {
        let path = format!(
            "/instances/{}/payouts/stellar/authorize",
            self.client.instance_id
        );
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
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
        self.submit_documents_with_options(payout_id, input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::submit_documents`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payouts/{id}/documents`
    pub async fn submit_documents_with_options(
        &self,
        payout_id: impl AsRef<str>,
        input: &SubmitPayoutDocumentsInput,
        options: &RequestOptions,
    ) -> Result<SubmitPayoutDocumentsResponse> {
        let path = format!(
            "/instances/{}/payouts/{}/documents",
            self.client.instance_id,
            encode_path_segment(payout_id.as_ref())
        );
        self.client
            .request_with::<_, (), _>(reqwest::Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Lists payouts for the instance.
    ///
    /// `GET /instances/{instance_id}/payouts`
    pub async fn list(&self, params: &ListPayoutsParams) -> Result<ListResponse<Payout>> {
        let path = format!("/instances/{}/payouts", self.client.instance_id);
        self.client.get_query(&path, params).await
    }

    /// Iterates every payout matching `params`, page by page (see [`Pager`]).
    /// Sends `limit` (100 unless set) and follows `pagination.next_page`.
    ///
    /// `GET /instances/{instance_id}/payouts`
    pub fn list_auto_paging(&self, params: &ListPayoutsParams) -> Pager<Payout> {
        let path = format!("/instances/{}/payouts", self.client.instance_id);
        Pager::new(Arc::clone(&self.client), path, params)
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

    /// Retrieves a payout via its public tracking endpoint.
    ///
    /// `GET /e/payouts/{id}`
    ///
    /// Public tracking endpoint for BlindPay's web and iOS apps: server-side
    /// calls get **401 in production** (it only admits the app origin).
    /// Use [`get`](Payouts::get) instead. Kept for completeness.
    pub async fn get_track(&self, payout_id: impl AsRef<str>) -> Result<Payout> {
        let path = format!("/e/payouts/{}", encode_path_segment(payout_id.as_ref()));
        self.client.get(&path).await
    }
}
