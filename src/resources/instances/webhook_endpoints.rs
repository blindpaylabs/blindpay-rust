//! The `webhook_endpoints` sub-resource: manage webhook delivery endpoints.

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;
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
        let path = self.client.instance_path("/webhook-endpoints");
        self.client.get(&path).await
    }

    /// Creates a webhook endpoint subscribed to the given events.
    ///
    /// `POST /instances/{instance_id}/webhook-endpoints`
    pub async fn create(
        &self,
        body: &CreateWebhookEndpointInput,
    ) -> Result<CreateWebhookEndpointResponse> {
        self.create_with_options(body, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/webhook-endpoints`
    pub async fn create_with_options(
        &self,
        body: &CreateWebhookEndpointInput,
        options: &RequestOptions,
    ) -> Result<CreateWebhookEndpointResponse> {
        let path = self.client.instance_path("/webhook-endpoints");
        self.client
            .request_with(Method::POST, &path, None::<&()>, Some(body), options)
            .await
    }

    /// Deletes a webhook endpoint.
    ///
    /// `DELETE /instances/{instance_id}/webhook-endpoints/{id}`
    pub async fn delete(&self, id: impl AsRef<str>) -> Result<Success> {
        self.delete_with_options(id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::delete`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{instance_id}/webhook-endpoints/{id}`
    pub async fn delete_with_options(
        &self,
        id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = self.client.instance_path(&format!(
            "/webhook-endpoints/{}",
            encode_path_segment(id.as_ref())
        ));
        self.client
            .request_with(Method::DELETE, &path, None::<&()>, None::<&()>, options)
            .await
    }

    /// Retrieves the signing secret for a webhook endpoint.
    ///
    /// `GET /instances/{instance_id}/webhook-endpoints/{id}/secret`
    pub async fn get_secret(&self, id: impl AsRef<str>) -> Result<WebhookEndpointSecret> {
        let path = self.client.instance_path(&format!(
            "/webhook-endpoints/{}/secret",
            encode_path_segment(id.as_ref())
        ));
        self.client.get(&path).await
    }

    /// Retrieves a URL granting access to the Svix consumer portal.
    ///
    /// `GET /instances/{instance_id}/webhook-endpoints/portal-access`
    pub async fn get_portal_access_url(&self) -> Result<PortalAccess> {
        let path = self
            .client
            .instance_path("/webhook-endpoints/portal-access");
        self.client.get(&path).await
    }
}
