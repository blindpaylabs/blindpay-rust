//! The `wallets` namespace: a grouping handle exposing three wallet
//! sub-resources — [`blockchain`](Wallets::blockchain) (customer-owned chain
//! wallets), [`custodial`](Wallets::custodial) (BlindPay-custodied Circle
//! wallets), and [`offramp`](Wallets::offramp) (offramp wallets on a bank
//! account).
//!
//! `Wallets` itself has no endpoints; use one of its sub-resource handles.

mod blockchain;
mod custodial;
mod offramp;
mod types;

pub use blockchain::BlockchainWallets;
pub use custodial::CustodialWallets;
pub use offramp::OfframpWallets;
pub use types::{
    BlockchainWallet, BlockchainWalletMessage, CreateAssetTrustlineResponse,
    CreateBlockchainWalletWithAddressInput, CreateBlockchainWalletWithHashInput,
    CreateOfframpWalletInput, CreateWalletInput, MintUsdbSolanaInput, MintUsdbSolanaResponse,
    MintUsdbStellarInput, OfframpWallet, PrepareSolanaDelegationInput,
    PrepareSolanaDelegationResponse, Wallet, WalletBalance, WalletTokenBalance,
};

use std::sync::Arc;

use crate::client::Inner;

/// Namespace handle for the `wallets` resources.
///
/// Obtained from the `wallets` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Wallets {
    /// Customer-owned blockchain wallets.
    pub blockchain: BlockchainWallets,
    /// BlindPay-custodied (Circle) wallets.
    pub custodial: CustodialWallets,
    /// Offramp wallets attached to a customer's bank account.
    pub offramp: OfframpWallets,
}

impl Wallets {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self {
            blockchain: BlockchainWallets::new(Arc::clone(&client)),
            custodial: CustodialWallets::new(Arc::clone(&client)),
            offramp: OfframpWallets::new(client),
        }
    }
}
