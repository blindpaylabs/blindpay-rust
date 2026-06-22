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
        let path = format!("/instances/{}", self.client.instance_id);
        self.client
            .request(Method::PUT, &path, None::<&()>, Some(body))
            .await
    }

    /// Deletes the instance.
    ///
    /// `DELETE /instances/{id}`
    pub async fn delete(&self) -> Result<Success> {
        let path = format!("/instances/{}", self.client.instance_id);
        self.client
            .request(Method::DELETE, &path, None::<&()>, None::<&()>)
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
            .request(Method::PUT, &path, None::<&()>, Some(&Body { user_role }))
            .await
    }

    /// Removes a member from the instance.
    ///
    /// `DELETE /instances/{id}/members/{user_id}`
    pub async fn delete_member(&self, user_id: impl AsRef<str>) -> Result<Success> {
        let path = format!(
            "/instances/{}/members/{}",
            self.client.instance_id,
            encode_path_segment(user_id.as_ref())
        );
        self.client
            .request(Method::DELETE, &path, None::<&()>, None::<&()>)
            .await
    }

    /// Transfers instance ownership to another current member.
    ///
    /// `POST /instances/{id}/ownership`
    pub async fn migrate_ownership(&self, user_id: impl Into<String>) -> Result<Success> {
        #[derive(Serialize)]
        struct Body {
            user_id: String,
        }

        let path = format!("/instances/{}/ownership", self.client.instance_id);
        self.client
            .request(
                Method::POST,
                &path,
                None::<&()>,
                Some(&Body {
                    user_id: user_id.into(),
                }),
            )
            .await
    }
}
