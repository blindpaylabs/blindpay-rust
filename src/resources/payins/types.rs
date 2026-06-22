//! Request and response types for the `payins` resource and its `payin-quotes`
//! sub-resource.

use serde::{Deserialize, Serialize};

use crate::common::{
    AccountClass, Currency, CurrencyType, Network, PaymentMethod, Token, TransactionStatus,
    open_enum,
};

open_enum! {
    /// The account-identifier type in a [`TransfersInstruction`] (Transfers 3.0).
    pub enum TransfersType {
        /// Clave Virtual Uniforme (CVU).
        Cvu => "CVU",
        /// Clave Bancaria Uniforme (CBU).
        Cbu => "CBU",
        /// Human-readable account alias.
        Alias => "ALIAS",
    }
}

open_enum! {
    /// The payer document type for PSE (Colombian) payins.
    pub enum PseDocumentType {
        /// Cédula de Ciudadanía.
        Cc => "CC",
        /// Número de Identificación Tributaria.
        Nit => "NIT",
    }
}

open_enum! {
    /// The lifecycle step shared by a payin's tracking legs.
    pub enum TrackingStatus {
        /// The leg is in progress.
        Processing => "processing",
        /// The leg is paused (e.g. awaiting review).
        OnHold => "on_hold",
        /// The leg is awaiting manual review.
        PendingReview => "pending_review",
        /// The leg has finished.
        Completed => "completed",
    }
}

open_enum! {
    /// The sub-status of a payin's inbound fiat-leg transaction.
    pub enum PayinTransactionStatus {
        /// The transaction failed.
        Failed => "failed",
        /// The transaction completed.
        Completed => "completed",
    }
}

/// The name of the payment provider that processed a payin leg.
///
/// The API defines this as a large, evolving set of provider names, so it is
/// modeled as a transparent newtype rather than an enumerated type — strongly
/// typed (never a bare `String`) yet forward-compatible with new providers.
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

/// Payer-restriction rules attached to a payin quote.
///
/// Constrains which payers (by tax id, name, document, etc.) may fund the
/// resulting payin. All fields are optional; only the ones relevant to the
/// chosen payment method apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PayerRules {
    /// Allowed PIX payer tax ids (CPF/CNPJ).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pix_allowed_tax_ids: Option<Vec<String>>,
    /// Allowed Transfers 3.0 payer tax id (CUIT/CUIL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfers_allowed_tax_id: Option<String>,
    /// Allowed PSE payer tax ids.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_allowed_tax_ids: Option<Vec<String>>,
    /// PSE payer full name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_full_name: Option<String>,
    /// PSE payer document type (`CC` or `NIT`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_document_type: Option<PseDocumentType>,
    /// PSE payer document number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_document_number: Option<String>,
    /// PSE payer email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_email: Option<String>,
    /// PSE payer phone (format `+573` followed by 9 digits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_phone: Option<String>,
    /// PSE payer bank code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pse_bank_code: Option<String>,
}

/// Input for `PayinQuotes::create` (`POST /instances/{instance_id}/payin-quotes`).
///
/// Provide exactly one wallet reference — `blockchain_wallet_id` (a `bw_…` id) or
/// `wallet_id` (a custodial `bl_…` id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatePayinQuoteInput {
    /// Blockchain wallet to credit (`bw_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain_wallet_id: Option<String>,
    /// Custodial wallet to credit (`bl_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<String>,
    /// Which side the `request_amount` is denominated in.
    pub currency_type: CurrencyType,
    /// If `true`, the sender covers the fees; otherwise the receiver does.
    pub cover_fees: bool,
    /// Requested amount in cents (minimum 500).
    pub request_amount: i64,
    /// Payin payment method.
    pub payment_method: PaymentMethod,
    /// Stablecoin token to receive.
    pub token: Token,
    /// Optional partner fee to apply (`pf_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee_id: Option<String>,
    /// Optional payer-restriction rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer_rules: Option<PayerRules>,
    /// Whether this quote is for an OTC (over-the-counter) transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_otc: Option<bool>,
}

impl Default for CreatePayinQuoteInput {
    fn default() -> Self {
        Self {
            blockchain_wallet_id: None,
            wallet_id: None,
            currency_type: CurrencyType::Sender,
            cover_fees: false,
            request_amount: 0,
            payment_method: PaymentMethod::Pix,
            token: Token::Usdc,
            partner_fee_id: None,
            payer_rules: None,
            is_otc: None,
        }
    }
}

/// A payin quote, returned by `PayinQuotes::create`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayinQuote {
    /// Quote id.
    pub id: String,
    /// Epoch (seconds) after which the quote expires.
    pub expires_at: i64,
    /// Commercial quotation (e.g. `495` = 1 USD = 4.95 BRL).
    pub commercial_quotation: i64,
    /// BlindPay quotation (commercial quotation plus BlindPay fees).
    pub blindpay_quotation: i64,
    /// Amount the receiver gets, in cents.
    pub receiver_amount: i64,
    /// Amount the sender must send, in cents.
    pub sender_amount: i64,
    /// Partner fee amount in cents, if any.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// Flat fee in cents.
    pub flat_fee: i64,
    /// Billing fee in cents, if any.
    #[serde(default)]
    pub billing_fee_amount: Option<i64>,
    /// Whether the quote is for an OTC transaction.
    #[serde(default)]
    pub is_otc: Option<bool>,
}

/// Input for `PayinQuotes::get_fx_rate`
/// (`POST /instances/{instance_id}/payin-quotes/fx`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PayinQuoteFxInput {
    /// Source currency.
    pub from: Currency,
    /// Target stablecoin token.
    pub to: Token,
    /// Requested amount in cents.
    pub request_amount: i64,
    /// Which side the `request_amount` is denominated in.
    pub currency_type: CurrencyType,
}

/// An FX-rate preview for a payin, returned by `PayinQuotes::get_fx_rate`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayinQuoteFx {
    /// Commercial quotation.
    pub commercial_quotation: i64,
    /// BlindPay quotation.
    pub blindpay_quotation: i64,
    /// Resulting amount in cents.
    pub result_amount: i64,
    /// Instance flat fee in cents.
    pub instance_flat_fee: i64,
    /// Instance percentage fee in basis points.
    pub instance_percentage_fee: i64,
}

/// Input for `Payins::create_evm` (`POST /instances/{instance_id}/payins/evm`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatePayinInput {
    /// The payin quote to execute (`pq_…`).
    pub payin_quote_id: String,
}

/// PSE payment instruction embedded in a payin's transaction tracking.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PseInstruction {
    /// PSE payment link URL.
    pub payment_link: String,
    /// PSE funding identifier (for webhook matching).
    pub fid: String,
    /// PSE payer full name.
    pub full_name: String,
    /// PSE payer document number.
    pub tax_id: String,
    /// PSE payer document type (`CC` or `NIT`).
    pub document_type: PseDocumentType,
    /// PSE payer phone number.
    pub phone: String,
    /// PSE payer email.
    pub email: String,
    /// PSE payer bank code.
    #[serde(default)]
    pub bank_code: Option<String>,
}

/// Transfers 3.0 payment instruction embedded in a payin's transaction tracking.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TransfersInstruction {
    /// CVU/CBU/Alias account identifier.
    pub account: String,
    /// Account type (`CVU`, `CBU`, or `ALIAS`).
    #[serde(rename = "type")]
    pub transfers_type: TransfersType,
    /// Payer CUIT/CUIL (digits only), if known.
    #[serde(default)]
    pub tax_id: Option<String>,
}

/// TED payment instruction embedded in a payin's transaction tracking.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TedInstruction {
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Beneficiary tax id.
    pub tax_id: String,
    /// Bank code.
    pub bank_code: String,
    /// Bank name.
    pub bank_name: String,
    /// Branch code.
    pub branch_code: String,
    /// Account number.
    pub account_number: String,
    /// Account type.
    pub account_type: String,
}

/// The transaction step of a payin (inbound fiat leg).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayinTrackingTransaction {
    /// The tracking step.
    pub step: TrackingStatus,
    /// Sub-status, when applicable.
    #[serde(default)]
    pub status: Option<PayinTransactionStatus>,
    /// External transaction id.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Completion timestamp.
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Blockchain transaction hash.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// Payment provider name.
    #[serde(default)]
    pub provider_name: Option<ProviderName>,
    /// Payment provider transaction id.
    #[serde(default)]
    pub provider_transaction_id: Option<String>,
    /// Sender name.
    #[serde(default)]
    pub sender_name: Option<String>,
    /// Sender bank name.
    #[serde(default)]
    pub sender_bank_name: Option<String>,
    /// Sender tax id.
    #[serde(default)]
    pub sender_tax_id: Option<String>,
    /// Sender bank code.
    #[serde(default)]
    pub sender_bank_code: Option<String>,
    /// Sender account number.
    #[serde(default)]
    pub sender_account_number: Option<String>,
    /// BACEN PIX end-to-end transaction id.
    #[serde(default)]
    pub end_to_end_id: Option<String>,
    /// ACH trace number.
    #[serde(default)]
    pub trace_number: Option<String>,
    /// Transaction reference.
    #[serde(default)]
    pub transaction_reference: Option<String>,
    /// Free-text description.
    #[serde(default)]
    pub description: Option<String>,
    /// PSE payment instruction, when the rail is PSE.
    #[serde(default)]
    pub pse_instruction: Option<PseInstruction>,
    /// Transfers 3.0 payment instruction, when the rail is Transfers.
    #[serde(default)]
    pub transfers_instruction: Option<TransfersInstruction>,
    /// TED payment instruction, when the rail is TED.
    #[serde(default)]
    pub ted_instruction: Option<TedInstruction>,
}

/// The payment step of a payin (BlindPay processing leg).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayinTrackingPayment {
    /// The tracking step.
    pub step: TrackingStatus,
    /// Payment provider name.
    #[serde(default)]
    pub provider_name: Option<String>,
    /// Completion timestamp.
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// The completion step of a payin (on-chain settlement leg).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayinTrackingComplete {
    /// The tracking step.
    pub step: TrackingStatus,
    /// Blockchain transaction hash.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// Completion timestamp.
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// The partner-fee settlement step of a payin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PayinTrackingPartnerFee {
    /// The tracking step.
    pub step: TrackingStatus,
    /// Blockchain transaction hash.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// Completion timestamp.
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// BlindPay deposit instructions returned for a payin (where the sender pays).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BlindpayBankAccount {
    /// Routing number.
    pub routing_number: String,
    /// Account number.
    pub account_number: String,
}

/// A beneficiary or receiving-bank block within [`BlindpayBankDetails`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BlindpayBankParty {
    /// Party name.
    pub name: String,
    /// First address line.
    pub address_line_1: String,
    /// Second address line.
    pub address_line_2: String,
}

/// SWIFT receiving-bank block (nullable fields) within [`BlindpayBankDetails`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct SwiftReceivingBank {
    /// Bank name.
    #[serde(default)]
    pub name: Option<String>,
    /// First address line.
    #[serde(default)]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(default)]
    pub address_line_2: Option<String>,
}

/// BlindPay bank details a sender uses to fund a payin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BlindpayBankDetails {
    /// Routing number.
    pub routing_number: String,
    /// Account number.
    pub account_number: String,
    /// Account type label.
    pub account_type: String,
    /// SWIFT/BIC code.
    #[serde(default)]
    pub swift_bic_code: Option<String>,
    /// ACH account block.
    #[serde(default)]
    pub ach: Option<BlindpayBankAccount>,
    /// Wire account block.
    #[serde(default)]
    pub wire: Option<BlindpayBankAccount>,
    /// RTP account block.
    #[serde(default)]
    pub rtp: Option<BlindpayBankAccount>,
    /// Beneficiary block.
    pub beneficiary: BlindpayBankParty,
    /// Receiving-bank block.
    pub receiving_bank: BlindpayBankParty,
    /// SWIFT account number.
    #[serde(default)]
    pub swift_account_number: Option<String>,
    /// SWIFT receiving bank.
    #[serde(default)]
    pub swift_receiving_bank: Option<SwiftReceivingBank>,
}

/// A payin, returned by `Payins::get`, `Payins::list`, and `Payins::get_track`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Payin {
    /// Payin id (`pi_…`).
    pub id: String,
    /// The receiver/customer this payin belongs to (`re_…`).
    pub receiver_id: String,
    /// PIX copy-and-paste code, when the rail is PIX.
    #[serde(default)]
    pub pix_code: Option<String>,
    /// Memo code, when applicable.
    #[serde(default)]
    pub memo_code: Option<String>,
    /// CLABE, when the rail is SPEI.
    #[serde(default)]
    pub clabe: Option<String>,
    /// Overall payin status.
    pub status: TransactionStatus,
    /// The quote this payin executed (`pq_…`).
    pub payin_quote_id: String,
    /// Owning instance id (`in_…`).
    pub instance_id: String,
    /// Inbound fiat-leg tracking.
    pub tracking_transaction: PayinTrackingTransaction,
    /// Processing-leg tracking.
    pub tracking_payment: PayinTrackingPayment,
    /// Settlement-leg tracking.
    pub tracking_complete: PayinTrackingComplete,
    /// Partner-fee settlement tracking, when applicable.
    #[serde(default)]
    pub tracking_partner_fee: Option<PayinTrackingPartnerFee>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last-update timestamp.
    pub updated_at: String,
    /// Receiver avatar URL.
    #[serde(default)]
    pub image_url: Option<String>,
    /// Receiver first name (individual).
    #[serde(default)]
    pub first_name: Option<String>,
    /// Receiver last name (individual).
    #[serde(default)]
    pub last_name: Option<String>,
    /// Receiver legal name (business).
    #[serde(default)]
    pub legal_name: Option<String>,
    /// Whether the receiver is an individual or a business.
    #[serde(rename = "type")]
    pub account_class: AccountClass,
    /// Payment method of the underlying quote.
    pub payment_method: PaymentMethod,
    /// Amount the sender sends, in cents.
    pub sender_amount: i64,
    /// Amount the receiver gets, in cents.
    pub receiver_amount: i64,
    /// Stablecoin token received.
    pub token: Token,
    /// Applied partner fee id (`pf_…`), if any.
    #[serde(default)]
    pub partner_fee_id: Option<String>,
    /// Partner fee amount in cents, if any.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// Total fee amount in cents, if any.
    #[serde(default)]
    pub total_fee_amount: Option<i64>,
    /// Commercial quotation.
    pub commercial_quotation: i64,
    /// BlindPay quotation.
    pub blindpay_quotation: i64,
    /// Quote currency.
    pub currency: Currency,
    /// Billing fee amount in cents, if any.
    #[serde(default)]
    pub billing_fee_amount: Option<i64>,
    /// Transaction fee amount in cents, if any.
    #[serde(default)]
    pub transaction_fee_amount: Option<i64>,
    /// Whether the underlying quote is OTC.
    #[serde(default)]
    pub is_otc: Option<bool>,
    /// Payer rules applied to the quote, if any.
    #[serde(default)]
    pub payer_rules: Option<PayerRules>,
    // `name` and `network` stay optional: `get_track` decodes this same type from
    // the unauthenticated `/e/` endpoint, which can null these fields.
    /// Receiver display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Destination wallet address.
    #[serde(default)]
    pub address: Option<String>,
    /// Destination wallet network.
    #[serde(default)]
    pub network: Option<Network>,
    /// BlindPay deposit instructions, when the sender funds the payin.
    #[serde(default)]
    pub blindpay_bank_details: Option<BlindpayBankDetails>,
    /// PSE payment link.
    #[serde(default)]
    pub pse_payment_link: Option<String>,
    /// PSE payer full name.
    #[serde(default)]
    pub pse_full_name: Option<String>,
    /// PSE payer document number.
    #[serde(default)]
    pub pse_tax_id: Option<String>,
    /// PSE payer document type (`CC` or `NIT`).
    #[serde(default)]
    pub pse_document_type: Option<PseDocumentType>,
}

/// The response of `Payins::create_evm`
/// (`POST /instances/{instance_id}/payins/evm`).
///
/// A narrower projection of a payin than [`Payin`], carrying the deposit
/// instructions and tracking needed right after creation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CreatePayinResponse {
    /// Payin id (`pi_…`).
    pub id: String,
    /// Overall payin status.
    pub status: TransactionStatus,
    /// PIX copy-and-paste code, when the rail is PIX.
    #[serde(default)]
    pub pix_code: Option<String>,
    /// Memo code, when applicable.
    #[serde(default)]
    pub memo_code: Option<String>,
    /// CLABE, when the rail is SPEI.
    #[serde(default)]
    pub clabe: Option<String>,
    /// Partner fee amount in cents.
    #[serde(default)]
    pub partner_fee: Option<i64>,
    /// Settlement-leg tracking.
    pub tracking_complete: PayinTrackingComplete,
    /// Processing-leg tracking.
    pub tracking_payment: PayinTrackingPayment,
    /// Inbound fiat-leg tracking.
    pub tracking_transaction: PayinTrackingTransaction,
    /// Partner-fee settlement tracking, when applicable.
    #[serde(default)]
    pub tracking_partner_fee: Option<PayinTrackingPartnerFee>,
    /// Billing fee amount in cents, if any.
    #[serde(default)]
    pub billing_fee_amount: Option<i64>,
    /// Transaction fee amount in cents, if any.
    #[serde(default)]
    pub transaction_fee_amount: Option<i64>,
    /// BlindPay deposit instructions.
    pub blindpay_bank_details: BlindpayBankDetails,
    /// The receiver/customer this payin belongs to (`re_…`).
    #[serde(default)]
    pub receiver_id: Option<String>,
    /// Amount the receiver gets, in cents.
    #[serde(default)]
    pub receiver_amount: Option<i64>,
    /// Payment method of the underlying quote.
    #[serde(default)]
    pub payment_method: Option<PaymentMethod>,
    /// Amount the sender sends, in cents.
    #[serde(default)]
    pub sender_amount: Option<i64>,
}

/// Query parameters for `Payins::list` (`GET /instances/{instance_id}/payins`).
///
/// Combines [`PaginationParams`](crate::common::PaginationParams)-style cursors
/// with payin-specific filters; only set fields are sent on the wire.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListPayinsParams {
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
    /// Filter by payin status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,
    /// Filter by customer id (`re_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    /// Filter by receiver display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_name: Option<String>,
    /// Filter by bank account id (`ba_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_id: Option<String>,
    /// Filter by country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Filter by payment method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<PaymentMethod>,
    /// Filter by destination network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    /// Filter by token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Token>,
}

impl ListPayinsParams {
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

    /// Filters by payin status.
    pub fn status(mut self, status: TransactionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by customer id.
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.customer_id = Some(customer_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_payin_quote_input_skips_unset_optional_fields() {
        let input = CreatePayinQuoteInput {
            blockchain_wallet_id: Some("bw_123".to_string()),
            currency_type: CurrencyType::Sender,
            cover_fees: true,
            request_amount: 1000,
            payment_method: PaymentMethod::Pix,
            token: Token::Usdc,
            ..Default::default()
        };
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "blockchain_wallet_id": "bw_123",
                "currency_type": "sender",
                "cover_fees": true,
                "request_amount": 1000,
                "payment_method": "pix",
                "token": "USDC"
            })
        );
    }

    #[test]
    fn payer_rules_round_trips_only_set_fields() {
        let rules = PayerRules {
            pix_allowed_tax_ids: Some(vec!["123.456.789-09".to_string()]),
            ..Default::default()
        };
        let json = serde_json::to_value(&rules).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "pix_allowed_tax_ids": ["123.456.789-09"] })
        );
    }

    #[test]
    fn list_params_serialize_pagination_and_filters() {
        let params = ListPayinsParams::new()
            .limit(crate::common::Limit::Fifty)
            .status(TransactionStatus::Completed)
            .customer_id("re_123");
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "limit": "50",
                "status": "completed",
                "customer_id": "re_123"
            })
        );
    }

    #[test]
    fn transfers_type_serde_round_trips() {
        for (variant, wire) in [
            (TransfersType::Cvu, "CVU"),
            (TransfersType::Cbu, "CBU"),
            (TransfersType::Alias, "ALIAS"),
        ] {
            assert_eq!(serde_json::to_value(&variant).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<TransfersType>(json!(wire)).unwrap(),
                variant
            );
        }
        assert_eq!(
            serde_json::from_value::<TransfersType>(json!("PVU")).unwrap(),
            TransfersType::Unknown("PVU".to_string())
        );
    }

    #[test]
    fn pse_document_type_serde_round_trips() {
        for (variant, wire) in [(PseDocumentType::Cc, "CC"), (PseDocumentType::Nit, "NIT")] {
            assert_eq!(serde_json::to_value(&variant).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<PseDocumentType>(json!(wire)).unwrap(),
                variant
            );
        }
        assert_eq!(
            serde_json::from_value::<PseDocumentType>(json!("CE")).unwrap(),
            PseDocumentType::Unknown("CE".to_string())
        );
    }

    #[test]
    fn tracking_status_serde_round_trips() {
        for (variant, wire) in [
            (TrackingStatus::Processing, "processing"),
            (TrackingStatus::OnHold, "on_hold"),
            (TrackingStatus::PendingReview, "pending_review"),
            (TrackingStatus::Completed, "completed"),
        ] {
            assert_eq!(serde_json::to_value(&variant).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<TrackingStatus>(json!(wire)).unwrap(),
                variant
            );
        }
        assert_eq!(
            serde_json::from_value::<TrackingStatus>(json!("refunded")).unwrap(),
            TrackingStatus::Unknown("refunded".to_string())
        );
    }

    #[test]
    fn payin_transaction_status_serde_round_trips() {
        for (variant, wire) in [
            (PayinTransactionStatus::Failed, "failed"),
            (PayinTransactionStatus::Completed, "completed"),
        ] {
            assert_eq!(serde_json::to_value(&variant).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<PayinTransactionStatus>(json!(wire)).unwrap(),
                variant
            );
        }
        assert_eq!(
            serde_json::from_value::<PayinTransactionStatus>(json!("pending")).unwrap(),
            PayinTransactionStatus::Unknown("pending".to_string())
        );
    }

    #[test]
    fn provider_name_serde_round_trips() {
        let provider: ProviderName = serde_json::from_value(json!("HSBC")).unwrap();
        assert_eq!(provider, ProviderName::from("HSBC"));
        assert_eq!(provider.as_str(), "HSBC");
        assert_eq!(serde_json::to_value(&provider).unwrap(), json!("HSBC"));
    }

    #[test]
    fn payin_deserializes_with_type_renamed_to_account_class() {
        let payin: Payin = serde_json::from_value(serde_json::json!({
            "id": "pi_123",
            "receiver_id": "re_123",
            "status": "processing",
            "payin_quote_id": "pq_123",
            "instance_id": "in_test",
            "tracking_transaction": { "step": "processing", "external_id": null },
            "tracking_payment": { "step": "processing" },
            "tracking_complete": { "step": "processing" },
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-01T00:00:00.000Z",
            "type": "individual",
            "payment_method": "pix",
            "sender_amount": 5240,
            "receiver_amount": 1010,
            "token": "USDC",
            "commercial_quotation": 495,
            "blindpay_quotation": 505,
            "currency": "BRL"
        }))
        .unwrap();
        assert_eq!(payin.id, "pi_123");
        assert_eq!(payin.account_class, AccountClass::Individual);
        assert_eq!(payin.payment_method, PaymentMethod::Pix);
        assert_eq!(payin.token, Token::Usdc);
        assert!(payin.partner_fee_id.is_none());
    }
}
