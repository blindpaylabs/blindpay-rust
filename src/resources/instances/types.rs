//! Request and response types for the `instances` resource and its
//! `tos` / `webhook_endpoints` sub-resources.

use serde::{Deserialize, Serialize};

use crate::common::WebhookEvent;
use crate::common::open_enum;

open_enum! {
    /// A member's role within an instance.
    pub enum UserRole {
        /// Instance owner.
        Owner => "owner",
        /// Administrator.
        Admin => "admin",
        /// Finance.
        Finance => "finance",
        /// Checker.
        Checker => "checker",
        /// Operations.
        Operations => "operations",
        /// Developer.
        Developer => "developer",
        /// Viewer.
        Viewer => "viewer",
    }
}

/// A member of an instance, returned by `Instances::get_members`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Member {
    /// Member (user) identifier, e.g. `"us_000000000000"`.
    pub id: String,
    /// Email address.
    pub email: String,
    /// First name.
    pub first_name: String,
    /// Middle name, when present.
    #[serde(default)]
    pub middle_name: Option<String>,
    /// Last name.
    pub last_name: String,
    /// Avatar image URL.
    pub image_url: String,
    /// Whether the member has registered a passkey.
    #[serde(default)]
    pub has_passkey: Option<bool>,
    /// The member's role within the instance.
    pub role: UserRole,
    /// Creation timestamp (ISO 8601).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Body for `Instances::update`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateInstanceInput {
    /// Display name for the instance (required).
    pub name: String,
    /// URL customers are redirected to after accepting an invite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_invite_redirect_url: Option<String>,
    /// Whether email notifications are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_notifications: Option<bool>,
    /// Whether a passkey is required for members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_passkey: Option<bool>,
    /// Logo shown in hosted flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// Accent colour for hosted flows (hex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    /// When non-empty, RFI emails go only to these addresses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compliance_emails: Option<Vec<String>>,
    /// Link to your own terms of service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_tos_link: Option<String>,
    /// Screenshot URL of your terms of service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_tos_screenshot: Option<String>,
    /// Whether customers receive RFI emails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customer_rfi_emails_enabled: Option<bool>,
    /// Accept the partner ToS terms (only `true` is accepted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_tos_terms_accepted: Option<bool>,
}

impl UpdateInstanceInput {
    /// Creates an update body with only the required `name` field set.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            customer_invite_redirect_url: None,
            email_notifications: None,
            require_passkey: None,
            logo_url: None,
            accent_color: None,
            compliance_emails: None,
            partner_tos_link: None,
            partner_tos_screenshot: None,
            customer_rfi_emails_enabled: None,
            partner_tos_terms_accepted: None,
        }
    }

    /// Sets `logo_url`.
    pub fn logo_url(mut self, logo_url: impl Into<String>) -> Self {
        self.logo_url = Some(logo_url.into());
        self
    }

    /// Sets `accent_color`.
    pub fn accent_color(mut self, accent_color: impl Into<String>) -> Self {
        self.accent_color = Some(accent_color.into());
        self
    }

    /// Sets `compliance_emails`.
    pub fn compliance_emails(mut self, compliance_emails: Vec<String>) -> Self {
        self.compliance_emails = Some(compliance_emails);
        self
    }

    /// Sets `partner_tos_link`.
    pub fn partner_tos_link(mut self, partner_tos_link: impl Into<String>) -> Self {
        self.partner_tos_link = Some(partner_tos_link.into());
        self
    }

    /// Sets `partner_tos_screenshot`.
    pub fn partner_tos_screenshot(mut self, partner_tos_screenshot: impl Into<String>) -> Self {
        self.partner_tos_screenshot = Some(partner_tos_screenshot.into());
        self
    }

    /// Sets `customer_rfi_emails_enabled`.
    pub fn customer_rfi_emails_enabled(mut self, customer_rfi_emails_enabled: bool) -> Self {
        self.customer_rfi_emails_enabled = Some(customer_rfi_emails_enabled);
        self
    }

    /// Sets `partner_tos_terms_accepted`.
    pub fn partner_tos_terms_accepted(mut self, partner_tos_terms_accepted: bool) -> Self {
        self.partner_tos_terms_accepted = Some(partner_tos_terms_accepted);
        self
    }
}

/// Body for `Instances::initiate` terms of service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InitiateTosInput {
    /// Idempotency key (UUID) for the request (required).
    pub idempotency_key: String,
    /// The customer the terms apply to (prefix `re_`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    /// URL to redirect to after acceptance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
}

impl InitiateTosInput {
    /// Creates an initiate body with only the required `idempotency_key` set.
    pub fn new(idempotency_key: impl Into<String>) -> Self {
        Self {
            idempotency_key: idempotency_key.into(),
            customer_id: None,
            redirect_url: None,
        }
    }
}

/// Response from `Tos::initiate`: a hosted URL to complete the flow.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct InitiateTosResponse {
    /// The hosted terms-of-service URL.
    pub url: String,
}

/// A webhook endpoint, returned by `WebhookEndpoints::list`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WebhookEndpoint {
    /// Endpoint identifier, e.g. `"we_000000000000"`.
    pub id: String,
    /// The destination URL.
    pub url: String,
    /// The events this endpoint is subscribed to.
    #[serde(default)]
    pub events: Vec<WebhookEvent>,
    /// Timestamp of the last delivered event, when any.
    #[serde(default)]
    pub last_event_at: Option<String>,
    /// The owning instance identifier.
    pub instance_id: String,
    /// Creation timestamp (ISO 8601).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last update timestamp (ISO 8601).
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Body for `WebhookEndpoints::create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateWebhookEndpointInput {
    /// The destination URL (required).
    pub url: String,
    /// The events to subscribe to (required).
    pub events: Vec<WebhookEvent>,
}

impl CreateWebhookEndpointInput {
    /// Creates a webhook-endpoint body from a URL and a set of events.
    pub fn new(url: impl Into<String>, events: Vec<WebhookEvent>) -> Self {
        Self {
            url: url.into(),
            events,
        }
    }
}

/// Response carrying the id of a newly created webhook endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CreateWebhookEndpointResponse {
    /// The created endpoint's identifier, e.g. `"we_000000000000"`.
    pub id: String,
}

/// Response from `WebhookEndpoints::get_secret`: the signing secret.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WebhookEndpointSecret {
    /// The signing secret (`whsec_…`) used to verify webhook payloads.
    pub key: String,
}

/// Response from `WebhookEndpoints::get_portal_access_url`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PortalAccess {
    /// The Svix application portal URL.
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_role_round_trips() {
        assert_eq!(
            serde_json::to_string(&UserRole::Owner).unwrap(),
            "\"owner\""
        );
        assert_eq!(
            serde_json::from_str::<UserRole>("\"checker\"").unwrap(),
            UserRole::Checker
        );
        assert_eq!(
            serde_json::from_str::<UserRole>("\"superuser\"").unwrap(),
            UserRole::Unknown("superuser".to_string())
        );
    }

    #[test]
    fn update_instance_input_skips_absent_optionals() {
        let body = UpdateInstanceInput::new("Acme");
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            serde_json::json!({ "name": "Acme" })
        );

        let full = UpdateInstanceInput {
            customer_invite_redirect_url: Some("https://example.com".to_string()),
            email_notifications: Some(false),
            require_passkey: Some(true),
            ..UpdateInstanceInput::new("Acme")
        };
        assert_eq!(
            serde_json::to_value(&full).unwrap(),
            serde_json::json!({
                "name": "Acme",
                "customer_invite_redirect_url": "https://example.com",
                "email_notifications": false,
                "require_passkey": true
            })
        );
    }

    #[test]
    fn initiate_tos_input_skips_absent_optionals() {
        let body = InitiateTosInput::new("123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            serde_json::json!({ "idempotency_key": "123e4567-e89b-12d3-a456-426614174000" })
        );
    }
}
