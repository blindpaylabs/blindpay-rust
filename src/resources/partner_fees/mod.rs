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
        let path = format!("/instances/{}/partner-fees", self.client.instance_id);
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
        let path = format!(
            "/instances/{}/partner-fees/{}",
            self.client.instance_id,
            encode_path_segment(id)
        );
        self.client.get(&path).await
    }

    /// Creates a partner fee.
    ///
    /// `POST /instances/{instance_id}/partner-fees`
    pub async fn create(&self, input: &CreatePartnerFeeInput) -> Result<PartnerFee> {
        let path = format!("/instances/{}/partner-fees", self.client.instance_id);
        self.client
            .request::<_, (), _>(Method::POST, &path, None, Some(input))
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
        let id = id.as_ref().trim();
        if id.is_empty() {
            return Err(Error::Config("partner fee id cannot be empty".to_string()));
        }
        let path = format!(
            "/instances/{}/partner-fees/{}",
            self.client.instance_id,
            encode_path_segment(id)
        );
        self.client
            .request::<_, (), ()>(Method::DELETE, &path, None, None)
            .await
    }
}
