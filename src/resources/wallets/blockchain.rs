//! The `wallets.blockchain` sub-resource: blockchain wallets registered for a
//! customer.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

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

    /// Records whether a wallet is self-custodied (mandatory for Brazilian
    /// customers). It can be set only once: a second call returns
    /// `409 WALLETS_SELF_CUSTODY_ALREADY_SET`.
    ///
    /// `PATCH /instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}`
    pub async fn set_self_custody(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
        is_self_custody: bool,
    ) -> Result<BlockchainWallet> {
        self.set_self_custody_with_options(
            customer_id,
            id,
            is_self_custody,
            &RequestOptions::default(),
        )
        .await
    }

    /// Like [`Self::set_self_custody`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `PATCH /instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}`
    pub async fn set_self_custody_with_options(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
        is_self_custody: bool,
        options: &RequestOptions,
    ) -> Result<BlockchainWallet> {
        #[derive(serde::Serialize)]
        struct Body {
            is_self_custody: bool,
        }
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client
            .request_with::<_, (), _>(
                Method::PATCH,
                &path,
                None,
                Some(&Body { is_self_custody }),
                options,
            )
            .await
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
        self.create_with_address_with_options(customer_id, input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_with_address`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/blockchain-wallets`
    pub async fn create_with_address_with_options(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateBlockchainWalletWithAddressInput,
        options: &RequestOptions,
    ) -> Result<BlockchainWallet> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
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
        self.create_with_hash_with_options(customer_id, input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_with_hash`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/blockchain-wallets`
    pub async fn create_with_hash_with_options(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateBlockchainWalletWithHashInput,
        options: &RequestOptions,
    ) -> Result<BlockchainWallet> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
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
        self.delete_with_options(customer_id, id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::delete`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}`
    pub async fn delete_with_options(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}/blockchain-wallets/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client
            .request_with::<_, (), ()>(Method::DELETE, &path, None, None, options)
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
        self.create_asset_trustline_with_options(address, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_asset_trustline`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/create-asset-trustline`
    pub async fn create_asset_trustline_with_options(
        &self,
        address: impl Into<String>,
        options: &RequestOptions,
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
            .request_with::<_, (), _>(
                Method::POST,
                &path,
                None,
                Some(&Body {
                    address: address.into(),
                }),
                options,
            )
            .await
    }

    /// Mints sandbox USDB to a Stellar wallet.
    ///
    /// `POST /instances/{instance_id}/mint-usdb-stellar`
    pub async fn mint_usdb_stellar(&self, input: &MintUsdbStellarInput) -> Result<Success> {
        self.mint_usdb_stellar_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::mint_usdb_stellar`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/mint-usdb-stellar`
    pub async fn mint_usdb_stellar_with_options(
        &self,
        input: &MintUsdbStellarInput,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!("/instances/{}/mint-usdb-stellar", self.client.instance_id);
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Mints sandbox USDB to a Solana wallet.
    ///
    /// `POST /instances/{instance_id}/mint-usdb-solana`
    pub async fn mint_usdb_solana(
        &self,
        input: &MintUsdbSolanaInput,
    ) -> Result<MintUsdbSolanaResponse> {
        self.mint_usdb_solana_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::mint_usdb_solana`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/mint-usdb-solana`
    pub async fn mint_usdb_solana_with_options(
        &self,
        input: &MintUsdbSolanaInput,
        options: &RequestOptions,
    ) -> Result<MintUsdbSolanaResponse> {
        let path = format!("/instances/{}/mint-usdb-solana", self.client.instance_id);
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Prepares a Solana token-delegation transaction for signing.
    ///
    /// `POST /instances/{instance_id}/prepare-delegate-solana`
    pub async fn prepare_solana_delegation_transaction(
        &self,
        input: &PrepareSolanaDelegationInput,
    ) -> Result<PrepareSolanaDelegationResponse> {
        self.prepare_solana_delegation_transaction_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::prepare_solana_delegation_transaction`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/prepare-delegate-solana`
    pub async fn prepare_solana_delegation_transaction_with_options(
        &self,
        input: &PrepareSolanaDelegationInput,
        options: &RequestOptions,
    ) -> Result<PrepareSolanaDelegationResponse> {
        let path = format!(
            "/instances/{}/prepare-delegate-solana",
            self.client.instance_id
        );
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
            .await
    }
}
