//! Request and response types for the `customers` resource.

use serde::{Deserialize, Serialize};

use crate::common::{AccountClass, Country, KycStatus, open_enum};

open_enum! {
    /// The KYC/KYB tier applied to a customer.
    pub enum KycType {
        /// Light verification.
        Light => "light",
        /// Standard verification.
        Standard => "standard",
        /// Enhanced verification.
        Enhanced => "enhanced",
    }
}

open_enum! {
    /// The legal structure of a business customer.
    pub enum BusinessType {
        /// Corporation.
        Corporation => "corporation",
        /// Limited liability company.
        Llc => "llc",
        /// Partnership.
        Partnership => "partnership",
        /// Sole proprietorship.
        SoleProprietorship => "sole_proprietorship",
        /// Trust.
        Trust => "trust",
        /// Non-profit organization.
        NonProfit => "non_profit",
    }
}

open_enum! {
    /// The type of document supporting a limit-increase request.
    pub enum SupportingDocumentType {
        /// Individual bank statement.
        IndividualBankStatement => "individual_bank_statement",
        /// Individual tax return.
        IndividualTaxReturn => "individual_tax_return",
        /// Individual proof of income.
        IndividualProofOfIncome => "individual_proof_of_income",
        /// Business bank statement.
        BusinessBankStatement => "business_bank_statement",
        /// Business financial statements.
        BusinessFinancialStatements => "business_financial_statements",
        /// Business tax return.
        BusinessTaxReturn => "business_tax_return",
    }
}

open_enum! {
    /// The status of a limit-increase request.
    pub enum LimitIncreaseStatus {
        /// Under review.
        InReview => "in_review",
        /// Approved.
        Approved => "approved",
        /// Rejected.
        Rejected => "rejected",
    }
}

open_enum! {
    /// The type of an identification document.
    pub enum IdentificationDocument {
        /// Passport.
        Passport => "PASSPORT",
        /// National ID card.
        IdCard => "ID_CARD",
        /// Driver's license.
        Drivers => "DRIVERS",
    }
}

open_enum! {
    /// The type of a proof-of-address document.
    pub enum ProofOfAddressDocType {
        /// Utility bill.
        UtilityBill => "UTILITY_BILL",
        /// Bank statement.
        BankStatement => "BANK_STATEMENT",
        /// Rental agreement.
        RentalAgreement => "RENTAL_AGREEMENT",
        /// Tax document.
        TaxDocument => "TAX_DOCUMENT",
        /// Government correspondence.
        GovernmentCorrespondence => "GOVERNMENT_CORRESPONDENCE",
    }
}

open_enum! {
    /// The tax-identifier type of a business owner. Mandatory when the owner's
    /// country is US.
    pub enum OwnerTaxType {
        /// Social Security Number.
        Ssn => "SSN",
        /// Individual Taxpayer Identification Number.
        Itin => "ITIN",
    }
}

open_enum! {
    /// The role of a business owner.
    pub enum OwnerRole {
        /// Both a beneficial owner and a controlling person.
        BeneficialControlling => "beneficial_controlling",
        /// A beneficial owner.
        BeneficialOwner => "beneficial_owner",
        /// A controlling person.
        ControllingPerson => "controlling_person",
    }
}

/// A beneficial owner of a business customer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Owner {
    /// The owner's role in the business.
    pub role: OwnerRole,
    /// First name.
    pub first_name: String,
    /// Last name.
    pub last_name: String,
    /// Date of birth (ISO 8601).
    pub date_of_birth: String,
    /// Tax identifier.
    pub tax_id: String,
    /// Tax-identifier type (mandatory when the country is US).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<OwnerTaxType>,
    /// First address line.
    pub address_line_1: String,
    /// Second address line.
    #[serde(default)]
    pub address_line_2: Option<String>,
    /// City.
    pub city: String,
    /// State, province, or region.
    pub state_province_region: String,
    /// Country.
    pub country: Country,
    /// Postal code.
    pub postal_code: String,
    /// Country that issued the identification document.
    pub id_doc_country: Country,
    /// Identification document type.
    pub id_doc_type: IdentificationDocument,
    /// URL of the front of the identification document.
    pub id_doc_front_file: String,
    /// URL of the back of the identification document, if applicable.
    #[serde(default)]
    pub id_doc_back_file: Option<String>,
    /// Proof-of-address document type.
    pub proof_of_address_doc_type: ProofOfAddressDocType,
    /// URL of the proof-of-address document, if applicable.
    #[serde(default)]
    pub proof_of_address_doc_file: Option<String>,
    /// Ownership percentage.
    #[serde(default)]
    pub ownership_percentage: Option<i64>,
    /// Job title.
    #[serde(default)]
    pub title: Option<String>,
}

/// A KYC/KYB warning attached to a customer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct KycWarning {
    /// Warning code.
    #[serde(default)]
    pub code: Option<String>,
    /// Human-readable warning message.
    #[serde(default)]
    pub message: Option<String>,
    /// Resolution status.
    #[serde(default)]
    pub resolution_status: Option<String>,
    /// Warning identifier.
    #[serde(default)]
    pub warning_id: Option<String>,
}

/// A fraud warning attached to a customer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct FraudWarning {
    /// Warning identifier.
    #[serde(default)]
    pub id: Option<String>,
    /// Warning name.
    #[serde(default)]
    pub name: Option<String>,
    /// The operation that triggered the warning.
    #[serde(default)]
    pub operation: Option<String>,
    /// The warning score.
    #[serde(default)]
    pub score: Option<i64>,
}

/// Anti-money-laundering match flags for a customer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct AmlHits {
    /// Whether a sanctions match was found.
    pub has_sanction_match: bool,
    /// Whether a politically-exposed-person match was found.
    pub has_pep_match: bool,
    /// Whether a watchlist match was found.
    pub has_watchlist_match: bool,
    /// Whether a crime-list match was found.
    pub has_crimelist_match: bool,
    /// Whether an adverse-media match was found.
    pub has_adversemedia_match: bool,
}

/// A customer's transaction limits.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerLimit {
    /// Per-transaction limit, in cents.
    pub per_transaction: i64,
    /// Daily limit, in cents.
    pub daily: i64,
    /// Monthly limit, in cents.
    pub monthly: i64,
}

/// A customer (individual or business).
///
/// The API returns a single object whose populated fields depend on the
/// customer's [`type`](Customer::type_) and [`kyc_type`](Customer::kyc_type);
/// variant-specific fields are absent for other variants, so they are all
/// optional.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Customer {
    /// Customer identifier (`re_` prefix).
    pub id: String,
    /// Public customer identifier (present on the list response).
    #[serde(default)]
    pub customer_id: Option<String>,
    /// Whether the account is an individual or a business.
    #[serde(rename = "type")]
    pub type_: AccountClass,
    /// The KYC/KYB tier.
    pub kyc_type: KycType,
    /// The current KYC/KYB status.
    pub kyc_status: KycStatus,
    /// Whether the customer has accepted the terms of service.
    #[serde(default)]
    pub is_tos_accepted: Option<bool>,
    /// KYC/KYB warnings.
    #[serde(default)]
    pub kyc_warnings: Option<Vec<KycWarning>>,
    /// Fraud warnings.
    #[serde(default)]
    pub fraud_warnings: Option<Vec<FraudWarning>>,
    /// Email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Tax identifier.
    #[serde(default)]
    pub tax_id: Option<String>,
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
    /// IP address captured at creation.
    #[serde(default)]
    pub ip_address: Option<String>,
    /// Avatar / logo image URL.
    #[serde(default)]
    pub image_url: Option<String>,
    /// Phone number.
    #[serde(default)]
    pub phone_number: Option<String>,
    /// Proof-of-address document type.
    #[serde(default)]
    pub proof_of_address_doc_type: Option<ProofOfAddressDocType>,
    /// Proof-of-address document URL.
    #[serde(default)]
    pub proof_of_address_doc_file: Option<String>,
    /// First name (individual).
    #[serde(default)]
    pub first_name: Option<String>,
    /// Last name (individual).
    #[serde(default)]
    pub last_name: Option<String>,
    /// Date of birth (individual).
    #[serde(default)]
    pub date_of_birth: Option<String>,
    /// Country that issued the identification document (individual).
    #[serde(default)]
    pub id_doc_country: Option<Country>,
    /// Identification document type (individual).
    #[serde(default)]
    pub id_doc_type: Option<IdentificationDocument>,
    /// URL of the front of the identification document (individual).
    #[serde(default)]
    pub id_doc_front_file: Option<String>,
    /// URL of the back of the identification document (individual).
    #[serde(default)]
    pub id_doc_back_file: Option<String>,
    /// Legal name (business).
    #[serde(default)]
    pub legal_name: Option<String>,
    /// Alternate / trade name (business).
    #[serde(default)]
    pub alternate_name: Option<String>,
    /// Formation date (business).
    #[serde(default)]
    pub formation_date: Option<String>,
    /// Website (business).
    #[serde(default)]
    pub website: Option<String>,
    /// Beneficial owners (business).
    #[serde(default)]
    pub owners: Option<Vec<Owner>>,
    /// Incorporation document URL (business).
    #[serde(default)]
    pub incorporation_doc_file: Option<String>,
    /// Proof-of-ownership document URL (business).
    #[serde(default)]
    pub proof_of_ownership_doc_file: Option<String>,
    /// Source-of-funds document type.
    #[serde(default)]
    pub source_of_funds_doc_type: Option<String>,
    /// Source-of-funds document URL.
    #[serde(default)]
    pub source_of_funds_doc_file: Option<String>,
    /// Selfie image URL.
    #[serde(default)]
    pub selfie_file: Option<String>,
    /// Front image of the individual holding their document (enhanced KYC).
    #[serde(default)]
    pub individual_holding_doc_front_file: Option<String>,
    /// Purpose of transactions.
    #[serde(default)]
    pub purpose_of_transactions: Option<String>,
    /// Explanation for the purpose of transactions.
    #[serde(default)]
    pub purpose_of_transactions_explanation: Option<String>,
    /// Whether the customer is a "for benefit of" account.
    #[serde(default)]
    pub is_fbo: Option<bool>,
    /// Account purpose.
    #[serde(default)]
    pub account_purpose: Option<String>,
    /// Free-form account-purpose description (when `account_purpose` is `other`).
    #[serde(default)]
    pub account_purpose_other: Option<String>,
    /// Business legal structure.
    #[serde(default)]
    pub business_type: Option<BusinessType>,
    /// Business description.
    #[serde(default)]
    pub business_description: Option<String>,
    /// Business NAICS industry code.
    #[serde(default)]
    pub business_industry: Option<String>,
    /// Estimated annual revenue band.
    #[serde(default)]
    pub estimated_annual_revenue: Option<String>,
    /// Source of wealth.
    #[serde(default)]
    pub source_of_wealth: Option<String>,
    /// Whether the business is publicly traded.
    #[serde(default)]
    pub publicly_traded: Option<bool>,
    /// Occupation.
    #[serde(default)]
    pub occupation: Option<String>,
    /// Caller-supplied external identifier.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Owning instance identifier.
    #[serde(default)]
    pub instance_id: Option<String>,
    /// Terms-of-service record identifier.
    #[serde(default)]
    pub tos_id: Option<String>,
    /// AiPrise validation key.
    #[serde(default)]
    pub aiprise_validation_key: Option<String>,
    /// Anti-money-laundering screening status.
    #[serde(default)]
    pub aml_status: Option<String>,
    /// Anti-money-laundering match flags.
    #[serde(default)]
    pub aml_hits: Option<AmlHits>,
    /// The customer's transaction limits.
    #[serde(default)]
    pub limit: Option<CustomerLimit>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last-update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Query parameters for listing customers.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListCustomersParams {
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<crate::common::Limit>,
    /// Number of items to skip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<crate::common::Offset>,
    /// Cursor: return items after this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_after: Option<String>,
    /// Cursor: return items before this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_before: Option<String>,
    /// Filter by full name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// Filter by customer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<String>,
    /// Filter by KYC status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<KycStatus>,
    /// Filter by customer ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    /// Filter by an associated bank-account ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_id: Option<String>,
    /// Filter by country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<Country>,
}

impl ListCustomersParams {
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

    /// Filters by full name.
    pub fn full_name(mut self, value: impl Into<String>) -> Self {
        self.full_name = Some(value.into());
        self
    }

    /// Filters by customer name.
    pub fn customer_name(mut self, value: impl Into<String>) -> Self {
        self.customer_name = Some(value.into());
        self
    }

    /// Filters by KYC status.
    pub fn status(mut self, status: KycStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by customer ID.
    pub fn customer_id(mut self, value: impl Into<String>) -> Self {
        self.customer_id = Some(value.into());
        self
    }

    /// Filters by an associated bank-account ID.
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

/// The response returned by the create-customer endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CreateCustomerResponse {
    /// The new customer's primary identifier.
    pub id: String,
    /// The new customer's customer identifier.
    pub customer_id: String,
}

/// Input for [`super::Customers::create_individual_with_standard_kyc`].
///
/// The `type` and `kyc_type` discriminators are injected by the method.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct CreateIndividualWithStandardKycInput {
    /// Country.
    pub country: Country,
    /// Email address.
    pub email: String,
    /// Account purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose: Option<String>,
    /// First address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Date of birth (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    /// Caller-supplied external identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// First name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// URL of the back of the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_back_file: Option<String>,
    /// Country that issued the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_country: Option<Country>,
    /// URL of the front of the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_front_file: Option<String>,
    /// Identification document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_type: Option<IdentificationDocument>,
    /// Avatar image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// Last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// Proof-of-address document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_file: Option<String>,
    /// Proof-of-address document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_type: Option<ProofOfAddressDocType>,
    /// Selfie image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfie_file: Option<String>,
    /// Source-of-funds document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_file: Option<String>,
    /// Source-of-funds document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_type: Option<String>,
    /// Source of wealth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_wealth: Option<String>,
    /// State, province, or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_province_region: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Terms-of-service record identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tos_id: Option<String>,
    /// Free-form account-purpose description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose_other: Option<String>,
    /// Occupation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupation: Option<String>,
    /// Estimated annual revenue band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_annual_revenue: Option<String>,
}

/// Input for [`super::Customers::create_individual_with_enhanced_kyc`].
///
/// The `type` and `kyc_type` discriminators are injected by the method.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct CreateIndividualWithEnhancedKycInput {
    /// Country.
    pub country: Country,
    /// Email address.
    pub email: String,
    /// Account purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose: Option<String>,
    /// First address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Date of birth (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    /// Caller-supplied external identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// First name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// URL of the back of the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_back_file: Option<String>,
    /// Country that issued the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_country: Option<Country>,
    /// URL of the front of the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_front_file: Option<String>,
    /// Identification document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_type: Option<IdentificationDocument>,
    /// Avatar image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// Last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// Proof-of-address document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_file: Option<String>,
    /// Proof-of-address document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_type: Option<ProofOfAddressDocType>,
    /// Purpose of transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_of_transactions: Option<String>,
    /// Explanation for the purpose of transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_of_transactions_explanation: Option<String>,
    /// Selfie image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfie_file: Option<String>,
    /// Source-of-funds document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_file: Option<String>,
    /// Source-of-funds document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_type: Option<String>,
    /// Source of wealth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_wealth: Option<String>,
    /// State, province, or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_province_region: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Terms-of-service record identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tos_id: Option<String>,
    /// Free-form account-purpose description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose_other: Option<String>,
    /// Occupation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupation: Option<String>,
    /// Estimated annual revenue band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_annual_revenue: Option<String>,
}

/// Input for [`super::Customers::create_business_with_standard_kyb`].
///
/// The `type` and `kyc_type` discriminators are injected by the method.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct CreateBusinessWithStandardKybInput {
    /// Country.
    pub country: Country,
    /// Email address.
    pub email: String,
    /// Account purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose: Option<String>,
    /// First address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// Alternate / trade name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_name: Option<String>,
    /// Business description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_description: Option<String>,
    /// Business NAICS industry code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_industry: Option<String>,
    /// Business legal structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_type: Option<BusinessType>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Estimated annual revenue band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_annual_revenue: Option<String>,
    /// Caller-supplied external identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Formation date (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formation_date: Option<String>,
    /// Logo image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Incorporation document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incorporation_doc_file: Option<String>,
    /// IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// Legal name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,
    /// Beneficial owners.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<Owner>>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// Proof-of-address document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_file: Option<String>,
    /// Proof-of-address document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_type: Option<ProofOfAddressDocType>,
    /// Proof-of-ownership document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_ownership_doc_file: Option<String>,
    /// Whether the business is publicly traded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicly_traded: Option<bool>,
    /// Source-of-funds document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_file: Option<String>,
    /// Source-of-funds document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_type: Option<String>,
    /// Source of wealth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_wealth: Option<String>,
    /// State, province, or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_province_region: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Terms-of-service record identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tos_id: Option<String>,
    /// Website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Free-form account-purpose description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose_other: Option<String>,
    /// Occupation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupation: Option<String>,
}

/// Input for [`super::Customers::update`]; every field is optional.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct UpdateCustomerInput {
    /// Account purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose: Option<String>,
    /// First address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// Alternate / trade name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_name: Option<String>,
    /// Business description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_description: Option<String>,
    /// Business NAICS industry code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_industry: Option<String>,
    /// Business legal structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_type: Option<BusinessType>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<Country>,
    /// Date of birth (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Estimated annual revenue band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_annual_revenue: Option<String>,
    /// Caller-supplied external identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// First name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Formation date (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formation_date: Option<String>,
    /// URL of the back of the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_back_file: Option<String>,
    /// Country that issued the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_country: Option<Country>,
    /// URL of the front of the identification document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_front_file: Option<String>,
    /// Identification document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_doc_type: Option<IdentificationDocument>,
    /// Image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Incorporation document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incorporation_doc_file: Option<String>,
    /// IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// Last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Legal name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,
    /// Beneficial owners.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<Owner>>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// Proof-of-address document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_file: Option<String>,
    /// Proof-of-address document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_address_doc_type: Option<ProofOfAddressDocType>,
    /// Proof-of-ownership document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_of_ownership_doc_file: Option<String>,
    /// Whether the business is publicly traded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicly_traded: Option<bool>,
    /// Purpose of transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_of_transactions: Option<String>,
    /// Explanation for the purpose of transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_of_transactions_explanation: Option<String>,
    /// Selfie image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfie_file: Option<String>,
    /// Source-of-funds document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_file: Option<String>,
    /// Source-of-funds document type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_funds_doc_type: Option<String>,
    /// Source of wealth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_of_wealth: Option<String>,
    /// State, province, or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_province_region: Option<String>,
    /// Tax identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// Terms-of-service record identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tos_id: Option<String>,
    /// Website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Free-form account-purpose description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_purpose_other: Option<String>,
    /// Occupation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupation: Option<String>,
}

/// One side (payin or payout) of a customer's aggregate limits.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LimitWindow {
    /// Daily limit, in cents.
    pub daily: i64,
    /// Monthly limit, in cents.
    pub monthly: i64,
}

/// A customer's payin and payout limits.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerLimits {
    /// Payin limits.
    pub payin: LimitWindow,
    /// Payout limits.
    pub payout: LimitWindow,
}

/// The response returned by [`super::Customers::get_limits`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct GetCustomerLimitsResponse {
    /// The customer's limits.
    pub limits: CustomerLimits,
}

/// A limit-increase request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LimitIncrease {
    /// Request identifier.
    pub id: String,
    /// The receiver the request belongs to.
    pub receiver_id: String,
    /// The current request status.
    pub status: LimitIncreaseStatus,
    /// Requested per-transaction limit, in cents.
    #[serde(default)]
    pub per_transaction: Option<i64>,
    /// Requested daily limit, in cents.
    #[serde(default)]
    pub daily: Option<i64>,
    /// Requested monthly limit, in cents.
    #[serde(default)]
    pub monthly: Option<i64>,
    /// Approved per-transaction limit, in cents.
    #[serde(default)]
    pub approved_per_transaction: Option<i64>,
    /// Approved daily limit, in cents.
    #[serde(default)]
    pub approved_daily: Option<i64>,
    /// Approved monthly limit, in cents.
    #[serde(default)]
    pub approved_monthly: Option<i64>,
    /// The supporting document's type.
    #[serde(default)]
    pub supporting_document_type: Option<SupportingDocumentType>,
    /// URL of the supporting document.
    #[serde(default)]
    pub supporting_document_file: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last-update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Input for [`super::Customers::request_limit_increase`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestLimitIncreaseInput {
    /// Requested per-transaction limit, in cents.
    pub per_transaction: i64,
    /// Requested daily limit, in cents.
    pub daily: i64,
    /// Requested monthly limit, in cents.
    pub monthly: i64,
    /// The supporting document's type.
    pub supporting_document_type: SupportingDocumentType,
    /// URL of the supporting document.
    pub supporting_document_file: String,
}

/// The response returned by [`super::Customers::request_limit_increase`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RequestLimitIncreaseResponse {
    /// The new limit-increase request's identifier.
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kyc_type_round_trips() {
        assert_eq!(
            serde_json::to_string(&KycType::Enhanced).unwrap(),
            "\"enhanced\""
        );
        assert_eq!(
            serde_json::from_str::<KycType>("\"light\"").unwrap(),
            KycType::Light
        );
        assert_eq!(
            serde_json::from_str::<KycType>("\"future\"").unwrap(),
            KycType::Unknown("future".to_string())
        );
    }

    #[test]
    fn business_type_round_trips() {
        assert_eq!(
            serde_json::to_string(&BusinessType::SoleProprietorship).unwrap(),
            "\"sole_proprietorship\""
        );
        assert_eq!(
            serde_json::from_str::<BusinessType>("\"non_profit\"").unwrap(),
            BusinessType::NonProfit
        );
    }

    #[test]
    fn supporting_document_type_round_trips() {
        assert_eq!(
            serde_json::to_string(&SupportingDocumentType::BusinessTaxReturn).unwrap(),
            "\"business_tax_return\""
        );
    }

    #[test]
    fn limit_increase_status_round_trips() {
        assert_eq!(
            serde_json::from_str::<LimitIncreaseStatus>("\"in_review\"").unwrap(),
            LimitIncreaseStatus::InReview
        );
    }

    #[test]
    fn identification_document_uses_uppercase_wire() {
        assert_eq!(
            serde_json::to_string(&IdentificationDocument::IdCard).unwrap(),
            "\"ID_CARD\""
        );
        assert_eq!(
            serde_json::from_str::<ProofOfAddressDocType>("\"UTILITY_BILL\"").unwrap(),
            ProofOfAddressDocType::UtilityBill
        );
    }

    #[test]
    fn owner_tax_type_uses_uppercase_wire() {
        assert_eq!(
            serde_json::to_string(&OwnerTaxType::Ssn).unwrap(),
            "\"SSN\""
        );
        assert_eq!(
            serde_json::from_str::<OwnerTaxType>("\"ITIN\"").unwrap(),
            OwnerTaxType::Itin
        );
    }

    #[test]
    fn create_individual_input_injects_no_discriminators_and_skips_none() {
        let input = CreateIndividualWithStandardKycInput {
            country: Country::from("BR"),
            email: "a@b.com".to_string(),
            first_name: Some("Ana".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json["country"], "BR");
        assert_eq!(json["email"], "a@b.com");
        assert_eq!(json["first_name"], "Ana");
        // Unset optionals are omitted; discriminators are added by the method.
        assert!(json.get("last_name").is_none());
        assert!(json.get("type").is_none());
        assert!(json.get("kyc_type").is_none());
    }
}
