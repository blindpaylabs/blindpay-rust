//! The `wallets.offramp` sub-resource: offramp wallets attached to a customer's
//! bank account.
//!
//! Although offramp wallets live under a bank account on the wire, they are
//! exposed here (under `wallets`) for parity with the other SDKs; every method
//! takes both a `customer_id` and a `bank_account_id`.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

use super::types::{CreateOfframpWalletInput, OfframpWallet, UpdateOfframpWalletInput};

/// Handle for the `wallets.offramp` sub-resource.
///
/// Reached via the `offramp` field of the
/// [`Wallets`](crate::resources::wallets::Wallets) handle.
#[derive(Clone)]
pub struct OfframpWallets {
    client: Arc<Inner>,
}

impl OfframpWallets {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists the offramp wallets attached to a bank account.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets`
    pub async fn list(
        &self,
        customer_id: impl AsRef<str>,
        bank_account_id: impl AsRef<str>,
    ) -> Result<Vec<OfframpWallet>> {
        let path = self.client.instance_path(&format!(
            "/customers/{}/bank-accounts/{}/offramp-wallets",
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(bank_account_id.as_ref())
        ));
        self.client.get(&path).await
    }

    /// Creates an offramp wallet on a bank account.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets`
    pub async fn create(
        &self,
        customer_id: impl AsRef<str>,
        bank_account_id: impl AsRef<str>,
        input: &CreateOfframpWalletInput,
    ) -> Result<OfframpWallet> {
        self.create_with_options(
            customer_id,
            bank_account_id,
            input,
            &RequestOptions::default(),
        )
        .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets`
    pub async fn create_with_options(
        &self,
        customer_id: impl AsRef<str>,
        bank_account_id: impl AsRef<str>,
        input: &CreateOfframpWalletInput,
        options: &RequestOptions,
    ) -> Result<OfframpWallet> {
        let path = self.client.instance_path(&format!(
            "/customers/{}/bank-accounts/{}/offramp-wallets",
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(bank_account_id.as_ref())
        ));
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Retrieves a single offramp wallet by ID.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}`
    pub async fn get(
        &self,
        customer_id: impl AsRef<str>,
        bank_account_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<OfframpWallet> {
        let path = self.client.instance_path(&format!(
            "/customers/{}/bank-accounts/{}/offramp-wallets/{}",
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(bank_account_id.as_ref()),
            encode_path_segment(id.as_ref())
        ));
        self.client.get(&path).await
    }

    /// Updates an offramp wallet's partner fee (set, change or clear; see
    /// [`UpdateOfframpWalletInput`]).
    ///
    /// `PUT /instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}`
    pub async fn update(
        &self,
        customer_id: impl AsRef<str>,
        bank_account_id: impl AsRef<str>,
        id: impl AsRef<str>,
        input: &UpdateOfframpWalletInput,
    ) -> Result<Success> {
        self.update_with_options(
            customer_id,
            bank_account_id,
            id,
            input,
            &RequestOptions::default(),
        )
        .await
    }

    /// Like [`Self::update`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `PUT /instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}`
    pub async fn update_with_options(
        &self,
        customer_id: impl AsRef<str>,
        bank_account_id: impl AsRef<str>,
        id: impl AsRef<str>,
        input: &UpdateOfframpWalletInput,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = self.client.instance_path(&format!(
            "/customers/{}/bank-accounts/{}/offramp-wallets/{}",
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(bank_account_id.as_ref()),
            encode_path_segment(id.as_ref())
        ));
        self.client
            .request_with::<_, (), _>(Method::PUT, &path, None, Some(input), options)
            .await
    }
}
