//! Integration tests for the `instances` resource and its `tos` /
//! `webhook_endpoints` sub-resources, backed by a mock HTTP server.

use blindpay::{
    BlindPay, CreateWebhookEndpointInput, Error, InitiateTosInput, UpdateInstanceInput, UserRole,
    WebhookEvent,
};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Builds a client pointed at the mock server. The mock's `/v1` prefix mirrors
/// the production base URL's path.
async fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test-api-key", "in_test")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .expect("client should build")
}

#[tokio::test]
async fn get_members_parses_response_and_sends_auth_headers() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/members"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "us_000000000000",
                "email": "owner@example.com",
                "first_name": "Harry",
                "middle_name": "James",
                "last_name": "Potter",
                "image_url": "https://example.com/image.png",
                "has_passkey": true,
                "role": "owner",
                "created_at": "2024-01-01T00:00:00.000Z"
            },
            {
                "id": "us_000000000001",
                "email": "dev@example.com",
                "first_name": "Ron",
                "last_name": "Weasley",
                "image_url": "https://example.com/ron.png",
                "role": "developer",
                "created_at": "2024-01-02T00:00:00.000Z"
            }
        ])))
        .mount(&server)
        .await;

    let members = client(&server).await.instances.get_members().await.unwrap();

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].id, "us_000000000000");
    assert_eq!(members[0].middle_name.as_deref(), Some("James"));
    assert_eq!(members[0].has_passkey, Some(true));
    assert_eq!(members[0].role, UserRole::Owner);
    // Second member omits the nullish fields entirely.
    assert_eq!(members[1].middle_name, None);
    assert_eq!(members[1].has_passkey, None);
    assert_eq!(members[1].role, UserRole::Developer);
}

#[tokio::test]
async fn update_sends_body_and_parses_success() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/instances/in_test"))
        .and(body_json(serde_json::json!({
            "name": "Acme",
            "require_passkey": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let body = UpdateInstanceInput {
        require_passkey: Some(true),
        ..UpdateInstanceInput::new("Acme")
    };
    let res = client(&server).await.instances.update(&body).await.unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn delete_sends_delete_and_parses_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let res = client(&server).await.instances.delete().await.unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn update_member_role_sends_user_role_body() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/instances/in_test/members/us_000000000001"))
        .and(body_json(serde_json::json!({ "user_role": "finance" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .instances
        .update_member_role("us_000000000001", UserRole::Finance)
        .await
        .unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn delete_member_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test/members/us_000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .instances
        .delete_member("us_000000000001")
        .await
        .unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn migrate_ownership_sends_user_id_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/ownership"))
        .and(body_json(
            serde_json::json!({ "user_id": "us_000000000001" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .instances
        .migrate_ownership("us_000000000001")
        .await
        .unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn get_members_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/members"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .get_members()
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "user_not_allowed");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn update_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/instances/in_test"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .update(&UpdateInstanceInput::new("Acme"))
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "user_not_allowed");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn delete_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server).await.instances.delete().await.unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "user_not_allowed");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn update_member_role_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/instances/in_test/members/us_000000000001"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .update_member_role("us_000000000001", UserRole::Finance)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "user_not_allowed");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn delete_member_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test/members/us_000000000001"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .delete_member("us_000000000001")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "user_not_allowed");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn migrate_ownership_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/ownership"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .migrate_ownership("us_000000000001")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "user_not_allowed");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

// --- tos ---

#[tokio::test]
async fn tos_initiate_uses_external_path_and_returns_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/e/instances/in_test/tos"))
        .and(body_json(serde_json::json!({
            "idempotency_key": "123e4567-e89b-12d3-a456-426614174000",
            "receiver_id": "cus_123"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "https://tos.blindpay.com/abc"
        })))
        .mount(&server)
        .await;

    let body = InitiateTosInput {
        receiver_id: Some("cus_123".to_string()),
        ..InitiateTosInput::new("123e4567-e89b-12d3-a456-426614174000")
    };
    let res = client(&server)
        .await
        .instances
        .tos
        .initiate(&body)
        .await
        .unwrap();
    assert_eq!(res.url, "https://tos.blindpay.com/abc");
}

#[tokio::test]
async fn tos_initiate_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/e/instances/in_test/tos"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "idempotency_key_already_exists"
        })))
        .mount(&server)
        .await;

    let body = InitiateTosInput::new("123e4567-e89b-12d3-a456-426614174000");
    let err = client(&server)
        .await
        .instances
        .tos
        .initiate(&body)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "idempotency_key_already_exists");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

// --- webhook_endpoints ---

#[tokio::test]
async fn webhook_endpoints_list_parses_events() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/webhook-endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "we_000000000000",
                "url": "https://example.com/webhook",
                "events": ["payin.complete", "payout.partnerFee"],
                "last_event_at": "2024-01-01T00:00:00.000Z",
                "instance_id": "in_test",
                "created_at": "2024-01-01T00:00:00.000Z",
                "updated_at": "2024-01-02T00:00:00.000Z"
            }
        ])))
        .mount(&server)
        .await;

    let endpoints = client(&server)
        .await
        .instances
        .webhook_endpoints
        .list()
        .await
        .unwrap();

    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].id, "we_000000000000");
    assert_eq!(
        endpoints[0].events,
        vec![WebhookEvent::PayinComplete, WebhookEvent::PayoutPartnerFee]
    );
    assert_eq!(
        endpoints[0].last_event_at.as_deref(),
        Some("2024-01-01T00:00:00.000Z")
    );
}

#[tokio::test]
async fn webhook_endpoints_create_sends_body_and_returns_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/webhook-endpoints"))
        .and(body_json(serde_json::json!({
            "url": "https://example.com/webhook",
            "events": ["payin.complete"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "we_000000000000"
        })))
        .mount(&server)
        .await;

    let body = CreateWebhookEndpointInput::new(
        "https://example.com/webhook",
        vec![WebhookEvent::PayinComplete],
    );
    let res = client(&server)
        .await
        .instances
        .webhook_endpoints
        .create(&body)
        .await
        .unwrap();
    assert_eq!(res.id, "we_000000000000");
}

#[tokio::test]
async fn webhook_endpoints_delete_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/v1/instances/in_test/webhook-endpoints/we_000000000000",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .instances
        .webhook_endpoints
        .delete("we_000000000000")
        .await
        .unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn webhook_endpoints_get_secret_returns_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/webhook-endpoints/we_000000000000/secret",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "key": "whsec_000000000000"
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .instances
        .webhook_endpoints
        .get_secret("we_000000000000")
        .await
        .unwrap();
    assert_eq!(res.key, "whsec_000000000000");
}

#[tokio::test]
async fn webhook_endpoints_get_portal_access_url_returns_url() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/webhook-endpoints/portal-access",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "https://portal.svix.com/abc"
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .instances
        .webhook_endpoints
        .get_portal_access_url()
        .await
        .unwrap();
    assert_eq!(res.url, "https://portal.svix.com/abc");
}

#[tokio::test]
async fn webhook_endpoints_list_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/webhook-endpoints"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "message": "internal error"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .webhook_endpoints
        .list()
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 500);
            assert_eq!(api.message, "internal error");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn webhook_endpoints_create_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/webhook-endpoints"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "invalid_url"
        })))
        .mount(&server)
        .await;

    let body = CreateWebhookEndpointInput::new(
        "https://example.com/webhook",
        vec![WebhookEvent::PayinComplete],
    );
    let err = client(&server)
        .await
        .instances
        .webhook_endpoints
        .create(&body)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "invalid_url");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn webhook_endpoints_delete_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/v1/instances/in_test/webhook-endpoints/we_000000000000",
        ))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "message": "webhook_endpoint_not_found"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .webhook_endpoints
        .delete("we_000000000000")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "webhook_endpoint_not_found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn webhook_endpoints_get_secret_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/webhook-endpoints/we_000000000000/secret",
        ))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "message": "webhook_endpoint_not_found"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .webhook_endpoints
        .get_secret("we_000000000000")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "webhook_endpoint_not_found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn webhook_endpoints_get_portal_access_url_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/webhook-endpoints/portal-access",
        ))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "message": "internal error"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .instances
        .webhook_endpoints
        .get_portal_access_url()
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 500);
            assert_eq!(api.message, "internal error");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
