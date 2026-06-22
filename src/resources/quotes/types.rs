//! Request and response types for the `quotes` (payout quotes) resource.

use serde::{Deserialize, Serialize};

use crate::common::{Currency, CurrencyType, Network, Token};

/// Body for `Quotes::create` (`POST /instances/{instance_id}/quotes`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateQuoteInput {
    /// The bank account that will receive the payout.
    pub bank_account_id: String,
    /// Which side of the quote `request_amount` is denominated in.
    pub currency_type: CurrencyType,
    /// If `true`, the sender covers the fees; if `false`, the receiver does.
    pub cover_fees: bool,
    /// Requested amount in the smallest unit (cents). Minimum `500`.
    pub request_amount: i64,
    /// The network the payout will be settled on.
    pub network: Network,
    /// The stablecoin token used for the payout.
    pub token: Token,
    /// Optional memo/description (only honored for USD and BRL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional partner fee to apply to the quote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee_id: Option<String>,
}

/// The blockchain network referenced by a [`QuoteContract`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteContractNetwork {
    /// Human-readable network name, e.g. `"Ethereum"`.
    pub name: String,
    /// EVM chain id, e.g. `1`.
    pub chain_id: i64,
}

/// The on-chain ERC-20 approval the sender must make to fund a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteContract {
    /// The ERC-20 ABI for the network in the request.
    #[serde(default)]
    pub abi: Vec<serde_json::Value>,
    /// Token contract address for the requested network.
    pub address: String,
    /// The ERC-20 function to call, e.g. `"approve"`.
    pub function_name: String,
    /// The BlindPay contract address to approve.
    pub blindpay_contract_address: String,
    /// Amount to approve, with the token's decimal places applied (string).
    pub amount: String,
    /// The network the contract call targets.
    pub network: QuoteContractNetwork,
}

/// A payout quote, returned by `Quotes::create`.
///
/// Its `id` is the `quote_id` consumed when creating a payout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Quote {
    /// The quote id (prefix `qu_`).
    pub id: String,
    /// Epoch (seconds) after which the quote is no longer valid.
    pub expires_at: i64,
    /// Commercial FX quotation, e.g. `495` for `1 USD = 4.95 BRL`.
    pub commercial_quotation: i64,
    /// BlindPay FX quotation (commercial minus BlindPay's spread).
    pub blindpay_quotation: i64,
    /// Amount the receiver gets, in the smallest unit.
    pub receiver_amount: i64,
    /// Amount the sender must send, in the smallest unit.
    pub sender_amount: i64,
    /// Partner fee charged, in the smallest unit.
    #[serde(default)]
    pub partner_fee_amount: Option<i64>,
    /// Flat fee charged, in the smallest unit.
    #[serde(default)]
    pub flat_fee: Option<i64>,
    /// Billing fee charged via invoice, in the smallest unit.
    #[serde(default)]
    pub billing_fee_amount: Option<i64>,
    /// Estimated amount in the receiver bank account's local currency.
    #[serde(default)]
    pub receiver_local_amount: Option<i64>,
    /// The memo/description echoed back from the request.
    #[serde(default)]
    pub description: Option<String>,
    /// The on-chain approval the sender must make to fund the payout.
    #[serde(default)]
    pub contract: Option<QuoteContract>,
}

/// Body for `Quotes::get_fx_rate` (`POST /instances/{instance_id}/quotes/fx`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetFxRateInput {
    /// The stablecoin token to convert from.
    pub from: Token,
    /// The fiat currency to convert to.
    pub to: Currency,
    /// Requested amount in the smallest unit (cents). Minimum `500`.
    pub request_amount: i64,
    /// Which side of the quote `request_amount` is denominated in.
    pub currency_type: CurrencyType,
}

/// An FX-rate quote, returned by `Quotes::get_fx_rate`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct QuoteFx {
    /// Commercial FX quotation, e.g. `495` for `1 USD = 4.95 BRL`.
    pub commercial_quotation: i64,
    /// BlindPay FX quotation (commercial minus BlindPay's spread).
    pub blindpay_quotation: i64,
    /// The converted amount, in the smallest unit.
    pub result_amount: i64,
    /// The instance's flat fee, in the smallest unit.
    #[serde(default)]
    pub instance_flat_fee: Option<i64>,
    /// The instance's percentage fee, in basis points.
    pub instance_percentage_fee: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_quote_input_skips_optional_fields() {
        let input = CreateQuoteInput {
            bank_account_id: "ba_123".to_string(),
            currency_type: CurrencyType::Sender,
            cover_fees: true,
            request_amount: 1000,
            network: Network::Sepolia,
            token: Token::Usdc,
            description: None,
            partner_fee_id: None,
        };
        let value = serde_json::to_value(&input).unwrap();
        assert_eq!(value["request_amount"], 1000);
        assert_eq!(value["token"], "USDC");
        assert!(value.get("description").is_none());
        assert!(value.get("partner_fee_id").is_none());
    }

    #[test]
    fn quote_contract_uses_camel_case_wire_keys() {
        let json = serde_json::json!({
            "abi": [{}],
            "address": "0xabc",
            "functionName": "approve",
            "blindpayContractAddress": "0xdef",
            "amount": "1000000000000000000",
            "network": { "name": "Ethereum", "chainId": 1 }
        });
        let contract: QuoteContract = serde_json::from_value(json).unwrap();
        assert_eq!(contract.function_name, "approve");
        assert_eq!(contract.blindpay_contract_address, "0xdef");
        assert_eq!(contract.network.chain_id, 1);
        assert_eq!(contract.abi.len(), 1);
    }

    #[test]
    fn quote_defaults_absent_optional_fields() {
        let json = serde_json::json!({
            "id": "qu_123",
            "expires_at": 1712958191_i64,
            "commercial_quotation": 495,
            "blindpay_quotation": 485,
            "receiver_amount": 5240,
            "sender_amount": 1010
        });
        let quote: Quote = serde_json::from_value(json).unwrap();
        assert_eq!(quote.id, "qu_123");
        assert_eq!(quote.partner_fee_amount, None);
        assert_eq!(quote.flat_fee, None);
        assert!(quote.contract.is_none());
    }
}
