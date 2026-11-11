//! The `instances` resource: manage an instance, its members, ownership, terms
//! of service, and webhook endpoints.

mod tos;
mod types;
mod webhook_endpoints;

pub use tos::Tos;
pub use types::{
    CreateWebhookEndpointInput, CreateWebhookEndpointResponse, InitiateTosInput,
    InitiateTosResponse, Member, PortalAccess, UpdateInstanceInput, UserRole, WebhookEndpoint,
    WebhookEndpointSecret,
};
pub use webhook_endpoints::WebhookEndpoints;

use std::sync::Arc;

use reqwest::Method;
use serde::Serialize;

use crate::client::Inner;
use crate::common::Success;
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

/// Handle for the `instances` resource.
///
/// Obtained from the `instances` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Instances {
    client: Arc<Inner>,
    /// Terms-of-service sub-resource.
    pub tos: Tos,
    /// Webhook-endpoints sub-resource.
    pub webhook_endpoints: WebhookEndpoints,
}

impl Instances {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self {
            tos: Tos::new(Arc::clone(&client)),
            webhook_endpoints: WebhookEndpoints::new(Arc::clone(&client)),
            client,
        }
    }

    /// Lists the members of the instance.
    ///
    /// `GET /instances/{id}/members`
    pub async fn get_members(&self) -> Result<Vec<Member>> {
        let path = format!("/instances/{}/members", self.client.instance_id);
        self.client.get(&path).await
    }

    /// Updates the instance's settings.
    ///
    /// `PUT /instances/{id}`
    pub async fn update(&self, body: &UpdateInstanceInput) -> Result<Success> {
        self.update_with_options(body, &RequestOptions::default())
            .await
    }

    /// Like [`Self::update`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `PUT /instances/{id}`
    pub async fn update_with_options(
        &self,
        body: &UpdateInstanceInput,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!("/instances/{}", self.client.instance_id);
        self.client
            .request_with(Method::PUT, &path, None::<&()>, Some(body), options)
            .await
    }

    /// Deletes the instance.
    ///
    /// `DELETE /instances/{id}`
    pub async fn delete(&self) -> Result<Success> {
        self.delete_with_options(&RequestOptions::default()).await
    }

    /// Like [`Self::delete`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{id}`
    pub async fn delete_with_options(&self, options: &RequestOptions) -> Result<Success> {
        let path = format!("/instances/{}", self.client.instance_id);
        self.client
            .request_with(Method::DELETE, &path, None::<&()>, None::<&()>, options)
            .await
    }

    /// Updates a member's role.
    ///
    /// `PUT /instances/{id}/members/{user_id}`
    pub async fn update_member_role(
        &self,
        user_id: impl AsRef<str>,
        user_role: UserRole,
    ) -> Result<Success> {
        self.update_member_role_with_options(user_id, user_role, &RequestOptions::default())
            .await
    }

    /// Like [`Self::update_member_role`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `PUT /instances/{id}/members/{user_id}`
    pub async fn update_member_role_with_options(
        &self,
        user_id: impl AsRef<str>,
        user_role: UserRole,
        options: &RequestOptions,
    ) -> Result<Success> {
        #[derive(Serialize)]
        struct Body {
            user_role: UserRole,
        }

        let path = format!(
            "/instances/{}/members/{}",
            self.client.instance_id,
            encode_path_segment(user_id.as_ref())
        );
        self.client
            .request_with(
                Method::PUT,
                &path,
                None::<&()>,
                Some(&Body { user_role }),
                options,
            )
            .await
    }

    /// Removes a member from the instance.
    ///
    /// `DELETE /instances/{id}/members/{user_id}`
    pub async fn delete_member(&self, user_id: impl AsRef<str>) -> Result<Success> {
        self.delete_member_with_options(user_id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::delete_member`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{id}/members/{user_id}`
    pub async fn delete_member_with_options(
        &self,
        user_id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/members/{}",
            self.client.instance_id,
            encode_path_segment(user_id.as_ref())
        );
        self.client
            .request_with(Method::DELETE, &path, None::<&()>, None::<&()>, options)
            .await
    }

    /// Transfers instance ownership to another current member.
    ///
    /// `POST /instances/{id}/ownership`
    pub async fn migrate_ownership(&self, user_id: impl Into<String>) -> Result<Success> {
        self.migrate_ownership_with_options(user_id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::migrate_ownership`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{id}/ownership`
    pub async fn migrate_ownership_with_options(
        &self,
        user_id: impl Into<String>,
        options: &RequestOptions,
    ) -> Result<Success> {
        #[derive(Serialize)]
        struct Body {
            user_id: String,
        }

        let path = format!("/instances/{}/ownership", self.client.instance_id);
        self.client
            .request_with(
                Method::POST,
                &path,
                None::<&()>,
                Some(&Body {
                    user_id: user_id.into(),
                }),
                options,
            )
            .await
    }
}
