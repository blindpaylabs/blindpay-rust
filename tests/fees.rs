//! Integration tests for the `fees` resource, backed by a mock HTTP server.

use blindpay::{BlindPay, Error};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Builds a client pointed at the mock server. The mock's `/v1` prefix mirrors
/// the production base URL's path.
async fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test-api-key", "in_test")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .expect("client should build")
}

fn fee_options(flat: i64) -> serde_json::Value {
    serde_json::json!({
        "payin_flat": flat,
        "payin_percentage": 50,
        "payout_flat": flat,
        "payout_percentage": 50
    })
}

#[tokio::test]
async fn get_parses_response_and_sends_auth_headers() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/billing/fees"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "ach": fee_options(40),
            "domestic_wire": fee_options(40),
            "rtp": fee_options(40),
            "international_swift": fee_options(40),
            "pix": fee_options(10),
            "pix_safe": fee_options(10),
            "ted": fee_options(15),
            "ach_colombia": fee_options(20),
            "transfers_3": fee_options(20),
            "spei": fee_options(25),
            "sepa": fee_options(30),
            "tron": fee_options(5),
            "ethereum": fee_options(5),
            "polygon": fee_options(5),
            "base": fee_options(5),
            "arbitrum": fee_options(5),
            "stellar": fee_options(5),
            "solana": fee_options(5),
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-02T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let fee = client(&server).await.fees.get().await.unwrap();

    assert_eq!(fee.id, "fe_123");
    assert_eq!(fee.instance_id, "in_test");
    assert_eq!(fee.ach.payin_flat, 40);
    assert_eq!(fee.pix.payin_flat, 10);
    assert_eq!(fee.sepa.payout_flat, 30);
    assert_eq!(fee.solana.payin_percentage, 50);
    assert_eq!(fee.ted.as_ref().unwrap().payin_flat, 15);
    assert_eq!(fee.created_at, "2024-01-01T00:00:00.000Z");
}

#[tokio::test]
async fn get_handles_absent_ted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/billing/fees"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "ach": fee_options(40),
            "domestic_wire": fee_options(40),
            "rtp": fee_options(40),
            "international_swift": fee_options(40),
            "pix": fee_options(10),
            "pix_safe": fee_options(10),
            "ach_colombia": fee_options(20),
            "transfers_3": fee_options(20),
            "spei": fee_options(25),
            "sepa": fee_options(30),
            "tron": fee_options(5),
            "ethereum": fee_options(5),
            "polygon": fee_options(5),
            "base": fee_options(5),
            "arbitrum": fee_options(5),
            "stellar": fee_options(5),
            "solana": fee_options(5),
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-02T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let fee = client(&server).await.fees.get().await.unwrap();
    assert!(fee.ted.is_none());
}

#[tokio::test]
async fn get_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/billing/fees"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "User not allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server).await.fees.get().await.unwrap_err();

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
async fn get_not_found_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/billing/fees"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "message": "Instance not found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server).await.fees.get().await.unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "Instance not found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
