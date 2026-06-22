//! Shared scalar enums and newtypes used across resources.
//!
//! The enums are *open* (see [`open_enum`](crate::common::open_enum)): values
//! the SDK doesn't recognize decode into `Unknown` rather than failing.

use serde::{Deserialize, Serialize};

use crate::common::open_enum;

open_enum! {
    /// A blockchain network supported by BlindPay.
    pub enum Network {
        /// Base mainnet.
        Base => "base",
        /// Ethereum Sepolia testnet.
        Sepolia => "sepolia",
        /// Arbitrum Sepolia testnet.
        ArbitrumSepolia => "arbitrum_sepolia",
        /// Base Sepolia testnet.
        BaseSepolia => "base_sepolia",
        /// Arbitrum One mainnet.
        Arbitrum => "arbitrum",
        /// Polygon mainnet.
        Polygon => "polygon",
        /// Polygon Amoy testnet.
        PolygonAmoy => "polygon_amoy",
        /// Ethereum mainnet.
        Ethereum => "ethereum",
        /// Stellar mainnet.
        Stellar => "stellar",
        /// Stellar testnet.
        StellarTestnet => "stellar_testnet",
        /// Tron mainnet.
        Tron => "tron",
        /// Solana mainnet.
        Solana => "solana",
        /// Solana devnet.
        SolanaDevnet => "solana_devnet",
    }
}

open_enum! {
    /// A stablecoin token. Wire values are uppercase.
    pub enum Token {
        /// USD Coin.
        Usdc => "USDC",
        /// Tether.
        Usdt => "USDT",
        /// BlindPay sandbox stablecoin.
        Usdb => "USDB",
    }
}

open_enum! {
    /// A fiat currency. Wire values are uppercase ISO 4217 codes.
    pub enum Currency {
        /// Brazilian Real.
        Brl => "BRL",
        /// US Dollar.
        Usd => "USD",
        /// Mexican Peso.
        Mxn => "MXN",
        /// Colombian Peso.
        Cop => "COP",
        /// Argentine Peso.
        Ars => "ARS",
        /// Euro.
        Eur => "EUR",
    }
}

open_enum! {
    /// A payin payment method.
    ///
    /// Distinct from [`Rail`] (used for bank accounts and payouts): payins use
    /// `spei`/`transfers`/`pse` where rails use `spei_bitso`/`transfers_bitso`/
    /// `ach_cop_bitso`.
    pub enum PaymentMethod {
        /// ACH (US).
        Ach => "ach",
        /// Domestic wire (US).
        Wire => "wire",
        /// PIX (Brazil).
        Pix => "pix",
        /// TED (Brazil).
        Ted => "ted",
        /// SPEI (Mexico).
        Spei => "spei",
        /// Transfers 3.0 (Argentina).
        Transfers => "transfers",
        /// PSE (Colombia).
        Pse => "pse",
        /// International SWIFT.
        InternationalSwift => "international_swift",
        /// Real-Time Payments (US).
        Rtp => "rtp",
    }
}

open_enum! {
    /// The payment rail of a bank account or payout.
    ///
    /// Shared across the `available`, `bank-accounts`, `payouts`, `quotes`, and
    /// `fees` resources.
    pub enum Rail {
        /// Domestic wire transfer (US).
        Wire => "wire",
        /// ACH transfer (US).
        Ach => "ach",
        /// PIX (Brazil).
        Pix => "pix",
        /// PIX Safe (Brazil).
        PixSafe => "pix_safe",
        /// TED (Brazil).
        Ted => "ted",
        /// SPEI via Bitso (Mexico).
        SpeiBitso => "spei_bitso",
        /// Transfers 3.0 via Bitso (Argentina).
        TransfersBitso => "transfers_bitso",
        /// ACH Colombia via Bitso (Colombia).
        AchCopBitso => "ach_cop_bitso",
        /// International SWIFT transfer.
        InternationalSwift => "international_swift",
        /// Real-Time Payments (US).
        Rtp => "rtp",
        /// SEPA (European Union).
        Sepa => "sepa",
    }
}

open_enum! {
    /// The KYC/KYB verification status of a customer.
    pub enum KycStatus {
        /// Verification in progress.
        Verifying => "verifying",
        /// Approved.
        Approved => "approved",
        /// Rejected.
        Rejected => "rejected",
        /// Deprecated / superseded.
        Deprecated => "deprecated",
        /// Pending manual review.
        PendingReview => "pending_review",
        /// Awaiting contract signature.
        AwaitingContract => "awaiting_contract",
        /// Additional compliance information requested.
        ComplianceRequest => "compliance_request",
    }
}

open_enum! {
    /// The status of a payin, payout, or transfer.
    ///
    /// This is the superset across all three resources; a given resource uses a
    /// subset (for example, transfers never report `on_hold`). Being an open
    /// enum keeps any per-resource value safe.
    pub enum TransactionStatus {
        /// Being processed.
        Processing => "processing",
        /// Temporarily on hold.
        OnHold => "on_hold",
        /// Failed.
        Failed => "failed",
        /// Refunded.
        Refunded => "refunded",
        /// Completed.
        Completed => "completed",
    }
}

open_enum! {
    /// Whether an account belongs to an individual or a business.
    pub enum AccountClass {
        /// Individual account holder.
        Individual => "individual",
        /// Business account holder.
        Business => "business",
    }
}

open_enum! {
    /// A bank account type.
    pub enum AccountType {
        /// Checking account.
        Checking => "checking",
        /// Savings account.
        Saving => "saving",
    }
}

open_enum! {
    /// Which side of a quote a requested amount is denominated in.
    pub enum CurrencyType {
        /// The sender's currency.
        Sender => "sender",
        /// The receiver's currency.
        Receiver => "receiver",
    }
}

open_enum! {
    /// A webhook event type. Wire values are dotted camelCase (e.g.
    /// `bankAccount.new`), so they're matched verbatim.
    pub enum WebhookEvent {
        /// A receiver (legacy customer) was created.
        ReceiverNew => "receiver.new",
        /// A receiver was updated.
        ReceiverUpdate => "receiver.update",
        /// A receiver was deleted.
        ReceiverDelete => "receiver.delete",
        /// A customer was created.
        CustomerNew => "customer.new",
        /// A customer was updated.
        CustomerUpdate => "customer.update",
        /// A customer was deleted.
        CustomerDelete => "customer.delete",
        /// A bank account was created.
        BankAccountNew => "bankAccount.new",
        /// A payout was created.
        PayoutNew => "payout.new",
        /// A payout was updated.
        PayoutUpdate => "payout.update",
        /// A payout completed.
        PayoutComplete => "payout.complete",
        /// A payout's partner fee was settled.
        PayoutPartnerFee => "payout.partnerFee",
        /// A blockchain wallet was created.
        BlockchainWalletNew => "blockchainWallet.new",
        /// A payin was created.
        PayinNew => "payin.new",
        /// A payin was updated.
        PayinUpdate => "payin.update",
        /// A payin completed.
        PayinComplete => "payin.complete",
        /// A payin's partner fee was settled.
        PayinPartnerFee => "payin.partnerFee",
        /// Terms of service were accepted.
        TosAccept => "tos.accept",
        /// A limit-increase request was created.
        LimitIncreaseNew => "limitIncrease.new",
        /// A limit-increase request was updated.
        LimitIncreaseUpdate => "limitIncrease.update",
        /// A virtual account was created.
        VirtualAccountNew => "virtualAccount.new",
        /// A virtual account completed setup.
        VirtualAccountComplete => "virtualAccount.complete",
        /// A transfer was created.
        TransferNew => "transfer.new",
        /// A transfer was updated.
        TransferUpdate => "transfer.update",
        /// A transfer completed.
        TransferComplete => "transfer.complete",
        /// A custodial wallet was created.
        WalletNew => "wallet.new",
        /// A custodial wallet received an inbound transfer.
        WalletInbound => "wallet.inbound",
    }
}

/// An ISO 3166-1 alpha-2 country code (uppercase, e.g. `"US"`, `"BR"`).
///
/// Modeled as a transparent newtype rather than an enum: the set is large and
/// the API occasionally returns non-ISO display codes (e.g. `eu` for SEPA), so
/// this stays strongly typed yet forward-compatible.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct Country(String);

impl Country {
    /// Returns the country code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Country {
    fn from(value: &str) -> Self {
        Country(value.to_string())
    }
}

impl From<String> for Country {
    fn from(value: String) -> Self {
        Country(value)
    }
}

impl AsRef<str> for Country {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<str> for Country {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Country {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_parses_known_and_unknown_values() {
        assert_eq!(Rail::from("pix"), Rail::Pix);
        assert_eq!(Rail::Pix.as_str(), "pix");
        assert_eq!(
            Rail::from("brand_new_rail"),
            Rail::Unknown("brand_new_rail".to_string())
        );
    }

    #[test]
    fn rail_serde_round_trips() {
        assert_eq!(serde_json::to_string(&Rail::Sepa).unwrap(), "\"sepa\"");
        let parsed: Rail = serde_json::from_str("\"future\"").unwrap();
        assert_eq!(parsed, Rail::Unknown("future".to_string()));
    }

    #[test]
    fn open_enums_round_trip_uppercase_and_snake_values() {
        // Uppercase wire values (tokens, currencies) survive unchanged.
        assert_eq!(serde_json::to_string(&Token::Usdc).unwrap(), "\"USDC\"");
        assert_eq!(
            serde_json::from_str::<Currency>("\"EUR\"").unwrap(),
            Currency::Eur
        );
        // snake_case network/status values.
        assert_eq!(
            serde_json::to_string(&Network::SolanaDevnet).unwrap(),
            "\"solana_devnet\""
        );
        assert_eq!(
            serde_json::from_str::<TransactionStatus>("\"on_hold\"").unwrap(),
            TransactionStatus::OnHold
        );
        assert_eq!(
            serde_json::from_str::<KycStatus>("\"compliance_request\"").unwrap(),
            KycStatus::ComplianceRequest
        );
        // Unknown fallthrough for an unrecognized value.
        assert_eq!(
            serde_json::from_str::<Network>("\"newchain\"").unwrap(),
            Network::Unknown("newchain".to_string())
        );
        assert_eq!(PaymentMethod::Pse.as_str(), "pse");
        assert_eq!(AccountType::Saving.as_str(), "saving");
        assert_eq!(CurrencyType::Sender.as_str(), "sender");
        assert_eq!(AccountClass::Business.as_str(), "business");
    }

    #[test]
    fn webhook_event_round_trips_dotted_camel_case() {
        assert_eq!(
            serde_json::to_string(&WebhookEvent::BankAccountNew).unwrap(),
            "\"bankAccount.new\""
        );
        assert_eq!(
            serde_json::from_str::<WebhookEvent>("\"payout.partnerFee\"").unwrap(),
            WebhookEvent::PayoutPartnerFee
        );
        assert_eq!(
            serde_json::from_str::<WebhookEvent>("\"future.event\"").unwrap(),
            WebhookEvent::Unknown("future.event".to_string())
        );
    }

    #[test]
    fn country_is_transparent() {
        let c: Country = serde_json::from_str("\"US\"").unwrap();
        assert_eq!(c.as_str(), "US");
        assert_eq!(c, "US");
        assert_eq!(serde_json::to_string(&c).unwrap(), "\"US\"");
    }
}
