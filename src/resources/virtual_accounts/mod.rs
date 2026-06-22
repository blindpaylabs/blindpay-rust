//! The `virtual_accounts` resource: per-customer virtual bank accounts that
//! convert inbound USD deposits into stablecoin.

mod types;

pub use types::{
    BankingPartner, CreateVirtualAccountInput, SoleProprietorDocType, UpdateVirtualAccountInput,
    UsAddress, UsBankCoordinates, UsSwiftIntermediaryBank, UsVirtualAccountDetails, VirtualAccount,
    VirtualAccountWallet,
};

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;

/// Handle for the `virtual_accounts` resource.
///
/// Obtained from the `virtual_accounts` field of a [`BlindPay`](crate::BlindPay)
/// client.
#[derive(Clone)]
pub struct VirtualAccounts {
    client: Arc<Inner>,
}

impl VirtualAccounts {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists the virtual accounts for a customer.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/virtual-accounts`
    pub async fn list(&self, customer_id: impl AsRef<str>) -> Result<Vec<VirtualAccount>> {
        let path = format!(
            "/instances/{}/customers/{}/virtual-accounts",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Retrieves a single virtual account by id.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}`
    ///
    /// Returns `None` when the API responds with JSON `null` (no matching
    /// virtual account).
    pub async fn get(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
    ) -> Result<Option<VirtualAccount>> {
        let path = format!(
            "/instances/{}/customers/{}/virtual-accounts/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Creates a virtual account for a customer.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/virtual-accounts`
    pub async fn create(
        &self,
        customer_id: impl AsRef<str>,
        input: &CreateVirtualAccountInput,
    ) -> Result<VirtualAccount> {
        let path = format!(
            "/instances/{}/customers/{}/virtual-accounts",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request::<VirtualAccount, (), _>(Method::POST, &path, None, Some(input))
            .await
    }

    /// Updates the token and blockchain wallet of a virtual account.
    ///
    /// `PUT /instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}`
    pub async fn update(
        &self,
        customer_id: impl AsRef<str>,
        id: impl AsRef<str>,
        input: &UpdateVirtualAccountInput,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}/virtual-accounts/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref()),
            encode_path_segment(id.as_ref())
        );
        self.client
            .request::<Success, (), _>(Method::PUT, &path, None, Some(input))
            .await
    }
}
