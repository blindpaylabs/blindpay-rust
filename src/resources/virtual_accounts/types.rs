//! Request and response types for the `virtual_accounts` resource.

use serde::{Deserialize, Serialize};

use crate::common::{KycStatus, Network, Token, open_enum};

open_enum! {
    /// The banking partner backing a virtual account.
    pub enum BankingPartner {
        /// JPMorgan Chase.
        Jpmorgan => "jpmorgan",
        /// Citi.
        Citi => "citi",
        /// HSBC.
        Hsbc => "hsbc",
        /// Cross River / CFSB.
        Cfsb => "cfsb",
    }
}

open_enum! {
    /// The document type for a sole-proprietor virtual account.
    pub enum SoleProprietorDocType {
        /// Master service agreement.
        MasterServiceAgreement => "master_service_agreement",
        /// Salary slip.
        SalarySlip => "salary_slip",
        /// Bank statement.
        BankStatement => "bank_statement",
    }
}

/// US domestic bank coordinates (routing + account number) for one rail of a
/// virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct UsBankCoordinates {
    /// The bank routing number.
    pub routing_number: String,
    /// The account number.
    pub account_number: String,
}

/// A beneficiary or bank address block within the US banking details.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct UsAddress {
    /// Name of the beneficiary or bank. May be absent.
    #[serde(default)]
    pub name: Option<String>,
    /// First address line. May be absent.
    #[serde(default)]
    pub address_line_1: Option<String>,
    /// Second address line. May be absent.
    #[serde(default)]
    pub address_line_2: Option<String>,
}

/// SWIFT intermediary bank details within the US banking details.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct UsSwiftIntermediaryBank {
    /// Intermediary bank name. May be absent.
    #[serde(default)]
    pub name: Option<String>,
    /// Intermediary bank SWIFT/BIC code. May be absent.
    #[serde(default)]
    pub swift_code_bic: Option<String>,
    /// Intermediary bank routing number. May be absent.
    #[serde(default)]
    pub routing_number: Option<String>,
    /// First address line. May be absent.
    #[serde(default)]
    pub address_line_1: Option<String>,
    /// Second address line. May be absent.
    #[serde(default)]
    pub address_line_2: Option<String>,
}

/// The nested US banking-details object on a [`VirtualAccount`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct UsVirtualAccountDetails {
    /// ACH coordinates. May be absent.
    #[serde(default)]
    pub ach: Option<UsBankCoordinates>,
    /// Domestic wire coordinates.
    pub wire: UsBankCoordinates,
    /// Real-Time Payments coordinates. May be absent.
    #[serde(default)]
    pub rtp: Option<UsBankCoordinates>,
    /// SWIFT/BIC code for international wires. May be absent.
    #[serde(default)]
    pub swift_bic_code: Option<String>,
    /// SWIFT account number (IBAN-style) for international wires. May be absent.
    #[serde(default)]
    pub swift_account_number: Option<String>,
    /// Account type display label, e.g. `"Personal checking"` or
    /// `"Business checking"`. Kept as a free string rather than an enum because
    /// the API exposes these as human-readable labels. May be absent.
    #[serde(default)]
    pub account_type: Option<String>,
    /// Beneficiary details. May be absent.
    #[serde(default)]
    pub beneficiary: Option<UsAddress>,
    /// Domestic receiving-bank details. May be absent.
    #[serde(default)]
    pub receiving_bank: Option<UsAddress>,
    /// SWIFT receiving-bank details. May be absent.
    #[serde(default)]
    pub swift_receiving_bank: Option<UsAddress>,
    /// SWIFT intermediary-bank details. May be absent.
    #[serde(default)]
    pub swift_intermediary_bank: Option<UsSwiftIntermediaryBank>,
}

/// The Bre-B key of a Colombian virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CoBreBDetails {
    /// The Bre-B key payers send to, e.g. `"@BP000000000000"`.
    pub key: String,
}

/// The SPEI coordinates of a Mexican virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct MxSpeiDetails {
    /// The 18-digit CLABE payers send to.
    pub clabe: String,
}

/// The beneficiary shown to payers of a non-US virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct VirtualAccountBeneficiary {
    /// Beneficiary name.
    pub name: String,
}

/// The nested Colombian (Bre-B) banking-details object on a [`VirtualAccount`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CoVirtualAccountDetails {
    /// Bre-B coordinates.
    pub bre_b: CoBreBDetails,
    /// Beneficiary details. May be `null`.
    #[serde(default)]
    pub beneficiary: Option<VirtualAccountBeneficiary>,
}

/// The nested Mexican (SPEI) banking-details object on a [`VirtualAccount`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct MxVirtualAccountDetails {
    /// SPEI coordinates.
    pub spei: MxSpeiDetails,
    /// Beneficiary details. May be `null`.
    #[serde(default)]
    pub beneficiary: Option<VirtualAccountBeneficiary>,
}

/// The blockchain wallet a virtual account settles to, as embedded in the
/// virtual-account response.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct VirtualAccountWallet {
    /// The wallet's blockchain network.
    pub network: Network,
    /// The on-chain wallet address. May be `null`.
    #[serde(default)]
    pub address: Option<String>,
}

/// A virtual account, returned by the `virtual_accounts` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct VirtualAccount {
    /// The virtual account identifier (`va_…`).
    pub id: String,
    /// The banking partner backing the account. May be `null`.
    #[serde(default)]
    pub banking_partner: Option<BankingPartner>,
    /// The KYC/KYB status of the owning customer. May be `null`.
    #[serde(default)]
    pub kyc_status: Option<KycStatus>,
    /// The US banking details (ACH/wire/RTP/SWIFT). `None` for non-US accounts.
    #[serde(default)]
    pub us: Option<UsVirtualAccountDetails>,
    /// The Colombian (Bre-B) banking details. `None` for non-CO accounts.
    #[serde(default)]
    pub co: Option<CoVirtualAccountDetails>,
    /// The Mexican (SPEI) banking details. `None` for non-MX accounts.
    #[serde(default)]
    pub mx: Option<MxVirtualAccountDetails>,
    /// The stablecoin token funds are converted to.
    pub token: Token,
    /// The blockchain wallet identifier (`bw_…`) funds settle to. May be absent.
    #[serde(default)]
    pub blockchain_wallet_id: Option<String>,
    /// The embedded blockchain wallet. Absent or `null` when no wallet is linked.
    #[serde(default)]
    pub blockchain_wallet: Option<VirtualAccountWallet>,
    /// The owning customer (`re_…`).
    #[serde(default)]
    pub customer_id: Option<String>,
    /// Partner fee applied to deposits (`fe_…`). May be `null`.
    #[serde(default)]
    pub partner_fee_id: Option<String>,
}

open_enum! {
    /// Account-holder gender, required by BRE-B virtual accounts.
    pub enum BreBGender {
        /// Male.
        Male => "male",
        /// Female.
        Female => "female",
    }
}

/// BRE-B details for a `bre_b` virtual account (spec `CreateVirtualAccountIn.bre_b`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct BreBInput {
    /// Account-holder gender.
    pub gender: BreBGender,
}

impl BreBInput {
    /// Creates the BRE-B details.
    pub fn new(gender: BreBGender) -> Self {
        Self { gender }
    }
}

/// Body for creating a virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateVirtualAccountInput {
    /// The banking partner to provision the account with.
    pub banking_partner: BankingPartner,
    /// The stablecoin token funds are converted to.
    pub token: Token,
    /// The blockchain wallet identifier (`bw_…`) funds settle to.
    pub blockchain_wallet_id: String,
    /// Document type for a sole-proprietor account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sole_proprietor_doc_type: Option<SoleProprietorDocType>,
    /// URL of the uploaded sole-proprietor document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sole_proprietor_doc_file: Option<String>,
    /// Partner fee to apply to deposits (`fe_…`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee_id: Option<String>,
    /// BRE-B details, required for `bre_b` virtual accounts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bre_b: Option<BreBInput>,
}

impl CreateVirtualAccountInput {
    /// Creates a new input with the required fields; optional fields default to
    /// `None`.
    pub fn new(
        banking_partner: BankingPartner,
        token: Token,
        blockchain_wallet_id: impl Into<String>,
    ) -> Self {
        Self {
            banking_partner,
            token,
            blockchain_wallet_id: blockchain_wallet_id.into(),
            sole_proprietor_doc_type: None,
            sole_proprietor_doc_file: None,
            partner_fee_id: None,
            bre_b: None,
        }
    }

    /// Sets `partner_fee_id`.
    pub fn partner_fee_id(mut self, partner_fee_id: impl Into<String>) -> Self {
        self.partner_fee_id = Some(partner_fee_id.into());
        self
    }

    /// Sets `bre_b`.
    pub fn bre_b(mut self, bre_b: BreBInput) -> Self {
        self.bre_b = Some(bre_b);
        self
    }
}

/// Body for updating a virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateVirtualAccountInput {
    /// The stablecoin token funds are converted to.
    pub token: Token,
    /// The blockchain wallet identifier (`bw_…`) funds settle to.
    pub blockchain_wallet_id: String,
    /// Partner fee to apply to deposits (`fe_…`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee_id: Option<String>,
}

impl UpdateVirtualAccountInput {
    /// Creates a new update input from the two required fields.
    pub fn new(token: Token, blockchain_wallet_id: impl Into<String>) -> Self {
        Self {
            token,
            blockchain_wallet_id: blockchain_wallet_id.into(),
            partner_fee_id: None,
        }
    }

    /// Sets `partner_fee_id`.
    pub fn partner_fee_id(mut self, partner_fee_id: impl Into<String>) -> Self {
        self.partner_fee_id = Some(partner_fee_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banking_partner_round_trips() {
        assert_eq!(
            serde_json::to_string(&BankingPartner::Jpmorgan).unwrap(),
            "\"jpmorgan\""
        );
        assert_eq!(
            serde_json::from_str::<BankingPartner>("\"cfsb\"").unwrap(),
            BankingPartner::Cfsb
        );
        assert_eq!(
            serde_json::from_str::<BankingPartner>("\"new_bank\"").unwrap(),
            BankingPartner::Unknown("new_bank".to_string())
        );
    }

    #[test]
    fn sole_proprietor_doc_type_round_trips() {
        assert_eq!(
            serde_json::to_string(&SoleProprietorDocType::MasterServiceAgreement).unwrap(),
            "\"master_service_agreement\""
        );
        assert_eq!(
            serde_json::from_str::<SoleProprietorDocType>("\"bank_statement\"").unwrap(),
            SoleProprietorDocType::BankStatement
        );
    }

    #[test]
    fn create_input_skips_absent_optional_fields() {
        let input = CreateVirtualAccountInput::new(BankingPartner::Jpmorgan, Token::Usdc, "bw_123");
        let value = serde_json::to_value(&input).unwrap();
        assert_eq!(value["banking_partner"], "jpmorgan");
        assert_eq!(value["token"], "USDC");
        assert_eq!(value["blockchain_wallet_id"], "bw_123");
        assert!(value.get("sole_proprietor_doc_type").is_none());
        assert!(value.get("sole_proprietor_doc_file").is_none());
    }

    #[test]
    fn virtual_account_parses_full_us_block_and_null_wallet() {
        let json = serde_json::json!({
            "id": "va_123",
            "banking_partner": "jpmorgan",
            "kyc_status": "approved",
            "us": {
                "ach": { "routing_number": "021000021", "account_number": "111111111" },
                "wire": { "routing_number": "021000021", "account_number": "222222222" },
                "rtp": null,
                "swift_bic_code": "TCCLGB3L",
                "account_type": "Business checking",
                "beneficiary": {
                    "name": "Test Co",
                    "address_line_1": "8 The Green",
                    "address_line_2": "Dover, DE 19901"
                },
                "swift_intermediary_bank": {
                    "name": "JP Morgan Chase NA",
                    "swift_code_bic": "CHASUS33"
                }
            },
            "token": "USDC",
            "blockchain_wallet_id": "bw_123",
            "blockchain_wallet": null
        });

        let va: VirtualAccount = serde_json::from_value(json).unwrap();
        assert_eq!(va.id, "va_123");
        assert_eq!(va.banking_partner, Some(BankingPartner::Jpmorgan));
        assert_eq!(va.kyc_status, Some(KycStatus::Approved));
        assert!(va.co.is_none() && va.mx.is_none());
        let us = va.us.unwrap();
        assert_eq!(us.wire.account_number, "222222222");
        assert!(us.ach.is_some());
        assert!(us.rtp.is_none());
        assert_eq!(us.swift_bic_code.as_deref(), Some("TCCLGB3L"));
        assert_eq!(
            us.swift_intermediary_bank
                .unwrap()
                .swift_code_bic
                .as_deref(),
            Some("CHASUS33")
        );
        assert!(va.blockchain_wallet.is_none());
    }

    // The shared spec fixtures can't express a non-US account (their `full`
    // variant fills every block), so these two are hand-written from the
    // spec's `VirtualAccountOut` examples.
    #[test]
    fn virtual_account_parses_colombian_bre_b_account_without_us_block() {
        let va: VirtualAccount = serde_json::from_value(serde_json::json!({
            "id": "va_000000000000",
            "banking_partner": "bre_b",
            "kyc_status": "approved",
            "us": null,
            "co": {
                "bre_b": { "key": "@BP000000000000" },
                "beneficiary": { "name": "Customer Name" }
            },
            "token": "USDC",
            "blockchain_wallet_id": "bw_000000000000",
            "blockchain_wallet": { "network": "polygon", "address": null }
        }))
        .unwrap();
        assert!(va.us.is_none());
        assert_eq!(
            va.banking_partner,
            Some(BankingPartner::Unknown("bre_b".to_string()))
        );
        let co = va.co.unwrap();
        assert_eq!(co.bre_b.key, "@BP000000000000");
        assert_eq!(co.beneficiary.unwrap().name, "Customer Name");
        let wallet = va.blockchain_wallet.unwrap();
        assert_eq!(wallet.network, Network::Polygon);
        assert!(wallet.address.is_none());
    }

    #[test]
    fn virtual_account_parses_mexican_spei_account_with_null_partner_and_kyc() {
        let va: VirtualAccount = serde_json::from_value(serde_json::json!({
            "id": "va_000000000000",
            "banking_partner": null,
            "kyc_status": null,
            "mx": {
                "spei": { "clabe": "710969000000000001" },
                "beneficiary": null
            },
            "token": "USDC"
        }))
        .unwrap();
        assert!(va.us.is_none() && va.co.is_none());
        assert!(va.banking_partner.is_none() && va.kyc_status.is_none());
        let mx = va.mx.unwrap();
        assert_eq!(mx.spei.clabe, "710969000000000001");
        assert!(mx.beneficiary.is_none());
    }
}
