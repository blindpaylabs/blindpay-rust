//! Typed webhook event payloads, keyed on `webhook_event`.

use serde::Deserialize;
use serde::de::Error as _;

use crate::common::{AccountClass, KycStatus, Network, WebhookEvent};
use crate::resources::customers::{BankAccount, Customer, KycType, LimitIncrease};
use crate::resources::payables::Payable;
use crate::resources::payins::CreatePayinResponse;
use crate::resources::payouts::CreatePayoutResponse;
use crate::resources::transfers::CreateTransferResponse;
use crate::resources::virtual_accounts::VirtualAccount;
use crate::resources::wallets::{BlockchainWallet, Wallet, WalletTokenBalance};

/// The `customer.delete` payload (spec `CustomerDeleteWebhookOut`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerDeleteEvent {
    /// Customer id (`re_…`).
    pub id: String,
    /// Customer id (`re_…`).
    pub customer_id: String,
    /// Owning instance id (`in_…`).
    pub instance_id: String,
    /// Individual or business.
    #[serde(rename = "type")]
    pub account_class: AccountClass,
    /// KYC tier.
    pub kyc_type: KycType,
    /// KYC status at deletion.
    pub kyc_status: KycStatus,
    /// Deletion timestamp (ISO 8601).
    pub deleted_at: String,
}

/// The `tos.accept` payload (spec `TosAcceptWebhookOut`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TosAcceptEvent {
    /// Accepted terms id (`to_…`).
    pub tos_id: String,
    /// The `idempotency_key` sent when the ToS flow was initiated.
    pub idempotency_key: String,
    /// The customer who accepted, when known. May be `null`.
    #[serde(default)]
    pub customer_id: Option<String>,
    /// Terms version (e.g. `1.0.1`). May be `null`.
    #[serde(default)]
    pub version: Option<String>,
}

/// The `wallet.inbound` payload: a deposit landed in a custodial wallet
/// (spec `WalletInboundWebhookOut`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct WalletInboundEvent {
    /// Wallet id (`bl_…`).
    pub id: String,
    /// Wallet address.
    pub address: String,
    /// Wallet network.
    pub network: Network,
    /// The received token and amount (whole tokens).
    pub token: WalletTokenBalance,
    /// Deposit id, matching the wallet deposits list (`wd_…`).
    pub deposit_id: String,
    /// On-chain transaction hash.
    pub transaction_hash: String,
}

/// A verified webhook delivery, decoded by its `webhook_event` name.
///
/// Returned by [`construct_webhook_event`](super::construct_webhook_event).
/// Event names this SDK version doesn't know decode to
/// [`Unknown`](Self::Unknown) with the raw JSON, so a new event never breaks
/// your handler.
///
/// A failed payout can arrive on **both** `payout.update` and
/// `payout.complete`: branch on the payload's `status`, not the event name.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum WebhookEventPayload {
    /// `customer.new`
    CustomerNew(Customer),
    /// `customer.update`
    CustomerUpdate(Customer),
    /// `customer.delete`
    CustomerDelete(CustomerDeleteEvent),
    /// `bankAccount.new`
    BankAccountNew(BankAccount),
    /// `payout.new`
    PayoutNew(CreatePayoutResponse),
    /// `payout.update`
    PayoutUpdate(CreatePayoutResponse),
    /// `payout.complete`
    PayoutComplete(CreatePayoutResponse),
    /// `payout.partnerFee`
    PayoutPartnerFee(CreatePayoutResponse),
    /// `blockchainWallet.new`
    BlockchainWalletNew(BlockchainWallet),
    /// `blockchainWallet.update`
    BlockchainWalletUpdate(BlockchainWallet),
    /// `payin.new`
    PayinNew(CreatePayinResponse),
    /// `payin.update`
    PayinUpdate(CreatePayinResponse),
    /// `payin.complete`
    PayinComplete(CreatePayinResponse),
    /// `payin.partnerFee`
    PayinPartnerFee(CreatePayinResponse),
    /// `tos.accept`
    TosAccept(TosAcceptEvent),
    /// `limitIncrease.new`
    LimitIncreaseNew(LimitIncrease),
    /// `limitIncrease.update`
    LimitIncreaseUpdate(LimitIncrease),
    /// `virtualAccount.new`
    VirtualAccountNew(VirtualAccount),
    /// `virtualAccount.complete`
    VirtualAccountComplete(VirtualAccount),
    /// `transfer.new`
    TransferNew(CreateTransferResponse),
    /// `transfer.update`
    TransferUpdate(CreateTransferResponse),
    /// `transfer.complete`
    TransferComplete(CreateTransferResponse),
    /// `wallet.new`
    WalletNew(Wallet),
    /// `wallet.update`
    WalletUpdate(Wallet),
    /// `wallet.inbound`
    WalletInbound(WalletInboundEvent),
    /// `payable.new`
    PayableNew(Payable),
    /// `payable.update`
    PayableUpdate(Payable),
    /// `payable.complete`
    PayableComplete(Payable),
    /// An event this SDK version doesn't model, as raw JSON (including its
    /// `webhook_event`).
    Unknown(serde_json::Value),
}

impl WebhookEventPayload {
    /// The event name (`webhook_event`).
    pub fn event(&self) -> WebhookEvent {
        use WebhookEventPayload as P;
        match self {
            P::CustomerNew(_) => WebhookEvent::CustomerNew,
            P::CustomerUpdate(_) => WebhookEvent::CustomerUpdate,
            P::CustomerDelete(_) => WebhookEvent::CustomerDelete,
            P::BankAccountNew(_) => WebhookEvent::BankAccountNew,
            P::PayoutNew(_) => WebhookEvent::PayoutNew,
            P::PayoutUpdate(_) => WebhookEvent::PayoutUpdate,
            P::PayoutComplete(_) => WebhookEvent::PayoutComplete,
            P::PayoutPartnerFee(_) => WebhookEvent::PayoutPartnerFee,
            P::BlockchainWalletNew(_) => WebhookEvent::BlockchainWalletNew,
            P::BlockchainWalletUpdate(_) => WebhookEvent::BlockchainWalletUpdate,
            P::PayinNew(_) => WebhookEvent::PayinNew,
            P::PayinUpdate(_) => WebhookEvent::PayinUpdate,
            P::PayinComplete(_) => WebhookEvent::PayinComplete,
            P::PayinPartnerFee(_) => WebhookEvent::PayinPartnerFee,
            P::TosAccept(_) => WebhookEvent::TosAccept,
            P::LimitIncreaseNew(_) => WebhookEvent::LimitIncreaseNew,
            P::LimitIncreaseUpdate(_) => WebhookEvent::LimitIncreaseUpdate,
            P::VirtualAccountNew(_) => WebhookEvent::VirtualAccountNew,
            P::VirtualAccountComplete(_) => WebhookEvent::VirtualAccountComplete,
            P::TransferNew(_) => WebhookEvent::TransferNew,
            P::TransferUpdate(_) => WebhookEvent::TransferUpdate,
            P::TransferComplete(_) => WebhookEvent::TransferComplete,
            P::WalletNew(_) => WebhookEvent::WalletNew,
            P::WalletUpdate(_) => WebhookEvent::WalletUpdate,
            P::WalletInbound(_) => WebhookEvent::WalletInbound,
            P::PayableNew(_) => WebhookEvent::PayableNew,
            P::PayableUpdate(_) => WebhookEvent::PayableUpdate,
            P::PayableComplete(_) => WebhookEvent::PayableComplete,
            P::Unknown(raw) => WebhookEvent::from(
                raw.get("webhook_event")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for WebhookEventPayload {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use WebhookEventPayload as P;
        let raw = serde_json::Value::deserialize(deserializer)?;
        let Some(name) = raw.get("webhook_event").and_then(serde_json::Value::as_str) else {
            return Err(D::Error::missing_field("webhook_event"));
        };
        fn typed<T: serde::de::DeserializeOwned, E: serde::de::Error>(
            raw: serde_json::Value,
        ) -> Result<T, E> {
            serde_json::from_value(raw).map_err(E::custom)
        }
        let event = WebhookEvent::from(name);
        Ok(match event {
            WebhookEvent::CustomerNew => P::CustomerNew(typed(raw)?),
            WebhookEvent::CustomerUpdate => P::CustomerUpdate(typed(raw)?),
            WebhookEvent::CustomerDelete => P::CustomerDelete(typed(raw)?),
            WebhookEvent::BankAccountNew => P::BankAccountNew(typed(raw)?),
            WebhookEvent::PayoutNew => P::PayoutNew(typed(raw)?),
            WebhookEvent::PayoutUpdate => P::PayoutUpdate(typed(raw)?),
            WebhookEvent::PayoutComplete => P::PayoutComplete(typed(raw)?),
            WebhookEvent::PayoutPartnerFee => P::PayoutPartnerFee(typed(raw)?),
            WebhookEvent::BlockchainWalletNew => P::BlockchainWalletNew(typed(raw)?),
            WebhookEvent::BlockchainWalletUpdate => P::BlockchainWalletUpdate(typed(raw)?),
            WebhookEvent::PayinNew => P::PayinNew(typed(raw)?),
            WebhookEvent::PayinUpdate => P::PayinUpdate(typed(raw)?),
            WebhookEvent::PayinComplete => P::PayinComplete(typed(raw)?),
            WebhookEvent::PayinPartnerFee => P::PayinPartnerFee(typed(raw)?),
            WebhookEvent::TosAccept => P::TosAccept(typed(raw)?),
            WebhookEvent::LimitIncreaseNew => P::LimitIncreaseNew(typed(raw)?),
            WebhookEvent::LimitIncreaseUpdate => P::LimitIncreaseUpdate(typed(raw)?),
            WebhookEvent::VirtualAccountNew => P::VirtualAccountNew(typed(raw)?),
            WebhookEvent::VirtualAccountComplete => P::VirtualAccountComplete(typed(raw)?),
            WebhookEvent::TransferNew => P::TransferNew(typed(raw)?),
            WebhookEvent::TransferUpdate => P::TransferUpdate(typed(raw)?),
            WebhookEvent::TransferComplete => P::TransferComplete(typed(raw)?),
            WebhookEvent::WalletNew => P::WalletNew(typed(raw)?),
            WebhookEvent::WalletUpdate => P::WalletUpdate(typed(raw)?),
            WebhookEvent::WalletInbound => P::WalletInbound(typed(raw)?),
            WebhookEvent::PayableNew => P::PayableNew(typed(raw)?),
            WebhookEvent::PayableUpdate => P::PayableUpdate(typed(raw)?),
            WebhookEvent::PayableComplete => P::PayableComplete(typed(raw)?),
            _ => P::Unknown(raw),
        })
    }
}
