//! The `wallets.blockchain` sub-resource: blockchain wallets registered for a
//! customer.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;

use super::types::{
    BlockchainWallet, BlockchainWalletMessage, CreateAssetTrustlineResponse,
    CreateBlockchainWalletWithAddressInput, CreateBlockchainWalletWithHashInput,
    MintUsdbSolanaInput, MintUsdbSolanaResponse, MintUsdbStellarInput,
    PrepareSolanaDelegationInput, PrepareSolanaDelegationResponse,
};

/// Handle for the `wallets.blockchain` sub-resource.
///
/// Reached via the `blockchain` field of the
/// [`Wallets`](crate::resources::wallets::Wallets) handle.
#[derive(Clone)]
pub struct BlockchainWallets {
    client: Arc<Inner>,
}

impl BlockchainWallets {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists the blockchain wallets registered for a customer.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/blockchain-wallets`
    pub async fn list(&self, customer_id: impl AsRef<str>) -> Result<Vec<BlockchainWallet>> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client.get(&path).await
    }

    /// Returns the message a customer must sign to register a blockchain wallet
    /// by signature.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/blockchain-wallets/sign-message`
    pub async fn get_wallet_message(
        &self,
        customer_id: impl AsRef<str>,
    ) -> Result<BlockchainWalletMessage> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets/sign-message",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client.get(&path).await
    }

    /// Retrieves a single blockchain wallet by ID.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}`
    pub async fn get(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<BlockchainWallet> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client.get(&path).await
    }

    /// Registers an account-abstraction blockchain wallet by address.
    ///
    /// Sets `is_account_abstraction: true`. Use this for non-EVM networks
    /// (Stellar, Solana, Tron) and for EVM wallets you register by address.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/blockchain-wallets`
    pub async fn create_with_address(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateBlockchainWalletWithAddressInput,
    ) -> Result<BlockchainWallet> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client
            .request::<_, (), _>(Method::POST, &path, None, Some(input))
            .await
    }

    /// Registers a blockchain wallet by signature (EVM networks only).
    ///
    /// Sets `is_account_abstraction: false`.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/blockchain-wallets`
    pub async fn create_with_hash(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateBlockchainWalletWithHashInput,
    ) -> Result<BlockchainWallet> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client
            .request::<_, (), _>(Method::POST, &path, None, Some(input))
            .await
    }

    /// Deletes a blockchain wallet.
    ///
    /// `DELETE /instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}`
    pub async fn delete(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client
            .request::<_, (), ()>(Method::DELETE, &path, None, None)
            .await
    }

    /// Creates a Stellar asset trustline for a wallet address, returning the
    /// unsigned transaction envelope (XDR) to sign and submit.
    ///
    /// `POST /instances/{instance_id}/create-asset-trustline`
    pub async fn create_asset_trustline(
        &self,
        address: impl Into<String>,
    ) -> Result<CreateAssetTrustlineResponse> {
        #[derive(serde::Serialize)]
        struct Body {
            address: String,
        }
        let path = format!(
            "/instances/{}/create-asset-trustline",
            self.client.instance_id
        );
        self.client
            .request::<_, (), _>(
                Method::POST,
                &path,
                None,
                Some(&Body {
                    address: address.into(),
                }),
            )
            .await
    }

    /// Mints sandbox USDB to a Stellar wallet.
    ///
    /// `POST /instances/{instance_id}/mint-usdb-stellar`
    pub async fn mint_usdb_stellar(&self, input: &MintUsdbStellarInput) -> Result<Success> {
        let path = format!("/instances/{}/mint-usdb-stellar", self.client.instance_id);
        self.client
            .request::<_, (), _>(Method::POST, &path, None, Some(input))
            .await
    }

    /// Mints sandbox USDB to a Solana wallet.
    ///
    /// `POST /instances/{instance_id}/mint-usdb-solana`
    pub async fn mint_usdb_solana(
        &self,
        input: &MintUsdbSolanaInput,
    ) -> Result<MintUsdbSolanaResponse> {
        let path = format!("/instances/{}/mint-usdb-solana", self.client.instance_id);
        self.client
            .request::<_, (), _>(Method::POST, &path, None, Some(input))
            .await
    }

    /// Prepares a Solana token-delegation transaction for signing.
    ///
    /// `POST /instances/{instance_id}/prepare-delegate-solana`
    pub async fn prepare_solana_delegation_transaction(
        &self,
        input: &PrepareSolanaDelegationInput,
    ) -> Result<PrepareSolanaDelegationResponse> {
        let path = format!(
            "/instances/{}/prepare-delegate-solana",
            self.client.instance_id
        );
        self.client
            .request::<_, (), _>(Method::POST, &path, None, Some(input))
            .await
    }
}
