//! Request and response types for the `transfers` resource.

use serde::{Deserialize, Serialize};

use crate::common::{CurrencyType, Network, TrackingStatus, TransactionStatus};

/// A single blockchain step in a transfer's tracking timeline.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TransferTrackingStep {
    /// The step's status.
    pub step: TrackingStatus,
    /// On-chain transaction hash, if any.
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// Gas fee for the transaction, as a string, if any.
    #[serde(default)]
    pub gas_fee: Option<String>,
    /// When the step completed (ISO 8601), if it has.
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Error message for the step, if it failed.
    #[serde(default)]
    pub error_message: Option<String>,
}

/// The transaction-monitoring step of a transfer's tracking timeline.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TransferTrackingTransactionMonitoring {
    /// The step's status.
    pub step: TrackingStatus,
    /// Blockchain screening score (0–100), if computed.
    #[serde(default)]
    pub blockchain_screening: Option<i64>,
    /// Risk score (0–100), if computed.
    #[serde(default)]
    pub risk_score: Option<i64>,
    /// When the step completed (ISO 8601), if it has.
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// A transfer between two blockchain wallets.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Transfer {
    /// The transfer identifier.
    pub id: String,
    /// The transfer's general status.
    pub status: TransactionStatus,
    /// The quote this transfer was created from.
    pub transfer_quote_id: String,
    /// The instance this transfer belongs to.
    pub instance_id: String,
    /// Transaction-monitoring tracking step.
    pub tracking_transaction_monitoring: TransferTrackingTransactionMonitoring,
    /// Paymaster tracking step.
    pub tracking_paymaster: TransferTrackingStep,
    /// Bridge-swap tracking step.
    pub tracking_bridge_swap: TransferTrackingStep,
    /// Completion tracking step.
    pub tracking_complete: TransferTrackingStep,
    /// Partner-fee tracking step.
    pub tracking_partner_fee: TransferTrackingStep,
    /// When the transfer was created (ISO 8601).
    pub created_at: String,
    /// When the transfer was last updated (ISO 8601).
    pub updated_at: String,
    /// Receiver image URL, if any.
    #[serde(default)]
    pub image_url: Option<String>,
    /// Receiver first name, if any.
    #[serde(default)]
    pub first_name: Option<String>,
    /// Receiver last name, if any.
    #[serde(default)]
    pub last_name: Option<String>,
    /// Receiver legal name, if any.
    #[serde(default)]
    pub legal_name: Option<String>,
    /// The sending wallet's identifier.
    pub wallet_id: String,
    /// The token sent.
    pub sender_token: crate::common::Token,
    /// The amount debited from the sender, in the smallest unit (cents).
    pub sender_amount: i64,
    /// The amount credited to the receiver, in the smallest unit (cents).
    pub receiver_amount: i64,
    /// The receiver's blockchain network.
    pub receiver_network: Network,
    /// The token received.
    pub receiver_token: crate::common::Token,
    /// The receiver's wallet address.
    pub receiver_wallet_address: String,
    /// The partner fee charged, in the smallest unit (cents), if any.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// The wallet's external identifier, if any.
    #[serde(default)]
    pub external_id: Option<String>,
    /// The receiver this transfer's wallet belongs to.
    pub receiver_id: String,
    /// The sending wallet's blockchain address.
    pub address: String,
    /// The sending wallet's blockchain network.
    pub network: Network,
}

/// Input for `Transfers::create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateTransferInput {
    /// The quote to execute, e.g. `qu_000000000000`.
    pub transfer_quote_id: String,
}

impl CreateTransferInput {
    /// Creates the input from a transfer-quote ID.
    pub fn new(transfer_quote_id: impl Into<String>) -> Self {
        Self {
            transfer_quote_id: transfer_quote_id.into(),
        }
    }
}

/// Response from `Transfers::create`: the freshly created transfer's id, status,
/// and initial tracking steps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CreateTransferResponse {
    /// The transfer identifier.
    pub id: String,
    /// The transfer's general status.
    pub status: TransactionStatus,
    /// Bridge-swap tracking step.
    pub tracking_bridge_swap: TransferTrackingStep,
    /// Completion tracking step.
    pub tracking_complete: TransferTrackingStep,
    /// Paymaster tracking step.
    pub tracking_paymaster: TransferTrackingStep,
    /// Transaction-monitoring tracking step.
    pub tracking_transaction_monitoring: TransferTrackingTransactionMonitoring,
    /// Partner-fee tracking step.
    pub tracking_partner_fee: TransferTrackingStep,
}

/// Input for `TransferQuotes::create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateTransferQuoteInput {
    /// The sending wallet's identifier.
    pub wallet_id: String,
    /// Whether `request_amount` is denominated in the sender's or receiver's
    /// currency.
    pub amount_reference: CurrencyType,
    /// The requested amount, in the smallest unit (cents).
    pub request_amount: i64,
    /// The token to send.
    pub sender_token: crate::common::Token,
    /// The receiver's wallet address.
    pub receiver_wallet_address: String,
    /// The token to receive.
    pub receiver_token: crate::common::Token,
    /// The receiver's blockchain network.
    pub receiver_network: Network,
    /// Whether the sender covers the fees (`true`) or the receiver does
    /// (`false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_fees: Option<bool>,
    /// A partner fee to apply to the quote, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee_id: Option<String>,
}

impl CreateTransferQuoteInput {
    /// Creates a quote input with the required fields; optional fields default to
    /// unset.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet_id: impl Into<String>,
        amount_reference: CurrencyType,
        request_amount: i64,
        sender_token: crate::common::Token,
        receiver_wallet_address: impl Into<String>,
        receiver_token: crate::common::Token,
        receiver_network: Network,
    ) -> Self {
        Self {
            wallet_id: wallet_id.into(),
            amount_reference,
            request_amount,
            sender_token,
            receiver_wallet_address: receiver_wallet_address.into(),
            receiver_token,
            receiver_network,
            cover_fees: None,
            partner_fee_id: None,
        }
    }

    /// Sets whether the sender covers the fees.
    pub fn cover_fees(mut self, cover_fees: bool) -> Self {
        self.cover_fees = Some(cover_fees);
        self
    }

    /// Sets the partner fee to apply.
    pub fn partner_fee_id(mut self, partner_fee_id: impl Into<String>) -> Self {
        self.partner_fee_id = Some(partner_fee_id.into());
        self
    }
}

/// A transfer quote, returned by `TransferQuotes::create`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct TransferQuote {
    /// The quote identifier, e.g. `qu_000000000000`.
    pub id: String,
    /// Unix epoch (ms) after which the quote is no longer valid, if set.
    #[serde(default)]
    pub expires_at: Option<i64>,
    /// The commercial quotation, if set.
    #[serde(default)]
    pub commercial_quotation: Option<f64>,
    /// The BlindPay quotation, if set.
    #[serde(default)]
    pub blindpay_quotation: Option<f64>,
    /// The amount the receiver gets, in the smallest unit (cents).
    pub receiver_amount: i64,
    /// The amount the sender pays, in the smallest unit (cents).
    pub sender_amount: i64,
    /// The partner fee, in the smallest unit (cents), if any.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// The flat fee, in the smallest unit (cents).
    pub flat_fee: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_transfer_input_serializes_quote_id() {
        let json = serde_json::to_value(CreateTransferInput::new("qu_000000000000")).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "transfer_quote_id": "qu_000000000000" })
        );
    }

    #[test]
    fn create_transfer_quote_input_skips_unset_optionals() {
        let input = CreateTransferQuoteInput::new(
            "bl_000000000000",
            CurrencyType::Sender,
            1000,
            crate::common::Token::Usdc,
            "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
            crate::common::Token::Usdc,
            Network::Polygon,
        );
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "wallet_id": "bl_000000000000",
                "amount_reference": "sender",
                "request_amount": 1000,
                "sender_token": "USDC",
                "receiver_wallet_address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
                "receiver_token": "USDC",
                "receiver_network": "polygon"
            })
        );

        let with_opts = input.cover_fees(true).partner_fee_id("pf_000000000000");
        let json = serde_json::to_value(&with_opts).unwrap();
        assert_eq!(json["cover_fees"], serde_json::json!(true));
        assert_eq!(json["partner_fee_id"], serde_json::json!("pf_000000000000"));
    }
}
