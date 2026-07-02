//! The `bank-accounts` sub-resource: payout destinations nested under a customer.
//!
//! Accessed via `customers.bank_accounts`. Bank-account creation is a
//! discriminated union keyed by the rail; this is exposed as one constructor per
//! rail (`create_pix`, `create_wire`, …), each injecting its `type` internally.

use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Inner;
use crate::common::{AccountClass, AccountType, Country, Rail, Success, TransfersType, open_enum};
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::resources::wallets::OfframpWallet;

/// A SWIFT payment-purpose code (the API's `swiftCode` enum).
///
/// The API defines this as a large, evolving set of ~2500 values, so it is
/// modeled as a transparent newtype rather than an enumerated type — strongly
/// typed (never a bare `String`) yet forward-compatible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct SwiftPaymentCode(String);

impl SwiftPaymentCode {
    /// Returns the code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SwiftPaymentCode {
    fn from(value: &str) -> Self {
        SwiftPaymentCode(value.to_string())
    }
}

impl From<String> for SwiftPaymentCode {
    fn from(value: String) -> Self {
        SwiftPaymentCode(value)
    }
}

impl AsRef<str> for SwiftPaymentCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SwiftPaymentCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

open_enum! {
    /// The verification status of a bank account.
    pub enum BankAccountStatus {
        /// Verification in progress.
        Verifying => "verifying",
        /// Approved.
        Approved => "approved",
        /// Rejected.
        Rejected => "rejected",
        /// Deprecated / superseded.
        Deprecated => "deprecated",
    }
}

open_enum! {
    /// The beneficiary's relationship to the account holder.
    pub enum RecipientRelationship {
        /// The beneficiary is the account holder themselves.
        FirstParty => "first_party",
        /// Employee.
        Employee => "employee",
        /// Independent contractor.
        IndependentContractor => "independent_contractor",
        /// Vendor or supplier.
        VendorOrSupplier => "vendor_or_supplier",
        /// Subsidiary or affiliate.
        SubsidiaryOrAffiliate => "subsidiary_or_affiliate",
        /// Merchant or partner.
        MerchantOrPartner => "merchant_or_partner",
        /// Customer.
        Customer => "customer",
        /// Landlord.
        Landlord => "landlord",
        /// Family member.
        Family => "family",
        /// Other relationship.
        Other => "other",
    }
}

open_enum! {
    /// The SPEI account identifier protocol (Mexico).
    pub enum SpeiProtocol {
        /// CLABE account number.
        Clabe => "clabe",
        /// Debit-card number.
        Debitcard => "debitcard",
        /// Phone number.
        Phonenum => "phonenum",
    }
}

open_enum! {
    /// The Colombian ACH beneficiary document type. Wire values are uppercase.
    pub enum AchCopDocument {
        /// Cédula de Ciudadanía.
        Cc => "CC",
        /// Cédula de Extranjería.
        Ce => "CE",
        /// Número de Identificación Tributaria.
        Nit => "NIT",
        /// Passport.
        Pass => "PASS",
        /// Permiso Especial de Permanencia.
        Pep => "PEP",
    }
}

/// A bank account (payout destination).
///
/// Populated fields depend on the account's [`type`](BankAccount::type_) (rail);
/// fields for other rails are absent, so all rail-specific fields are optional.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BankAccount {
    /// Bank-account identifier (`ba_` prefix).
    pub id: String,
    /// The payment rail.
    #[serde(rename = "type")]
    pub type_: Rail,
    /// Display name for the account.
    pub name: String,
    /// PIX key (PIX).
    #[serde(default)]
    pub pix_key: Option<String>,
    /// Beneficiary name.
    #[serde(default)]
    pub beneficiary_name: Option<String>,
    /// Routing number (ACH/wire/RTP).
    #[serde(default)]
    pub routing_number: Option<String>,
    /// Account number.
    #[serde(default)]
    pub account_number: Option<String>,
    /// Account type (checking/saving).
    #[serde(default)]
    pub account_type: Option<AccountType>,
    /// Account class (individual/business).
    #[serde(default)]
    pub account_class: Option<AccountClass>,
    /// First address line.
    #[serde(default)]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(default)]
    pub address_line_2: Option<String>,
    /// City.
    #[serde(default)]
    pub city: Option<String>,
    /// State, province, or region.
    #[serde(default)]
    pub state_province_region: Option<String>,
    /// Country.
    #[serde(default)]
    pub country: Option<Country>,
    /// Postal code.
    #[serde(default)]
    pub postal_code: Option<String>,
    /// SPEI protocol (Mexico).
    #[serde(default)]
    pub spei_protocol: Option<SpeiProtocol>,
    /// SPEI institution code (Mexico).
    #[serde(default)]
    pub spei_institution_code: Option<String>,
    /// SPEI CLABE (Mexico).
    #[serde(default)]
    pub spei_clabe: Option<String>,
    /// Transfers 3.0 type (Argentina).
    #[serde(default)]
    pub transfers_type: Option<TransfersType>,
    /// Transfers 3.0 account (Argentina).
    #[serde(default)]
    pub transfers_account: Option<String>,
    /// Colombian ACH beneficiary first name.
    #[serde(default)]
    pub ach_cop_beneficiary_first_name: Option<String>,
    /// Colombian ACH beneficiary last name.
    #[serde(default)]
    pub ach_cop_beneficiary_last_name: Option<String>,
    /// Colombian ACH document ID.
    #[serde(default)]
    pub ach_cop_document_id: Option<String>,
    /// Colombian ACH document type.
    #[serde(default)]
    pub ach_cop_document_type: Option<AchCopDocument>,
    /// Colombian ACH beneficiary email.
    #[serde(default)]
    pub ach_cop_email: Option<String>,
    /// Colombian ACH bank code.
    #[serde(default)]
    pub ach_cop_bank_code: Option<String>,
    /// Colombian ACH bank account.
    #[serde(default)]
    pub ach_cop_bank_account: Option<String>,
    /// SWIFT/BIC code (international SWIFT).
    #[serde(default)]
    pub swift_code_bic: Option<String>,
    /// SWIFT account holder name.
    #[serde(default)]
    pub swift_account_holder_name: Option<String>,
    /// SWIFT account number / IBAN.
    #[serde(default)]
    pub swift_account_number_iban: Option<String>,
    /// SWIFT beneficiary first address line.
    #[serde(default)]
    pub swift_beneficiary_address_line_1: Option<String>,
    /// SWIFT beneficiary second address line.
    #[serde(default)]
    pub swift_beneficiary_address_line_2: Option<String>,
    /// SWIFT beneficiary country.
    #[serde(default)]
    pub swift_beneficiary_country: Option<Country>,
    /// SWIFT beneficiary city.
    #[serde(default)]
    pub swift_beneficiary_city: Option<String>,
    /// SWIFT beneficiary state / province / region.
    #[serde(default)]
    pub swift_beneficiary_state_province_region: Option<String>,
    /// SWIFT beneficiary postal code.
    #[serde(default)]
    pub swift_beneficiary_postal_code: Option<String>,
    /// SWIFT bank name.
    #[serde(default)]
    pub swift_bank_name: Option<String>,
    /// SWIFT bank first address line.
    #[serde(default)]
    pub swift_bank_address_line_1: Option<String>,
    /// SWIFT bank second address line.
    #[serde(default)]
    pub swift_bank_address_line_2: Option<String>,
    /// SWIFT bank country.
    #[serde(default)]
    pub swift_bank_country: Option<Country>,
    /// SWIFT bank city.
    #[serde(default)]
    pub swift_bank_city: Option<String>,
    /// SWIFT bank state / province / region.
    #[serde(default)]
    pub swift_bank_state_province_region: Option<String>,
    /// SWIFT bank postal code.
    #[serde(default)]
    pub swift_bank_postal_code: Option<String>,
    /// SWIFT intermediary bank SWIFT/BIC code.
    #[serde(default)]
    pub swift_intermediary_bank_swift_code_bic: Option<String>,
    /// SWIFT intermediary bank account number / IBAN.
    #[serde(default)]
    pub swift_intermediary_bank_account_number_iban: Option<String>,
    /// SWIFT intermediary bank name.
    #[serde(default)]
    pub swift_intermediary_bank_name: Option<String>,
    /// SWIFT intermediary bank country.
    #[serde(default)]
    pub swift_intermediary_bank_country: Option<Country>,
    /// SEPA IBAN.
    #[serde(default)]
    pub sepa_iban: Option<String>,
    /// SEPA beneficiary BIC.
    #[serde(default)]
    pub sepa_beneficiary_bic: Option<String>,
    /// SEPA beneficiary legal name.
    #[serde(default)]
    pub sepa_beneficiary_legal_name: Option<String>,
    /// SEPA beneficiary first address line.
    #[serde(default)]
    pub sepa_beneficiary_address_line_1: Option<String>,
    /// SEPA beneficiary second address line.
    #[serde(default)]
    pub sepa_beneficiary_address_line_2: Option<String>,
    /// SEPA beneficiary city.
    #[serde(default)]
    pub sepa_beneficiary_city: Option<String>,
    /// SEPA beneficiary state / province / region.
    #[serde(default)]
    pub sepa_beneficiary_state_province_region: Option<String>,
    /// SEPA beneficiary postal code.
    #[serde(default)]
    pub sepa_beneficiary_postal_code: Option<String>,
    /// SEPA beneficiary country.
    #[serde(default)]
    pub sepa_beneficiary_country: Option<Country>,
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
    /// Tron wallet hash, when an off-ramp wallet is provisioned.
    #[serde(default)]
    pub tron_wallet_hash: Option<String>,
    /// Business NAICS industry code.
    #[serde(default)]
    pub business_industry: Option<String>,
    /// Phone number.
    #[serde(default)]
    pub phone_number: Option<String>,
    /// Tax identifier.
    #[serde(default)]
    pub tax_id: Option<String>,
    /// Date of birth.
    #[serde(default)]
    pub date_of_birth: Option<String>,
    /// SWIFT payment code.
    #[serde(default)]
    pub swift_payment_code: Option<SwiftPaymentCode>,
    /// SWIFT IFSC branch code (required when the SWIFT bank country is `IN`).
    #[serde(default)]
    pub swift_ifsc_branch_code: Option<String>,
    /// The verification status.
    #[serde(default)]
    pub status: Option<BankAccountStatus>,
    /// The beneficiary's relationship to the account holder.
    #[serde(default)]
    pub recipient_relationship: Option<RecipientRelationship>,
    /// Off-ramp wallets provisioned for this account.
    #[serde(default)]
    pub offramp_wallets: Option<Vec<OfframpWallet>>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Query parameters for listing bank accounts.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListBankAccountsParams {
    /// Filter by verification status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BankAccountStatus>,
    /// Filter by rail.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<Rail>,
    /// Filter by name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Filter by bank-account ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_id: Option<String>,
    /// Filter by country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<Country>,
}

impl ListBankAccountsParams {
    /// Creates an empty set of list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by verification status.
    pub fn status(mut self, status: BankAccountStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by rail.
    pub fn rail(mut self, rail: Rail) -> Self {
        self.type_ = Some(rail);
        self
    }

    /// Filters by name.
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Filters by bank-account ID.
    pub fn bank_account_id(mut self, value: impl Into<String>) -> Self {
        self.bank_account_id = Some(value.into());
        self
    }

    /// Filters by country.
    pub fn country(mut self, country: impl Into<Country>) -> Self {
        self.country = Some(country.into());
        self
    }
}

/// Input for [`BankAccounts::create_pix`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatePixInput {
    /// Display name for the account.
    pub name: String,
    /// PIX key.
    pub pix_key: String,
    /// Whether to force CPF/CNPJ validation of the PIX key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_cpf_cnpj: Option<bool>,
}

/// Input for [`BankAccounts::create_pix_safe`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatePixSafeInput {
    /// Display name for the account.
    pub name: String,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Account number.
    pub account_number: String,
    /// Account type.
    pub account_type: AccountType,
    /// PIX Safe bank code.
    pub pix_safe_bank_code: String,
    /// PIX Safe branch code.
    pub pix_safe_branch_code: String,
    /// PIX Safe CPF/CNPJ.
    pub pix_safe_cpf_cnpj: String,
}

/// Input for [`BankAccounts::create_ted`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateTedInput {
    /// Display name for the account.
    pub name: String,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Account number.
    pub account_number: String,
    /// Account type.
    pub account_type: AccountType,
    /// TED bank code.
    pub ted_bank_code: String,
    /// TED branch code.
    pub ted_branch_code: String,
    /// TED CPF/CNPJ.
    pub ted_cpf_cnpj: String,
}

/// Input for [`BankAccounts::create_spei_bitso`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateSpeiBitsoInput {
    /// Display name for the account.
    pub name: String,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// SPEI protocol.
    pub spei_protocol: SpeiProtocol,
    /// SPEI CLABE / card / phone number.
    pub spei_clabe: String,
    /// SPEI institution code (required for `debitcard`/`phonenum`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spei_institution_code: Option<String>,
}

/// Input for [`BankAccounts::create_transfers_bitso`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateTransfersBitsoInput {
    /// Display name for the account.
    pub name: String,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Transfers 3.0 type.
    pub transfers_type: TransfersType,
    /// Transfers 3.0 account.
    pub transfers_account: String,
}

/// Input for [`BankAccounts::create_ach_cop_bitso`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateAchCopBitsoInput {
    /// Display name for the account.
    pub name: String,
    /// Account type.
    pub account_type: AccountType,
    /// Beneficiary first name.
    pub ach_cop_beneficiary_first_name: String,
    /// Beneficiary last name.
    pub ach_cop_beneficiary_last_name: String,
    /// Beneficiary document ID.
    pub ach_cop_document_id: String,
    /// Beneficiary document type.
    pub ach_cop_document_type: AchCopDocument,
    /// Beneficiary email.
    pub ach_cop_email: String,
    /// Bank code.
    pub ach_cop_bank_code: String,
    /// Bank account.
    pub ach_cop_bank_account: String,
}

/// Input for [`BankAccounts::create_ach`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateAchInput {
    /// Display name for the account.
    pub name: String,
    /// Account class.
    pub account_class: AccountClass,
    /// Account number.
    pub account_number: String,
    /// Account type.
    pub account_type: AccountType,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Routing number.
    pub routing_number: String,
    /// The beneficiary's relationship to the account holder.
    pub recipient_relationship: RecipientRelationship,
    /// First address line.
    pub address_line_1: String,
    /// City.
    pub city: String,
    /// State, province, or region.
    pub state_province_region: String,
    /// Country.
    pub country: Country,
    /// Postal code.
    pub postal_code: String,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// Business NAICS industry code (required for business accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_industry: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Date of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
}

/// Input for [`BankAccounts::create_wire`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateWireInput {
    /// Display name for the account.
    pub name: String,
    /// Account class.
    pub account_class: AccountClass,
    /// Account number.
    pub account_number: String,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Routing number.
    pub routing_number: String,
    /// The beneficiary's relationship to the account holder.
    pub recipient_relationship: RecipientRelationship,
    /// First address line.
    pub address_line_1: String,
    /// City.
    pub city: String,
    /// State, province, or region.
    pub state_province_region: String,
    /// Country.
    pub country: Country,
    /// Postal code.
    pub postal_code: String,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// Business NAICS industry code (required for business accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_industry: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Date of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
}

/// Input for [`BankAccounts::create_rtp`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateRtpInput {
    /// Display name for the account.
    pub name: String,
    /// Account class.
    pub account_class: AccountClass,
    /// Beneficiary name.
    pub beneficiary_name: String,
    /// Routing number.
    pub routing_number: String,
    /// Account number.
    pub account_number: String,
    /// The beneficiary's relationship to the account holder.
    pub recipient_relationship: RecipientRelationship,
    /// First address line.
    pub address_line_1: String,
    /// City.
    pub city: String,
    /// State, province, or region.
    pub state_province_region: String,
    /// Country.
    pub country: Country,
    /// Postal code.
    pub postal_code: String,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// Business NAICS industry code (required for business accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_industry: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Date of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
}

/// Input for [`BankAccounts::create_international_swift`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateInternationalSwiftInput {
    /// Display name for the account.
    pub name: String,
    /// Account class.
    pub account_class: AccountClass,
    /// The beneficiary's relationship to the account holder.
    pub recipient_relationship: RecipientRelationship,
    /// SWIFT account holder name.
    pub swift_account_holder_name: String,
    /// SWIFT account number / IBAN.
    pub swift_account_number_iban: String,
    /// SWIFT/BIC code.
    pub swift_code_bic: String,
    /// SWIFT beneficiary first address line.
    pub swift_beneficiary_address_line_1: String,
    /// SWIFT beneficiary city.
    pub swift_beneficiary_city: String,
    /// SWIFT beneficiary country.
    pub swift_beneficiary_country: Country,
    /// SWIFT beneficiary postal code.
    pub swift_beneficiary_postal_code: String,
    /// SWIFT beneficiary state / province / region.
    pub swift_beneficiary_state_province_region: String,
    /// SWIFT bank name.
    pub swift_bank_name: String,
    /// SWIFT bank first address line.
    pub swift_bank_address_line_1: String,
    /// SWIFT bank city.
    pub swift_bank_city: String,
    /// SWIFT bank country.
    pub swift_bank_country: Country,
    /// SWIFT bank postal code.
    pub swift_bank_postal_code: String,
    /// SWIFT bank state / province / region.
    pub swift_bank_state_province_region: String,
    /// SWIFT beneficiary second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_beneficiary_address_line_2: Option<String>,
    /// SWIFT bank second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_bank_address_line_2: Option<String>,
    /// SWIFT intermediary bank SWIFT/BIC code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_intermediary_bank_swift_code_bic: Option<String>,
    /// SWIFT intermediary bank account number / IBAN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_intermediary_bank_account_number_iban: Option<String>,
    /// SWIFT intermediary bank name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_intermediary_bank_name: Option<String>,
    /// SWIFT intermediary bank country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_intermediary_bank_country: Option<Country>,
    /// SWIFT payment code (required for some destination countries).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_payment_code: Option<SwiftPaymentCode>,
    /// SWIFT IFSC branch code (required when the SWIFT bank country is `IN`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swift_ifsc_branch_code: Option<String>,
    /// Business NAICS industry code (required for business accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_industry: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Date of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
}

/// Input for [`BankAccounts::create_sepa`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateSepaInput {
    /// Display name for the account.
    pub name: String,
    /// Account class.
    pub account_class: AccountClass,
    /// SEPA IBAN.
    pub sepa_iban: String,
    /// SEPA beneficiary BIC.
    pub sepa_beneficiary_bic: String,
    /// SEPA beneficiary legal name.
    pub sepa_beneficiary_legal_name: String,
    /// SEPA beneficiary first address line.
    pub sepa_beneficiary_address_line_1: String,
    /// SEPA beneficiary city.
    pub sepa_beneficiary_city: String,
    /// SEPA beneficiary postal code.
    pub sepa_beneficiary_postal_code: String,
    /// SEPA beneficiary country.
    pub sepa_beneficiary_country: Country,
    /// SEPA beneficiary second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sepa_beneficiary_address_line_2: Option<String>,
    /// SEPA beneficiary state / province / region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sepa_beneficiary_state_province_region: Option<String>,
}

/// Handle for the `bank-accounts` sub-resource (`customers.bank_accounts`).
///
#[derive(Clone)]
pub struct BankAccounts {
    client: Arc<Inner>,
}

impl BankAccounts {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists the bank accounts of a customer.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn list(
        &self,
        customer_id: impl AsRef<str>,
        params: &ListBankAccountsParams,
    ) -> Result<Vec<BankAccount>> {
        let path = self.collection_path(customer_id);
        self.client.get_query(&path, params).await
    }

    /// Retrieves a single bank account by ID.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/bank-accounts/{id}`
    pub async fn get(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<BankAccount> {
        let path = format!(
            "{}/{}",
            self.collection_path(customer_id),
            encode_path_segment(id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Deletes a bank account.
    ///
    /// `DELETE /instances/{instance_id}/customers/{customer_id}/bank-accounts/{id}`
    pub async fn delete(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<Success> {
        let path = format!(
            "{}/{}",
            self.collection_path(customer_id),
            encode_path_segment(id.as_ref())
        );
        self.client
            .request(Method::DELETE, &path, None::<&()>, None::<&()>)
            .await
    }

    /// Creates a PIX bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_pix(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreatePixInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "pix", input).await
    }

    /// Creates a PIX Safe bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_pix_safe(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreatePixSafeInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "pix_safe", input).await
    }

    /// Creates a TED bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_ted(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateTedInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "ted", input).await
    }

    /// Creates a SPEI (via Bitso) bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_spei_bitso(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateSpeiBitsoInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "spei_bitso", input).await
    }

    /// Creates a Transfers 3.0 (via Bitso) bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_transfers_bitso(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateTransfersBitsoInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "transfers_bitso", input).await
    }

    /// Creates a Colombian ACH (via Bitso) bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_ach_cop_bitso(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateAchCopBitsoInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "ach_cop_bitso", input).await
    }

    /// Creates an ACH bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_ach(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateAchInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "ach", input).await
    }

    /// Creates a domestic-wire bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_wire(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateWireInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "wire", input).await
    }

    /// Creates an RTP bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_rtp(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateRtpInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "rtp", input).await
    }

    /// Creates an international SWIFT bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_international_swift(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateInternationalSwiftInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "international_swift", input).await
    }

    /// Creates a SEPA bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts`
    pub async fn create_sepa(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateSepaInput,
    ) -> Result<BankAccount> {
        self.create(customer_id, "sepa", input).await
    }

    async fn create<T: Serialize>(
        &self,
        customer_id: impl AsRef<str>,
        rail: &'static str,
        input: &T,
    ) -> Result<BankAccount> {
        let body = Discriminated { type_: rail, input };
        let path = self.collection_path(customer_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(&body))
            .await
    }

    fn collection_path(&self, customer_id: impl AsRef<str>) -> String {
        format!(
            "/instances/{}/customers/{}/bank-accounts",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        )
    }
}

/// Wraps a per-rail create input with the `type` discriminator the API's
/// discriminated union expects, flattening the input alongside it.
#[derive(Serialize)]
struct Discriminated<'a, T: Serialize> {
    #[serde(rename = "type")]
    type_: &'static str,
    #[serde(flatten)]
    input: &'a T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfers_type_uses_uppercase_wire() {
        assert_eq!(
            serde_json::to_string(&TransfersType::Cvu).unwrap(),
            "\"CVU\""
        );
        assert_eq!(
            serde_json::from_str::<TransfersType>("\"ALIAS\"").unwrap(),
            TransfersType::Alias
        );
    }

    #[test]
    fn ach_cop_document_uses_uppercase_wire() {
        assert_eq!(
            serde_json::to_string(&AchCopDocument::Nit).unwrap(),
            "\"NIT\""
        );
        assert_eq!(
            serde_json::from_str::<AchCopDocument>("\"CC\"").unwrap(),
            AchCopDocument::Cc
        );
    }

    #[test]
    fn spei_protocol_round_trips() {
        assert_eq!(
            serde_json::to_string(&SpeiProtocol::Clabe).unwrap(),
            "\"clabe\""
        );
    }

    #[test]
    fn bank_account_status_round_trips() {
        assert_eq!(
            serde_json::from_str::<BankAccountStatus>("\"verifying\"").unwrap(),
            BankAccountStatus::Verifying
        );
    }

    #[test]
    fn recipient_relationship_round_trips() {
        assert_eq!(
            serde_json::to_string(&RecipientRelationship::VendorOrSupplier).unwrap(),
            "\"vendor_or_supplier\""
        );
    }

    #[test]
    fn swift_payment_code_serde_round_trips() {
        let code = SwiftPaymentCode::from("GOODS");
        assert_eq!(serde_json::to_string(&code).unwrap(), "\"GOODS\"");
        assert_eq!(
            serde_json::from_str::<SwiftPaymentCode>("\"SERVICES\"").unwrap(),
            SwiftPaymentCode::from("SERVICES")
        );
        assert_eq!(code.as_str(), "GOODS");
    }

    #[test]
    fn discriminated_create_body_injects_type_and_flattens_input() {
        let input = CreatePixInput {
            name: "My PIX".to_string(),
            pix_key: "11122233344".to_string(),
            force_cpf_cnpj: None,
        };
        let body = Discriminated {
            type_: "pix",
            input: &input,
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["type"], "pix");
        assert_eq!(json["name"], "My PIX");
        assert_eq!(json["pix_key"], "11122233344");
        assert!(json.get("force_cpf_cnpj").is_none());
    }

    #[test]
    fn list_params_serialize_type_as_type_key() {
        let params = ListBankAccountsParams::new()
            .rail(Rail::Pix)
            .status(BankAccountStatus::Approved);
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["type"], "pix");
        assert_eq!(json["status"], "approved");
        assert!(json.get("name").is_none());
    }
}
