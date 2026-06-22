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

/// The blockchain wallet a virtual account settles to, as embedded in the
/// virtual-account response.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct VirtualAccountWallet {
    /// The wallet's blockchain network.
    pub network: Network,
    /// The on-chain wallet address.
    pub address: String,
}

/// A virtual account, returned by the `virtual_accounts` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct VirtualAccount {
    /// The virtual account identifier (`va_…`).
    pub id: String,
    /// The banking partner backing the account.
    pub banking_partner: BankingPartner,
    /// The KYC/KYB status of the owning customer.
    pub kyc_status: KycStatus,
    /// The US banking details for funding the account.
    pub us: UsVirtualAccountDetails,
    /// The stablecoin token funds are converted to.
    pub token: Token,
    /// The blockchain wallet identifier (`bw_…`) funds settle to. May be absent.
    #[serde(default)]
    pub blockchain_wallet_id: Option<String>,
    /// The embedded blockchain wallet. Absent or `null` when no wallet is linked.
    #[serde(default)]
    pub blockchain_wallet: Option<VirtualAccountWallet>,
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
        }
    }
}

/// Body for updating a virtual account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateVirtualAccountInput {
    /// The stablecoin token funds are converted to.
    pub token: Token,
    /// The blockchain wallet identifier (`bw_…`) funds settle to.
    pub blockchain_wallet_id: String,
}

impl UpdateVirtualAccountInput {
    /// Creates a new update input from the two required fields.
    pub fn new(token: Token, blockchain_wallet_id: impl Into<String>) -> Self {
        Self {
            token,
            blockchain_wallet_id: blockchain_wallet_id.into(),
        }
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
        assert_eq!(va.banking_partner, BankingPartner::Jpmorgan);
        assert_eq!(va.kyc_status, KycStatus::Approved);
        assert_eq!(va.us.wire.account_number, "222222222");
        assert!(va.us.ach.is_some());
        assert!(va.us.rtp.is_none());
        assert_eq!(va.us.swift_bic_code.as_deref(), Some("TCCLGB3L"));
        assert_eq!(
            va.us
                .swift_intermediary_bank
                .unwrap()
                .swift_code_bic
                .as_deref(),
            Some("CHASUS33")
        );
        assert!(va.blockchain_wallet.is_none());
    }
}
