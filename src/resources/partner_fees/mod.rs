//! The `partner_fees` resource: configure payin/payout fees that are split out to
//! a partner.

mod types;

pub use types::{CreatePartnerFeeInput, PartnerFee};

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::{Error, Result};
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

/// Handle for the `partner_fees` resource.
///
/// Obtained from the `partner_fees` field of a [`BlindPay`](crate::BlindPay)
/// client.
#[derive(Clone)]
pub struct PartnerFees {
    client: Arc<Inner>,
}

impl PartnerFees {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists every partner fee configured for the instance.
    ///
    /// `GET /instances/{instance_id}/partner-fees`
    pub async fn list(&self) -> Result<Vec<PartnerFee>> {
        let path = self.client.instance_path("/partner-fees");
        self.client.get(&path).await
    }

    /// Retrieves a single partner fee by id.
    ///
    /// `GET /instances/{instance_id}/partner-fees/{id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if `id` is empty.
    pub async fn get(&self, id: impl AsRef<str>) -> Result<PartnerFee> {
        let id = id.as_ref().trim();
        if id.is_empty() {
            return Err(Error::Config("partner fee id cannot be empty".to_string()));
        }
        let path = self
            .client
            .instance_path(&format!("/partner-fees/{}", encode_path_segment(id)));
        self.client.get(&path).await
    }

    /// Creates a partner fee.
    ///
    /// `POST /instances/{instance_id}/partner-fees`
    pub async fn create(&self, input: &CreatePartnerFeeInput) -> Result<PartnerFee> {
        self.create_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/partner-fees`
    pub async fn create_with_options(
        &self,
        input: &CreatePartnerFeeInput,
        options: &RequestOptions,
    ) -> Result<PartnerFee> {
        let path = self.client.instance_path("/partner-fees");
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(input), options)
            .await
    }

    /// Deletes a partner fee by id.
    ///
    /// `DELETE /instances/{instance_id}/partner-fees/{id}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if `id` is empty.
    pub async fn delete(&self, id: impl AsRef<str>) -> Result<Success> {
        self.delete_with_options(id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::delete`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{instance_id}/partner-fees/{id}`
    pub async fn delete_with_options(
        &self,
        id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Success> {
        let id = id.as_ref().trim();
        if id.is_empty() {
            return Err(Error::Config("partner fee id cannot be empty".to_string()));
        }
        let path = self
            .client
            .instance_path(&format!("/partner-fees/{}", encode_path_segment(id)));
        self.client
            .request_with::<_, (), ()>(Method::DELETE, &path, None, None, options)
            .await
    }
}
