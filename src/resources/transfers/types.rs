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
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct TransferTrackingTransactionMonitoring {
    /// The step's status.
    pub step: TrackingStatus,
    /// Blockchain screening score (0–100), if computed.
    #[serde(default)]
    pub blockchain_screening: Option<i64>,
    /// Risk score (0–100, may be fractional), if computed.
    #[serde(default)]
    pub risk_score: Option<f64>,
    /// When the step completed (ISO 8601), if it has.
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// A transfer between two blockchain wallets.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    /// Partner-fee tracking step, if any.
    #[serde(default)]
    pub tracking_partner_fee: Option<TransferTrackingStep>,
    /// When the transfer was created (ISO 8601).
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the transfer was last updated (ISO 8601).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Customer image URL, if any.
    #[serde(default)]
    pub image_url: Option<String>,
    /// Customer first name, if any.
    #[serde(default)]
    pub first_name: Option<String>,
    /// Customer last name, if any.
    #[serde(default)]
    pub last_name: Option<String>,
    /// Customer legal name, if any.
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
    /// The destination blockchain network.
    pub customer_network: Network,
    /// The token received.
    pub customer_token: crate::common::Token,
    /// The destination wallet address.
    pub customer_wallet_address: String,
    /// The partner fee charged, in the smallest unit (cents), if any.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// The wallet's external identifier, if any.
    #[serde(default)]
    pub external_id: Option<String>,
    /// The customer this transfer's wallet belongs to (`re_…`).
    pub customer_id: String,
    /// The sending wallet's blockchain address.
    #[serde(default)]
    pub address: Option<String>,
    /// The sending wallet's blockchain network.
    pub network: Network,
    /// Partner fee in basis points (`150` = 1.50%).
    #[serde(default)]
    pub partner_percentage_fee: Option<i64>,
    /// Partner flat fee in integer cents of the sender token.
    #[serde(default)]
    pub partner_flat_fee: Option<i64>,
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    /// Partner-fee tracking step, if any.
    #[serde(default)]
    pub tracking_partner_fee: Option<TransferTrackingStep>,
}

/// Input for `TransferQuotes::create`.
///
/// A swap between different tokens needs at least 100 USDC, measured on the
/// USDC side; a smaller amount fails with 400
/// [`ErrorCode::TransfersSwapAmountBelowMinimum`](crate::ErrorCode::TransfersSwapAmountBelowMinimum).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateTransferQuoteInput {
    /// The sending wallet's identifier.
    pub wallet_id: String,
    /// Whether `request_amount` is denominated in the sender's or receiver's
    /// currency.
    pub amount_reference: CurrencyType,
    /// The requested amount, in the smallest unit (cents). Swaps between
    /// different tokens need at least 100 USDC (`10000`) on the USDC side.
    pub request_amount: i64,
    /// The token to send.
    pub sender_token: crate::common::Token,
    /// The destination wallet address.
    pub customer_wallet_address: String,
    /// The token to receive.
    pub customer_token: crate::common::Token,
    /// The destination blockchain network.
    pub customer_network: Network,
    /// Whether the sender covers the fees (`true`) or the receiver does
    /// (`false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_fees: Option<bool>,
    /// A partner fee to apply to the quote, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee_id: Option<String>,
    /// Partner fee in basis points (`100` = 1%).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_percentage_fee: Option<i64>,
    /// Partner flat fee in integer cents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_flat_fee: Option<i64>,
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
        customer_wallet_address: impl Into<String>,
        customer_token: crate::common::Token,
        customer_network: Network,
    ) -> Self {
        Self {
            wallet_id: wallet_id.into(),
            amount_reference,
            request_amount,
            sender_token,
            customer_wallet_address: customer_wallet_address.into(),
            customer_token,
            customer_network,
            cover_fees: None,
            partner_fee_id: None,
            partner_percentage_fee: None,
            partner_flat_fee: None,
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

    /// Sets `partner_percentage_fee`.
    pub fn partner_percentage_fee(mut self, partner_percentage_fee: i64) -> Self {
        self.partner_percentage_fee = Some(partner_percentage_fee);
        self
    }

    /// Sets `partner_flat_fee`.
    pub fn partner_flat_fee(mut self, partner_flat_fee: i64) -> Self {
        self.partner_flat_fee = Some(partner_flat_fee);
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
    /// Partner fee in basis points (`150` = 1.50%).
    #[serde(default)]
    pub partner_percentage_fee: Option<i64>,
    /// Partner flat fee in integer cents of the sender token.
    #[serde(default)]
    pub partner_flat_fee: Option<i64>,
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
                "customer_wallet_address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
                "customer_token": "USDC",
                "customer_network": "polygon"
            })
        );

        let with_opts = input.cover_fees(true).partner_fee_id("pf_000000000000");
        let json = serde_json::to_value(&with_opts).unwrap();
        assert_eq!(json["cover_fees"], serde_json::json!(true));
        assert_eq!(json["partner_fee_id"], serde_json::json!("pf_000000000000"));
    }
}

/// Query parameters for `Transfers::list` and `Transfers::list_auto_paging`.
/// The transfers list is always paginated (default page size 10).
///
/// ```
/// use blindpay::{Limit, ListTransfersParams, TransactionStatus};
/// let params = ListTransfersParams::new()
///     .limit(Limit::Fifty)
///     .status(TransactionStatus::Completed);
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct ListTransfersParams {
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<crate::common::Limit>,
    /// Cursor: return items after this transfer id (`next_page`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_after: Option<String>,
    /// Cursor: return items before this transfer id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_before: Option<String>,
    /// Filter by customer id (`re_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    /// Filter by source wallet id (`bl_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<String>,
    /// Filter by status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<crate::common::TransactionStatus>,
    /// Filter by network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    /// Filter by token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<crate::common::Token>,
}

impl ListTransfersParams {
    /// Creates empty list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `limit`: maximum number of items to return.
    pub fn limit(mut self, limit: crate::common::Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets `starting_after`: cursor: return items after this transfer id (`next_page`).
    pub fn starting_after(mut self, starting_after: impl Into<String>) -> Self {
        self.starting_after = Some(starting_after.into());
        self
    }

    /// Sets `ending_before`: cursor: return items before this transfer id.
    pub fn ending_before(mut self, ending_before: impl Into<String>) -> Self {
        self.ending_before = Some(ending_before.into());
        self
    }

    /// Sets `customer_id`: filter by customer id (`re_…`).
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.customer_id = Some(customer_id.into());
        self
    }

    /// Sets `wallet_id`: filter by source wallet id (`bl_…`).
    pub fn wallet_id(mut self, wallet_id: impl Into<String>) -> Self {
        self.wallet_id = Some(wallet_id.into());
        self
    }

    /// Sets `status`: filter by status.
    pub fn status(mut self, status: crate::common::TransactionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets `network`: filter by network.
    pub fn network(mut self, network: Network) -> Self {
        self.network = Some(network);
        self
    }

    /// Sets `token`: filter by token.
    pub fn token(mut self, token: crate::common::Token) -> Self {
        self.token = Some(token);
        self
    }
}

impl From<crate::common::PaginationParams> for ListTransfersParams {
    fn from(p: crate::common::PaginationParams) -> Self {
        Self {
            limit: p.limit,
            starting_after: p.starting_after,
            ending_before: p.ending_before,
            ..Self::default()
        }
    }
}
