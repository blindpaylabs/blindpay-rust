//! Request and response types for the `wallets` resources (blockchain wallets,
//! custodial wallets, and offramp wallets).

use serde::{Deserialize, Serialize};

use crate::common::{Network, Token};

/// Response from `BlockchainWallets::create_asset_trustline`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CreateAssetTrustlineResponse {
    /// The unsigned Stellar transaction envelope (XDR) to sign and submit.
    pub xdr: String,
}

/// Input for `BlockchainWallets::mint_usdb_stellar`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MintUsdbStellarInput {
    /// The Stellar wallet address to mint to.
    pub address: String,
    /// The amount of USDB to mint.
    pub amount: String,
    /// A signed trustline transaction (XDR) to submit first, when one is needed.
    #[serde(rename = "signedXdr", skip_serializing_if = "Option::is_none")]
    pub signed_xdr: Option<String>,
}

/// Input for `BlockchainWallets::mint_usdb_solana`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MintUsdbSolanaInput {
    /// The Solana wallet address to mint to.
    pub address: String,
    /// The amount of USDB to mint.
    pub amount: String,
}

/// Response from `BlockchainWallets::mint_usdb_solana`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct MintUsdbSolanaResponse {
    /// Whether the mint succeeded.
    pub success: bool,
    /// The Solana transaction signature, when successful.
    #[serde(default)]
    pub signature: Option<String>,
    /// The error message, when unsuccessful.
    #[serde(default)]
    pub error: Option<String>,
}

/// Input for `BlockchainWallets::prepare_solana_delegation_transaction`.
///
/// Provide either `quote_id` (recommended) or both `token_address` and
/// `amount`; `owner_address` is always required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrepareSolanaDelegationInput {
    /// The Solana wallet address that owns the tokens.
    pub owner_address: String,
    /// A quote ID; when set, the token address and amount are taken from it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    /// The SPL token mint address. Required if `quote_id` is not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,
    /// The token amount to delegate. Required if `quote_id` is not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

/// Response from `BlockchainWallets::prepare_solana_delegation_transaction`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PrepareSolanaDelegationResponse {
    /// Whether the transaction was prepared successfully.
    pub success: bool,
    /// The prepared (base64-encoded) Solana transaction, when successful.
    #[serde(default)]
    pub transaction: Option<String>,
}

/// A blockchain wallet registered for a customer, returned by the
/// `wallets.blockchain` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BlockchainWallet {
    /// Wallet identifier (prefix `bw_`).
    pub id: String,
    /// Display name for the wallet.
    pub name: String,
    /// The blockchain network the wallet is on.
    pub network: Network,
    /// The wallet address. Present for account-abstraction registrations.
    #[serde(default)]
    pub address: Option<String>,
    /// The signature transaction hash. Present for signature-based (EVM)
    /// registrations.
    #[serde(default)]
    pub signature_tx_hash: Option<String>,
    /// Whether the wallet was registered as an account-abstraction wallet
    /// (address-based) rather than via a signature.
    pub is_account_abstraction: bool,
    /// Identifier of the customer the wallet belongs to (wire field
    /// `receiver_id`, prefix `re_`).
    pub receiver_id: String,
}

/// The challenge message a customer signs to register a blockchain wallet by
/// signature, returned by `wallets.blockchain.get_wallet_message`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BlockchainWalletMessage {
    /// The message to sign.
    pub message: String,
}

/// Request body for registering an account-abstraction blockchain wallet by
/// address (`is_account_abstraction: true`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateBlockchainWalletWithAddressInput {
    /// Display name for the wallet.
    pub name: String,
    /// The blockchain network the wallet is on.
    pub network: Network,
    /// The wallet address.
    pub address: String,
    /// Always `true` for this constructor.
    pub is_account_abstraction: bool,
}

/// Request body for registering a blockchain wallet by signature
/// (`is_account_abstraction: false`, EVM networks only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateBlockchainWalletWithHashInput {
    /// Display name for the wallet.
    pub name: String,
    /// The blockchain network the wallet is on (EVM only).
    pub network: Network,
    /// The transaction hash of the signature that proves wallet ownership.
    pub signature_tx_hash: String,
    /// Always `false` for this constructor.
    pub is_account_abstraction: bool,
}

/// A BlindPay-custodied (Circle) wallet, returned by the `wallets.custodial`
/// endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Wallet {
    /// Wallet identifier (prefix `bl_`).
    pub id: String,
    /// Display name for the wallet.
    pub name: String,
    /// Caller-supplied external identifier, if any.
    #[serde(default)]
    pub external_id: Option<String>,
    /// The wallet address, once Circle has provisioned it.
    #[serde(default)]
    pub address: Option<String>,
    /// The blockchain network the wallet is on.
    pub network: Network,
    /// When the wallet was created.
    pub created_at: String,
}

/// Request body for creating a custodial wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateWalletInput {
    /// The blockchain network for the wallet.
    pub network: Network,
    /// Display name for the wallet.
    pub name: String,
    /// Optional caller-supplied external identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

/// The balance of a single token within a custodial wallet.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct WalletTokenBalance {
    /// The token's on-chain address.
    pub address: String,
    /// Identifier of the balance record.
    pub id: String,
    /// The token symbol.
    pub symbol: Token,
    /// The balance amount. This is a fractional token balance (e.g. `12.5`
    /// tokens), not a minor-unit integer like the monetary amounts elsewhere in
    /// the API — keep it `f64`.
    pub amount: f64,
}

/// The per-token balances of a custodial wallet, returned by
/// `wallets.custodial.get_balance`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct WalletBalance {
    /// USDC balance.
    #[serde(rename = "USDC")]
    pub usdc: WalletTokenBalance,
    /// USDT balance.
    #[serde(rename = "USDT")]
    pub usdt: WalletTokenBalance,
    /// USDB balance.
    #[serde(rename = "USDB")]
    pub usdb: WalletTokenBalance,
}

/// An offramp wallet attached to a bank account, returned by the
/// `wallets.offramp` endpoints.
///
/// The `create` endpoint returns a subset of these fields; the absent ones
/// (`instance_id`, `receiver_id`, `bank_account_id`, `created_at`, `updated_at`)
/// default to `None`/empty in that case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OfframpWallet {
    /// Wallet identifier (prefix `ow_`).
    pub id: String,
    /// Caller-supplied external identifier, if any.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Identifier of the instance the wallet belongs to. Absent on `create`.
    #[serde(default)]
    pub instance_id: Option<String>,
    /// Identifier of the customer the wallet belongs to (wire field
    /// `receiver_id`). Absent on `create`.
    #[serde(default)]
    pub receiver_id: Option<String>,
    /// Identifier of the bank account the wallet is attached to. Absent on
    /// `create`.
    #[serde(default)]
    pub bank_account_id: Option<String>,
    /// The underlying Circle wallet identifier, if any.
    #[serde(default)]
    pub circle_wallet_id: Option<String>,
    /// The blockchain network the wallet is on.
    pub network: Network,
    /// The wallet address.
    pub address: String,
    /// When the wallet was created. Absent on `create`.
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the wallet was last updated. Absent on `create`.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Request body for creating an offramp wallet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateOfframpWalletInput {
    /// Optional caller-supplied external identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// The blockchain network for the wallet.
    pub network: Network,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockchain_wallet_uses_receiver_id_wire_field() {
        let json = serde_json::json!({
            "id": "bw_000000000000",
            "name": "My Wallet",
            "network": "polygon",
            "address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
            "is_account_abstraction": true,
            "receiver_id": "re_000000000000"
        });
        let wallet: BlockchainWallet = serde_json::from_value(json).unwrap();
        assert_eq!(wallet.id, "bw_000000000000");
        assert_eq!(wallet.network, Network::Polygon);
        assert!(wallet.is_account_abstraction);
        assert_eq!(wallet.receiver_id, "re_000000000000");
        assert!(wallet.signature_tx_hash.is_none());
    }

    #[test]
    fn create_blockchain_wallet_with_hash_serializes_discriminator() {
        let input = CreateBlockchainWalletWithHashInput {
            name: "Sig Wallet".to_string(),
            network: Network::Base,
            signature_tx_hash: "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359".to_string(),
            is_account_abstraction: false,
        };
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "name": "Sig Wallet",
                "network": "base",
                "signature_tx_hash": "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359",
                "is_account_abstraction": false
            })
        );
    }

    #[test]
    fn wallet_balance_maps_uppercase_token_keys() {
        let json = serde_json::json!({
            "USDC": { "address": "0xabc", "id": "tok_1", "symbol": "USDC", "amount": 1000.0 },
            "USDT": { "address": "0xdef", "id": "tok_2", "symbol": "USDT", "amount": 0.0 },
            "USDB": { "address": "0xghi", "id": "tok_3", "symbol": "USDB", "amount": 25.5 }
        });
        let balance: WalletBalance = serde_json::from_value(json).unwrap();
        assert_eq!(balance.usdc.symbol, Token::Usdc);
        assert_eq!(balance.usdc.amount, 1000.0);
        assert_eq!(balance.usdb.amount, 25.5);
    }

    #[test]
    fn offramp_wallet_create_shape_defaults_absent_fields() {
        let json = serde_json::json!({
            "id": "ow_000000000000",
            "external_id": null,
            "circle_wallet_id": null,
            "network": "tron",
            "address": "TALJN9zTTEL9TVBb4WuTt6wLvPqJZr3hvb"
        });
        let wallet: OfframpWallet = serde_json::from_value(json).unwrap();
        assert_eq!(wallet.id, "ow_000000000000");
        assert_eq!(wallet.network, Network::Tron);
        assert!(wallet.bank_account_id.is_none());
        assert!(wallet.created_at.is_none());
    }

    #[test]
    fn create_offramp_wallet_skips_unset_external_id() {
        let input = CreateOfframpWalletInput {
            external_id: None,
            network: Network::Base,
        };
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json, serde_json::json!({ "network": "base" }));
    }
}
