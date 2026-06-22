//! Request and response types for the `payouts` resource.

use serde::{Deserialize, Serialize};

use crate::common::{
    AccountClass, AccountType, Country, Currency, Network, Rail, Token, TransactionStatus,
    open_enum,
};

open_enum! {
    /// The step a payout tracking stage has reached.
    ///
    /// Distinct from the payout's overall [`TransactionStatus`]: this is the
    /// per-stage progression reported inside the `tracking_*` objects.
    pub enum PayoutTrackingStep {
        /// Stage is being processed.
        Processing => "processing",
        /// Stage is temporarily on hold.
        OnHold => "on_hold",
        /// Stage is awaiting manual review.
        PendingReview => "pending_review",
        /// Stage completed.
        Completed => "completed",
    }
}

open_enum! {
    /// Estimated time of arrival reported in payout tracking stages.
    pub enum EstimatedTimeOfArrival {
        /// Within 5 minutes.
        FiveMin => "5_min",
        /// Between 5 and 30 minutes.
        FiveToThirtyMin => "5_30_min",
        /// Within 30 minutes.
        ThirtyMin => "30_min",
        /// Within 2 hours.
        TwoHours => "2_hours",
        /// Within 1 business day.
        OneBusinessDay => "1_business_day",
        /// Within 2 business days.
        TwoBusinessDays => "2_business_days",
        /// Within 5 business days.
        FiveBusinessDays => "5_business_days",
    }
}

open_enum! {
    /// Transaction-level status reported inside `tracking_transaction`.
    pub enum PayoutTransactionStatus {
        /// The on-chain transaction failed.
        Failed => "failed",
        /// The on-chain transaction was found.
        Found => "found",
    }
}

open_enum! {
    /// Provider status reported inside `tracking_payment`.
    pub enum PayoutPaymentProviderStatus {
        /// The payment was canceled.
        Canceled => "canceled",
        /// The payment failed.
        Failed => "failed",
        /// The payment was returned.
        Returned => "returned",
        /// The payment was sent.
        Sent => "sent",
    }
}

open_enum! {
    /// Provider status reported inside `tracking_liquidity`.
    pub enum PayoutLiquidityProviderStatus {
        /// Funds were deposited.
        Deposited => "deposited",
        /// Funds were converted.
        Converted => "converted",
        /// Funds were withdrawn.
        Withdrawn => "withdrawn",
    }
}

open_enum! {
    /// Completion status reported inside `tracking_complete`.
    pub enum PayoutCompleteStatus {
        /// Tokens were refunded to the sender.
        TokensRefunded => "tokens_refunded",
        /// The payout was paid out.
        Paid => "paid",
    }
}

open_enum! {
    /// Documents status reported inside `tracking_documents`.
    pub enum PayoutDocumentsStatus {
        /// Awaiting supporting documents from the sender.
        WaitingDocuments => "waiting_documents",
        /// Documents are under compliance review.
        ComplianceReviewing => "compliance_reviewing",
    }
}

/// The name of the payment provider that processed a payout leg.
///
/// The API defines this as a large, evolving vendor set (and the wire values use
/// vendor display names like `"JPMorgan Chase"`), so it is modeled as a
/// transparent newtype rather than an enumerated type — strongly typed (never a
/// bare `String`) yet forward-compatible with new providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ProviderName(String);

impl ProviderName {
    /// Returns the provider name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ProviderName {
    fn from(value: &str) -> Self {
        ProviderName(value.to_string())
    }
}

impl From<String> for ProviderName {
    fn from(value: String) -> Self {
        ProviderName(value)
    }
}

impl AsRef<str> for ProviderName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

open_enum! {
    /// The type of a transaction supporting document.
    pub enum TransactionDocumentType {
        /// An invoice.
        Invoice => "invoice",
        /// A purchase order.
        PurchaseOrder => "purchase_order",
        /// A delivery slip.
        DeliverySlip => "delivery_slip",
        /// A contract.
        Contract => "contract",
        /// A customs declaration.
        CustomsDeclaration => "customs_declaration",
        /// A bill of lading.
        BillOfLading => "bill_of_lading",
        /// Any other document type.
        Others => "others",
    }
}

/// The on-chain (token) leg of a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayoutTrackingTransaction {
    /// The step this stage has reached.
    pub step: PayoutTrackingStep,
    /// Transaction-level status (`failed` / `found`), if known.
    #[serde(default)]
    pub status: Option<PayoutTransactionStatus>,
    /// Blockchain transaction hash.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// When the stage completed (ISO 8601).
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Provider-side handle for the on-chain leg (e.g. Circle CPN payment id).
    #[serde(default)]
    pub provider_transaction_id: Option<String>,
    /// Ledger transaction id for the inbound (crypto deposit) leg.
    #[serde(default)]
    pub ledger_in_transaction_id: Option<String>,
    /// Ledger transaction id for the outbound (fiat withdrawal) leg.
    #[serde(default)]
    pub ledger_out_transaction_id: Option<String>,
}

/// The fiat-payment leg of a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayoutTrackingPayment {
    /// The step this stage has reached.
    pub step: PayoutTrackingStep,
    /// Payment provider name.
    #[serde(default)]
    pub provider_name: Option<ProviderName>,
    /// Payment provider transaction id.
    #[serde(default)]
    pub provider_transaction_id: Option<String>,
    /// Payment provider status (`canceled` / `failed` / `returned` / `sent`).
    #[serde(default)]
    pub provider_status: Option<PayoutPaymentProviderStatus>,
    /// Provider error reason when the payment fails.
    #[serde(default)]
    pub provider_error_reason: Option<String>,
    /// Unique end-to-end transaction reference (UETR).
    #[serde(default)]
    pub provider_uetr: Option<String>,
    /// Fed Input Message Accountability Data (IMAD); domestic Fedwire only.
    #[serde(default)]
    pub provider_imad: Option<String>,
    /// Clearing system that processed the payment (e.g. `FED`, `CHIPS`).
    #[serde(default)]
    pub provider_clearing_system: Option<String>,
    /// Recipient name.
    #[serde(default)]
    pub recipient_name: Option<String>,
    /// Recipient tax id.
    #[serde(default)]
    pub recipient_tax_id: Option<String>,
    /// Recipient bank code.
    #[serde(default)]
    pub recipient_bank_code: Option<String>,
    /// Recipient branch code.
    #[serde(default)]
    pub recipient_branch_code: Option<String>,
    /// Recipient account number.
    #[serde(default)]
    pub recipient_account_number: Option<String>,
    /// Recipient account type.
    #[serde(default)]
    pub recipient_account_type: Option<String>,
    /// COELSA transaction reference id for ARS transfers.
    #[serde(default)]
    pub coelsa_id: Option<String>,
    /// BACEN Pix end-to-end transaction id.
    #[serde(default)]
    pub end_to_end_id: Option<String>,
    /// Estimated time of arrival.
    #[serde(default)]
    pub estimated_time_of_arrival: Option<EstimatedTimeOfArrival>,
    /// When the stage completed (ISO 8601).
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// The liquidity-provisioning leg of a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayoutTrackingLiquidity {
    /// The step this stage has reached.
    pub step: PayoutTrackingStep,
    /// Payment provider transaction id.
    #[serde(default)]
    pub provider_transaction_id: Option<String>,
    /// Provider status (`deposited` / `converted` / `withdrawn`).
    #[serde(default)]
    pub provider_status: Option<PayoutLiquidityProviderStatus>,
    /// Estimated time of arrival.
    #[serde(default)]
    pub estimated_time_of_arrival: Option<EstimatedTimeOfArrival>,
    /// When the stage completed (ISO 8601).
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// The completion leg of a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayoutTrackingComplete {
    /// The step this stage has reached.
    pub step: PayoutTrackingStep,
    /// Completion status (`tokens_refunded` / `paid`).
    #[serde(default)]
    pub status: Option<PayoutCompleteStatus>,
    /// Reason for refund when tokens are returned to the sender.
    #[serde(default)]
    pub refund_reason: Option<String>,
    /// Completion transaction hash.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// When the stage completed (ISO 8601).
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// The partner-fee settlement leg of a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayoutTrackingPartnerFee {
    /// The step this stage has reached.
    pub step: PayoutTrackingStep,
    /// Partner-fee transaction hash.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// When the stage completed (ISO 8601).
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// The supporting-documents leg of a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayoutTrackingDocuments {
    /// The step this stage has reached.
    pub step: PayoutTrackingStep,
    /// Documents status (`waiting_documents` / `compliance_reviewing`).
    #[serde(default)]
    pub status: Option<PayoutDocumentsStatus>,
    /// Email or name of the reviewer.
    #[serde(default)]
    pub reviewed_by: Option<String>,
    /// When the stage completed (ISO 8601).
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// A payout — the large joined object returned by `Payouts::get`,
/// `Payouts::get_track`, and each item of `Payouts::list`.
///
/// Combines the payout record with denormalized receiver, quote, and
/// bank-account fields.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Payout {
    /// Receiver id (`re_…`).
    pub receiver_id: String,
    /// Payout id (`pa_…`).
    pub id: String,
    /// Overall payout status.
    pub status: TransactionStatus,
    /// Sender wallet address.
    pub sender_wallet_address: String,
    /// Signed transaction, for chains that require client-side signing.
    #[serde(default)]
    pub signed_transaction: Option<String>,
    /// Quote id this payout was created from (`qu_…`).
    pub quote_id: String,
    /// Instance id (`in_…`).
    pub instance_id: String,
    /// Partner fee amount in cents (can be negative for rebates).
    #[serde(default)]
    pub partner_fee: Option<i64>,
    /// On-chain (token) tracking leg.
    #[serde(default)]
    pub tracking_transaction: Option<PayoutTrackingTransaction>,
    /// Fiat-payment tracking leg.
    #[serde(default)]
    pub tracking_payment: Option<PayoutTrackingPayment>,
    /// Liquidity tracking leg.
    #[serde(default)]
    pub tracking_liquidity: Option<PayoutTrackingLiquidity>,
    /// Completion tracking leg.
    #[serde(default)]
    pub tracking_complete: Option<PayoutTrackingComplete>,
    /// Partner-fee tracking leg.
    #[serde(default)]
    pub tracking_partner_fee: Option<PayoutTrackingPartnerFee>,
    /// Documents tracking leg.
    #[serde(default)]
    pub tracking_documents: Option<PayoutTrackingDocuments>,
    /// JPMorgan tracking data, when the payout was processed via JPM.
    #[serde(default)]
    pub jpm_track_data: Option<serde_json::Value>,
    /// Circle CPN payment id.
    #[serde(default)]
    pub cpn_payment_id: Option<String>,
    /// Creation timestamp (ISO 8601).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last-update timestamp (ISO 8601).
    #[serde(default)]
    pub updated_at: Option<String>,

    // ---- Denormalized receiver fields ----
    /// Receiver image URL.
    #[serde(default)]
    pub image_url: Option<String>,
    /// Receiver first name.
    #[serde(default)]
    pub first_name: Option<String>,
    /// Receiver last name.
    #[serde(default)]
    pub last_name: Option<String>,
    /// Receiver legal (business) name.
    #[serde(default)]
    pub legal_name: Option<String>,

    // ---- Denormalized quote fields ----
    /// Destination network.
    #[serde(default)]
    pub network: Option<Network>,
    /// Stablecoin token.
    #[serde(default)]
    pub token: Option<Token>,
    /// Payment description / memo.
    #[serde(default)]
    pub description: Option<String>,
    /// Amount the sender sends, in cents.
    #[serde(default)]
    pub sender_amount: Option<i64>,
    /// Amount the receiver receives (in stablecoin), in cents.
    #[serde(default)]
    pub receiver_amount: Option<i64>,
    /// Partner fee id (`pf_…`).
    #[serde(default)]
    pub partner_fee_id: Option<String>,
    /// Partner fee amount in cents.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// Commercial quotation.
    #[serde(default)]
    pub commercial_quotation: Option<i64>,
    /// BlindPay quotation.
    #[serde(default)]
    pub blindpay_quotation: Option<i64>,
    /// Total fee amount in cents.
    #[serde(default)]
    pub total_fee_amount: Option<i64>,
    /// Amount the receiver receives in their local currency, in cents.
    #[serde(default)]
    pub receiver_local_amount: Option<i64>,
    /// Local fiat currency of the receiver.
    #[serde(default)]
    pub currency: Option<Currency>,
    /// Transaction fee amount in cents.
    #[serde(default)]
    pub transaction_fee_amount: Option<i64>,
    /// Billing fee amount in cents.
    #[serde(default)]
    pub billing_fee_amount: Option<i64>,
    /// URL of the supporting transaction document.
    #[serde(default)]
    pub transaction_document_file: Option<String>,
    /// Type of the supporting transaction document.
    #[serde(default)]
    pub transaction_document_type: Option<TransactionDocumentType>,
    /// Identifier of the supporting transaction document.
    #[serde(default)]
    pub transaction_document_id: Option<String>,

    // ---- Denormalized bank-account fields ----
    /// Bank account holder name.
    #[serde(default)]
    pub name: Option<String>,
    /// Bank account rail.
    #[serde(default, rename = "type")]
    pub account_rail: Option<Rail>,
    /// PIX key.
    #[serde(default)]
    pub pix_key: Option<String>,
    /// PIX Safe bank code.
    #[serde(default)]
    pub pix_safe_bank_code: Option<String>,
    /// PIX Safe branch code.
    #[serde(default)]
    pub pix_safe_branch_code: Option<String>,
    /// PIX Safe CPF/CNPJ.
    #[serde(default)]
    pub pix_safe_cpf_cnpj: Option<String>,
    /// TED bank code.
    #[serde(default)]
    pub ted_bank_code: Option<String>,
    /// TED branch code.
    #[serde(default)]
    pub ted_branch_code: Option<String>,
    /// TED CPF/CNPJ.
    #[serde(default)]
    pub ted_cpf_cnpj: Option<String>,
    /// Account number.
    #[serde(default)]
    pub account_number: Option<String>,
    /// Routing number.
    #[serde(default)]
    pub routing_number: Option<String>,
    /// Bank account country.
    #[serde(default)]
    pub country: Option<Country>,
    /// Account class (individual / business).
    #[serde(default)]
    pub account_class: Option<AccountClass>,
    /// Address line 1.
    #[serde(default)]
    pub address_line_1: Option<String>,
    /// Address line 2.
    #[serde(default)]
    pub address_line_2: Option<String>,
    /// City.
    #[serde(default)]
    pub city: Option<String>,
    /// State / province / region.
    #[serde(default)]
    pub state_province_region: Option<String>,
    /// Postal code.
    #[serde(default)]
    pub postal_code: Option<String>,
    /// Bank account type (checking / saving).
    #[serde(default)]
    pub account_type: Option<AccountType>,
    /// ACH COP beneficiary first name.
    #[serde(default)]
    pub ach_cop_beneficiary_first_name: Option<String>,
    /// ACH COP bank account.
    #[serde(default)]
    pub ach_cop_bank_account: Option<String>,
    /// ACH COP bank code.
    #[serde(default)]
    pub ach_cop_bank_code: Option<String>,
    /// ACH COP beneficiary last name.
    #[serde(default)]
    pub ach_cop_beneficiary_last_name: Option<String>,
    /// ACH COP document id.
    #[serde(default)]
    pub ach_cop_document_id: Option<String>,
    /// ACH COP document type.
    #[serde(default)]
    pub ach_cop_document_type: Option<String>,
    /// ACH COP email.
    #[serde(default)]
    pub ach_cop_email: Option<String>,
    /// Beneficiary name.
    #[serde(default)]
    pub beneficiary_name: Option<String>,
    /// SPEI CLABE.
    #[serde(default)]
    pub spei_clabe: Option<String>,
    /// SPEI protocol.
    #[serde(default)]
    pub spei_protocol: Option<String>,
    /// SPEI institution code.
    #[serde(default)]
    pub spei_institution_code: Option<String>,
    /// SWIFT beneficiary country.
    #[serde(default)]
    pub swift_beneficiary_country: Option<Country>,
    /// SWIFT/BIC code.
    #[serde(default)]
    pub swift_code_bic: Option<String>,
    /// SWIFT account holder name.
    #[serde(default)]
    pub swift_account_holder_name: Option<String>,
    /// SWIFT account number / IBAN.
    #[serde(default)]
    pub swift_account_number_iban: Option<String>,
    /// Transfers 3.0 account.
    #[serde(default)]
    pub transfers_account: Option<String>,
    /// Transfers 3.0 account type (`CVU` / `CBU` / `ALIAS`).
    #[serde(default)]
    pub transfers_type: Option<String>,

    /// Whether the payout is associated with a virtual account.
    #[serde(default)]
    pub has_virtual_account: Option<bool>,
    /// Legal name of the sending instance (originator), shown on the receipt.
    #[serde(default)]
    pub sender_legal_name: Option<String>,
}

/// The response returned by the chain-specific create methods
/// (`create_evm`, `create_solana`, `create_stellar`).
///
/// A trimmed view of a payout: the just-created record plus its initial
/// tracking stages and resolved relation ids.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CreatePayoutResponse {
    /// Payout id (`pa_…`).
    pub id: String,
    /// Overall payout status.
    pub status: TransactionStatus,
    /// Sender wallet address.
    pub sender_wallet_address: String,
    /// Billing fee amount in cents.
    #[serde(default)]
    pub billing_fee_amount: Option<i64>,
    /// Transaction fee amount in cents.
    #[serde(default)]
    pub transaction_fee_amount: Option<i64>,
    /// Partner fee amount in cents (can be negative for rebates).
    #[serde(default)]
    pub partner_fee: Option<i64>,
    /// Completion tracking leg.
    #[serde(default)]
    pub tracking_complete: Option<PayoutTrackingComplete>,
    /// Fiat-payment tracking leg.
    #[serde(default)]
    pub tracking_payment: Option<PayoutTrackingPayment>,
    /// On-chain (token) tracking leg.
    #[serde(default)]
    pub tracking_transaction: Option<PayoutTrackingTransaction>,
    /// Partner-fee tracking leg.
    #[serde(default)]
    pub tracking_partner_fee: Option<PayoutTrackingPartnerFee>,
    /// Liquidity tracking leg.
    #[serde(default)]
    pub tracking_liquidity: Option<PayoutTrackingLiquidity>,
    /// Documents tracking leg.
    #[serde(default)]
    pub tracking_documents: Option<PayoutTrackingDocuments>,
    /// Receiver id (`re_…`).
    #[serde(default)]
    pub receiver_id: Option<String>,
    /// Bank account id (`ba_…`).
    #[serde(default)]
    pub bank_account_id: Option<String>,
    /// Offramp wallet id (`ow_…`).
    #[serde(default)]
    pub offramp_wallet_id: Option<String>,
}

/// Input for `Payouts::create_evm`
/// (`POST /instances/{instance_id}/payouts/evm`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateEvmPayoutInput {
    /// Quote id to consume (`qu_…`).
    pub quote_id: String,
    /// Sender wallet address.
    pub sender_wallet_address: String,
}

impl CreateEvmPayoutInput {
    /// Creates the input from a quote id and sender wallet address.
    pub fn new(quote_id: impl Into<String>, sender_wallet_address: impl Into<String>) -> Self {
        Self {
            quote_id: quote_id.into(),
            sender_wallet_address: sender_wallet_address.into(),
        }
    }
}

/// Input for `Payouts::create_solana`
/// (`POST /instances/{instance_id}/payouts/solana`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateSolanaPayoutInput {
    /// Quote id to consume (`qu_…`).
    pub quote_id: String,
    /// Sender wallet address.
    pub sender_wallet_address: String,
    /// Client-signed transaction, when the flow requires it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_transaction: Option<String>,
}

impl CreateSolanaPayoutInput {
    /// Creates the input from a quote id and sender wallet address.
    pub fn new(quote_id: impl Into<String>, sender_wallet_address: impl Into<String>) -> Self {
        Self {
            quote_id: quote_id.into(),
            sender_wallet_address: sender_wallet_address.into(),
            signed_transaction: None,
        }
    }

    /// Sets the signed transaction.
    pub fn signed_transaction(mut self, signed_transaction: impl Into<String>) -> Self {
        self.signed_transaction = Some(signed_transaction.into());
        self
    }
}

/// Input for `Payouts::create_stellar`
/// (`POST /instances/{instance_id}/payouts/stellar`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateStellarPayoutInput {
    /// Quote id to consume (`qu_…`).
    pub quote_id: String,
    /// Sender wallet address.
    pub sender_wallet_address: String,
    /// Client-signed transaction, when the flow requires it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_transaction: Option<String>,
}

impl CreateStellarPayoutInput {
    /// Creates the input from a quote id and sender wallet address.
    pub fn new(quote_id: impl Into<String>, sender_wallet_address: impl Into<String>) -> Self {
        Self {
            quote_id: quote_id.into(),
            sender_wallet_address: sender_wallet_address.into(),
            signed_transaction: None,
        }
    }

    /// Sets the signed transaction.
    pub fn signed_transaction(mut self, signed_transaction: impl Into<String>) -> Self {
        self.signed_transaction = Some(signed_transaction.into());
        self
    }
}

/// Input for `Payouts::authorize_stellar_token`
/// (`POST /instances/{instance_id}/payouts/stellar/authorize`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorizeStellarTokenInput {
    /// Quote id to consume (`qu_…`).
    pub quote_id: String,
    /// Sender wallet address.
    pub sender_wallet_address: String,
}

impl AuthorizeStellarTokenInput {
    /// Creates the input from a quote id and sender wallet address.
    pub fn new(quote_id: impl Into<String>, sender_wallet_address: impl Into<String>) -> Self {
        Self {
            quote_id: quote_id.into(),
            sender_wallet_address: sender_wallet_address.into(),
        }
    }
}

/// Response from `Payouts::authorize_stellar_token`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct AuthorizeStellarTokenResponse {
    /// The authorization transaction hash.
    pub transaction_hash: String,
}

/// Input for `Payouts::submit_documents`
/// (`POST /instances/{instance_id}/payouts/{id}/documents`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubmitPayoutDocumentsInput {
    /// Document type.
    pub transaction_document_type: TransactionDocumentType,
    /// Document identifier (invoice number, contract id, etc.).
    pub transaction_document_id: String,
    /// URL to the uploaded document.
    pub transaction_document_file: String,
    /// Optional payment description / memo (max 128 chars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response from `Payouts::submit_documents`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct SubmitPayoutDocumentsResponse {
    /// Whether the documents were submitted successfully.
    pub success: bool,
}

/// Query parameters for `Payouts::list`
/// (`GET /instances/{instance_id}/payouts`).
///
/// Combines [`PaginationParams`](crate::common::PaginationParams)-style cursors
/// with payout-specific filters; only set fields are sent on the wire.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListPayoutsParams {
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<crate::common::Limit>,
    /// Number of items to skip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<crate::common::Offset>,
    /// Cursor: return items after this object id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_after: Option<String>,
    /// Cursor: return items before this object id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_before: Option<String>,
    /// Filter by receiver id (`re_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_id: Option<String>,
    /// Filter by customer id (`cus_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    /// Filter by payout status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,
    /// Filter by receiver display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_name: Option<String>,
    /// Filter by bank account id (`ba_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_id: Option<String>,
    /// Filter by country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Filter by payment rail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<Rail>,
    /// Filter by destination network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    /// Filter by token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Token>,
}

impl ListPayoutsParams {
    /// Creates an empty set of list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of items to return.
    pub fn limit(mut self, limit: crate::common::Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the number of items to skip.
    pub fn offset(mut self, offset: crate::common::Offset) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Filters by payout status.
    pub fn status(mut self, status: TransactionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by receiver id.
    pub fn receiver_id(mut self, receiver_id: impl Into<String>) -> Self {
        self.receiver_id = Some(receiver_id.into());
        self
    }

    /// Filters by payment rail.
    pub fn payment_method(mut self, rail: Rail) -> Self {
        self.payment_method = Some(rail);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_document_type_round_trips() {
        assert_eq!(
            serde_json::to_string(&TransactionDocumentType::BillOfLading).unwrap(),
            "\"bill_of_lading\""
        );
        assert_eq!(
            serde_json::from_str::<TransactionDocumentType>("\"invoice\"").unwrap(),
            TransactionDocumentType::Invoice
        );
        assert_eq!(
            serde_json::from_str::<TransactionDocumentType>("\"future\"").unwrap(),
            TransactionDocumentType::Unknown("future".to_string())
        );
    }

    #[test]
    fn tracking_step_handles_pending_review_and_unknown() {
        assert_eq!(
            serde_json::from_str::<PayoutTrackingStep>("\"pending_review\"").unwrap(),
            PayoutTrackingStep::PendingReview
        );
        assert_eq!(
            serde_json::from_str::<PayoutTrackingStep>("\"new_step\"").unwrap(),
            PayoutTrackingStep::Unknown("new_step".to_string())
        );
    }

    #[test]
    fn estimated_time_of_arrival_round_trips() {
        assert_eq!(
            serde_json::to_string(&EstimatedTimeOfArrival::OneBusinessDay).unwrap(),
            "\"1_business_day\""
        );
        assert_eq!(
            serde_json::from_str::<EstimatedTimeOfArrival>("\"5_30_min\"").unwrap(),
            EstimatedTimeOfArrival::FiveToThirtyMin
        );
    }

    #[test]
    fn create_solana_input_skips_unset_signed_transaction() {
        let input = CreateSolanaPayoutInput::new("qu_123", "0xabc");
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "quote_id": "qu_123", "sender_wallet_address": "0xabc" })
        );

        let with_sig = CreateSolanaPayoutInput::new("qu_123", "0xabc").signed_transaction("AAA");
        let json = serde_json::to_value(&with_sig).unwrap();
        assert_eq!(json["signed_transaction"], "AAA");
    }

    #[test]
    fn submit_documents_input_skips_unset_description() {
        let input = SubmitPayoutDocumentsInput {
            transaction_document_type: TransactionDocumentType::Invoice,
            transaction_document_id: "INV-12345".to_string(),
            transaction_document_file: "https://example.com/document.pdf".to_string(),
            description: None,
        };
        let json = serde_json::to_value(&input).unwrap();
        assert!(json.get("description").is_none());
        assert_eq!(json["transaction_document_type"], "invoice");
    }

    #[test]
    fn payout_transaction_status_round_trips() {
        assert_eq!(
            serde_json::to_string(&PayoutTransactionStatus::Found).unwrap(),
            "\"found\""
        );
        assert_eq!(
            serde_json::from_str::<PayoutTransactionStatus>("\"failed\"").unwrap(),
            PayoutTransactionStatus::Failed
        );
        assert_eq!(
            serde_json::from_str::<PayoutTransactionStatus>("\"future\"").unwrap(),
            PayoutTransactionStatus::Unknown("future".to_string())
        );
    }

    #[test]
    fn payout_payment_provider_status_round_trips() {
        assert_eq!(
            serde_json::to_string(&PayoutPaymentProviderStatus::Returned).unwrap(),
            "\"returned\""
        );
        assert_eq!(
            serde_json::from_str::<PayoutPaymentProviderStatus>("\"sent\"").unwrap(),
            PayoutPaymentProviderStatus::Sent
        );
    }

    #[test]
    fn payout_liquidity_provider_status_round_trips() {
        assert_eq!(
            serde_json::to_string(&PayoutLiquidityProviderStatus::Converted).unwrap(),
            "\"converted\""
        );
        assert_eq!(
            serde_json::from_str::<PayoutLiquidityProviderStatus>("\"withdrawn\"").unwrap(),
            PayoutLiquidityProviderStatus::Withdrawn
        );
    }

    #[test]
    fn payout_complete_status_round_trips() {
        assert_eq!(
            serde_json::to_string(&PayoutCompleteStatus::TokensRefunded).unwrap(),
            "\"tokens_refunded\""
        );
        assert_eq!(
            serde_json::from_str::<PayoutCompleteStatus>("\"paid\"").unwrap(),
            PayoutCompleteStatus::Paid
        );
    }

    #[test]
    fn payout_documents_status_round_trips() {
        assert_eq!(
            serde_json::to_string(&PayoutDocumentsStatus::WaitingDocuments).unwrap(),
            "\"waiting_documents\""
        );
        assert_eq!(
            serde_json::from_str::<PayoutDocumentsStatus>("\"compliance_reviewing\"").unwrap(),
            PayoutDocumentsStatus::ComplianceReviewing
        );
    }

    #[test]
    fn provider_name_round_trips() {
        // Vendor display names include spaces and mixed casing; the newtype
        // preserves them verbatim.
        assert_eq!(
            serde_json::to_string(&ProviderName::from("JPMorgan Chase")).unwrap(),
            "\"JPMorgan Chase\""
        );
        assert_eq!(
            serde_json::from_str::<ProviderName>("\"Triple A Technologies\"").unwrap(),
            ProviderName::from("Triple A Technologies")
        );
    }

    #[test]
    fn list_params_serializes_network_and_token() {
        let params = ListPayoutsParams {
            network: Some(Network::Polygon),
            token: Some(Token::Usdc),
            ..ListPayoutsParams::new()
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["network"], "polygon");
        assert_eq!(json["token"], "USDC");
    }

    #[test]
    fn list_params_skip_unset_fields() {
        let params = ListPayoutsParams::new()
            .limit(crate::common::Limit::Fifty)
            .payment_method(Rail::Pix)
            .status(TransactionStatus::Completed);
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "limit": "50",
                "payment_method": "pix",
                "status": "completed"
            })
        );
    }
}
