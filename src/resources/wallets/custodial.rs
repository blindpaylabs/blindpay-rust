//! The `wallets.custodial` sub-resource: BlindPay-custodied (Circle) wallets
//! for a customer.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

use super::types::{CreateWalletInput, Wallet, WalletBalance};

/// Handle for the `wallets.custodial` sub-resource.
///
/// Reached via the `custodial` field of the
/// [`Wallets`](crate::resources::wallets::Wallets) handle.
#[derive(Clone)]
pub struct CustodialWallets {
    client: Arc<Inner>,
}

impl CustodialWallets {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists the custodial wallets for a customer.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/wallets`
    pub async fn list(&self, customer_id: impl AsRef<str>) -> Result<Vec<Wallet>> {
        let path = format!(
            "/instances/{}/customers/{}/wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client.get(&path).await
    }

    /// Retrieves a single custodial wallet by ID.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/wallets/{id}`
    pub async fn get(&self, customer_id: impl AsRef<str>, id: impl AsRef<str>) -> Result<Wallet> {
        let path = format!(
            "/instances/{}/customers/{}/wallets/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client.get(&path).await
    }

    /// Retrieves the per-token balance of a custodial wallet.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/wallets/{id}/balance`
    pub async fn get_balance(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<WalletBalance> {
        let path = format!(
            "/instances/{}/customers/{}/wallets/{}/balance",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client.get(&path).await
    }

    /// Creates a custodial wallet for a customer.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/wallets`
    pub async fn create(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateWalletInput,
    ) -> Result<Wallet> {
        self.create_with_options(customer_id, input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/wallets`
    pub async fn create_with_options(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateWalletInput,
        options: &RequestOptions,
    ) -> Result<Wallet> {
        let path = format!(
            "/instances/{}/customers/{}/wallets",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
        );
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Deletes a custodial wallet.
    ///
    /// `DELETE /instances/{instance_id}/customers/{customer_id}/wallets/{id}`
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
    /// `DELETE /instances/{instance_id}/customers/{customer_id}/wallets/{id}`
    pub async fn delete_with_options(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}/wallets/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref()),
        );
        self.client
            .request_with::<_, (), ()>(Method::DELETE, &path, None, None, options)
            .await
    }
}
