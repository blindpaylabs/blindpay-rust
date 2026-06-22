//! Integration tests for the `partner_fees` resource, backed by a mock HTTP server.

use blindpay::{BlindPay, CreatePartnerFeeInput, Error};
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
async fn list_parses_response_and_sends_auth_headers() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/partner-fees"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "fe_123",
                "instance_id": "in_test",
                "name": "Display Name",
                "payout_percentage_fee": 100,
                "payout_flat_fee": 1000,
                "payin_percentage_fee": 50,
                "payin_flat_fee": 500,
                "virtual_account_set": false,
                "created_at": "2021-01-01T00:00:00Z",
                "updated_at": "2021-01-02T00:00:00Z"
            }
        ])))
        .mount(&server)
        .await;

    let fees = client(&server).await.partner_fees.list().await.unwrap();

    assert_eq!(fees.len(), 1);
    assert_eq!(fees[0].id, "fe_123");
    assert_eq!(fees[0].name, "Display Name");
    assert_eq!(fees[0].payout_flat_fee, 1000);
    assert_eq!(fees[0].virtual_account_set, Some(false));
    assert_eq!(fees[0].created_at.as_deref(), Some("2021-01-01T00:00:00Z"));
}

#[tokio::test]
async fn get_parses_create_out_shape() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/partner-fees/fe_123"))
        .and(header("authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "name": "Display Name",
            "payout_percentage_fee": 0,
            "payout_flat_fee": 0,
            "payin_percentage_fee": 0,
            "payin_flat_fee": 0
        })))
        .mount(&server)
        .await;

    let fee = client(&server)
        .await
        .partner_fees
        .get("fe_123")
        .await
        .unwrap();

    assert_eq!(fee.id, "fe_123");
    assert_eq!(fee.virtual_account_set, None);
    assert_eq!(fee.created_at, None);
}

#[tokio::test]
async fn create_sends_body_and_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/partner-fees"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(body_json(serde_json::json!({
            "name": "Display Name",
            "payout_percentage_fee": 100,
            "payout_flat_fee": 1000,
            "payin_percentage_fee": 50,
            "payin_flat_fee": 500
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "name": "Display Name",
            "payout_percentage_fee": 100,
            "payout_flat_fee": 1000,
            "payin_percentage_fee": 50,
            "payin_flat_fee": 500
        })))
        .mount(&server)
        .await;

    let input = CreatePartnerFeeInput::new("Display Name", 100, 1000, 50, 500);
    let fee = client(&server)
        .await
        .partner_fees
        .create(&input)
        .await
        .unwrap();

    assert_eq!(fee.id, "fe_123");
    assert_eq!(fee.payin_flat_fee, 500);
}

#[tokio::test]
async fn create_serializes_virtual_account_set_when_set() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/partner-fees"))
        .and(body_json(serde_json::json!({
            "name": "Display Name",
            "payout_percentage_fee": 0,
            "payout_flat_fee": 0,
            "payin_percentage_fee": 0,
            "payin_flat_fee": 0,
            "virtual_account_set": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "name": "Display Name",
            "payout_percentage_fee": 0,
            "payout_flat_fee": 0,
            "payin_percentage_fee": 0,
            "payin_flat_fee": 0
        })))
        .mount(&server)
        .await;

    let mut input = CreatePartnerFeeInput::new("Display Name", 0, 0, 0, 0);
    input.virtual_account_set = Some(true);
    let fee = client(&server)
        .await
        .partner_fees
        .create(&input)
        .await
        .unwrap();

    assert_eq!(fee.id, "fe_123");
}

#[tokio::test]
async fn delete_returns_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test/partner-fees/fe_123"))
        .and(header("authorization", "Bearer test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let result = client(&server)
        .await
        .partner_fees
        .delete("fe_123")
        .await
        .unwrap();

    assert!(result.success);
}

#[tokio::test]
async fn empty_id_fails_before_any_request() {
    let client = BlindPay::new("test-api-key", "in_test").unwrap();
    let err = client.partner_fees.get("  ").await.unwrap_err();
    assert!(matches!(err, Error::Config(_)));

    let err = client.partner_fees.delete("  ").await.unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[tokio::test]
async fn list_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/partner-fees"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "User not allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server).await.partner_fees.list().await.unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "User not allowed");
            assert!(api.raw_body.contains("User not allowed"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn create_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/partner-fees"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "virtual_account_partner_fee_already_exists"
        })))
        .mount(&server)
        .await;

    let mut input = CreatePartnerFeeInput::new("Display Name", 0, 0, 0, 0);
    input.virtual_account_set = Some(true);
    let err = client(&server)
        .await
        .partner_fees
        .create(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "virtual_account_partner_fee_already_exists");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn delete_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test/partner-fees/fe_123"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "user_not_allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .partner_fees
        .delete("fe_123")
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
async fn get_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/partner-fees/fe_123"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({ "message": "not found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .partner_fees
        .get("fe_123")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "not found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
