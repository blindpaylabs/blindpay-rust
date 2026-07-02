//! The `tos` sub-resource: initiate a hosted terms-of-service flow.

use std::sync::Arc;

use crate::client::Inner;
use crate::error::Result;
use crate::resources::instances::types::{InitiateTosInput, InitiateTosResponse};

/// Handle for the terms-of-service sub-resource, reached via
/// [`Instances::tos`](crate::Instances).
///
#[derive(Clone)]
pub struct Tos {
    client: Arc<Inner>,
}

impl Tos {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Initiates a hosted terms-of-service flow and returns the URL to send the
    /// customer to.
    ///
    /// `POST /e/instances/{instance_id}/tos`
    pub async fn initiate(&self, body: &InitiateTosInput) -> Result<InitiateTosResponse> {
        let path = format!("/e/instances/{}/tos", self.client.instance_id);
        self.client
            .request(reqwest::Method::POST, &path, None::<&()>, Some(body))
            .await
    }
}
