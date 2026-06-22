//! The `webhook_endpoints` sub-resource: manage webhook delivery endpoints.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::resources::instances::types::{
    CreateWebhookEndpointInput, CreateWebhookEndpointResponse, PortalAccess, WebhookEndpoint,
    WebhookEndpointSecret,
};

/// Handle for the webhook-endpoints sub-resource, reached via
/// [`Instances::webhook_endpoints`](crate::Instances).
///
#[derive(Clone)]
pub struct WebhookEndpoints {
    client: Arc<Inner>,
}

impl WebhookEndpoints {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists the instance's webhook endpoints.
    ///
    /// `GET /instances/{instance_id}/webhook-endpoints`
    pub async fn list(&self) -> Result<Vec<WebhookEndpoint>> {
        let path = format!("/instances/{}/webhook-endpoints", self.client.instance_id);
        self.client.get(&path).await
    }

    /// Creates a webhook endpoint subscribed to the given events.
    ///
    /// `POST /instances/{instance_id}/webhook-endpoints`
    pub async fn create(
        &self,
        body: &CreateWebhookEndpointInput,
    ) -> Result<CreateWebhookEndpointResponse> {
        let path = format!("/instances/{}/webhook-endpoints", self.client.instance_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(body))
            .await
    }

    /// Deletes a webhook endpoint.
    ///
    /// `DELETE /instances/{instance_id}/webhook-endpoints/{id}`
    pub async fn delete(&self, id: impl AsRef<str>) -> Result<Success> {
        let path = format!(
            "/instances/{}/webhook-endpoints/{}",
            self.client.instance_id,
            encode_path_segment(id.as_ref())
        );
        self.client
            .request(Method::DELETE, &path, None::<&()>, None::<&()>)
            .await
    }

    /// Retrieves the signing secret for a webhook endpoint.
    ///
    /// `GET /instances/{instance_id}/webhook-endpoints/{id}/secret`
    pub async fn get_secret(&self, id: impl AsRef<str>) -> Result<WebhookEndpointSecret> {
        let path = format!(
            "/instances/{}/webhook-endpoints/{}/secret",
            self.client.instance_id,
            encode_path_segment(id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Retrieves a URL granting access to the Svix consumer portal.
    ///
    /// `GET /instances/{instance_id}/webhook-endpoints/portal-access`
    pub async fn get_portal_access_url(&self) -> Result<PortalAccess> {
        let path = format!(
            "/instances/{}/webhook-endpoints/portal-access",
            self.client.instance_id
        );
        self.client.get(&path).await
    }
}
