//! Integration tests for the `transfers` resource, backed by a mock HTTP server.

use blindpay::{
    BlindPay, CreateTransferInput, CreateTransferQuoteInput, CurrencyType, Error, Network,
    PaginationParams, Token, TrackingStatus, TransactionStatus,
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Builds a client pointed at the mock server. The mock's `/v1` prefix mirrors
/// the production base URL's path.
async fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test-api-key", "in_test")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .expect("client should build")
}

fn tracking_step() -> serde_json::Value {
    serde_json::json!({
        "step": "processing",
        "transaction_hash": null,
        "gas_fee": null,
        "completed_at": null,
        "error_message": null
    })
}

fn tracking_monitoring() -> serde_json::Value {
    serde_json::json!({
        "step": "processing",
        "blockchain_screening": null,
        "risk_score": null,
        "completed_at": null
    })
}

fn transfer_body() -> serde_json::Value {
    serde_json::json!({
        "id": "tr_000000000000",
        "status": "processing",
        "transfer_quote_id": "qu_000000000000",
        "instance_id": "in_test",
        "tracking_transaction_monitoring": tracking_monitoring(),
        "tracking_paymaster": tracking_step(),
        "tracking_bridge_swap": tracking_step(),
        "tracking_complete": tracking_step(),
        "tracking_partner_fee": tracking_step(),
        "created_at": "2025-01-01T00:00:00.000Z",
        "updated_at": "2025-01-01T00:00:00.000Z",
        "image_url": null,
        "first_name": "John",
        "last_name": "Doe",
        "legal_name": null,
        "wallet_id": "bl_000000000000",
        "sender_token": "USDC",
        "sender_amount": 1000,
        "receiver_amount": 1000,
        "receiver_network": "polygon",
        "receiver_token": "USDC",
        "receiver_wallet_address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
        "partner_fee_amount": null,
        "external_id": "ext_1",
        "receiver_id": "re_000000000000",
        "address": "0xAbC0000000000000000000000000000000000000",
        "network": "polygon"
    })
}

#[tokio::test]
async fn quotes_create_sends_body_and_parses_response() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/transfer-quotes"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .and(body_json(serde_json::json!({
            "wallet_id": "bl_000000000000",
            "amount_reference": "sender",
            "request_amount": 1000,
            "sender_token": "USDC",
            "receiver_wallet_address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
            "receiver_token": "USDC",
            "receiver_network": "polygon",
            "cover_fees": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "qu_000000000000",
            "expires_at": 1712958191,
            "commercial_quotation": 100,
            "blindpay_quotation": 100,
            "receiver_amount": 1000,
            "sender_amount": 1000,
            "partner_fee_amount": null,
            "flat_fee": 0
        })))
        .mount(&server)
        .await;

    let input = CreateTransferQuoteInput::new(
        "bl_000000000000",
        CurrencyType::Sender,
        1000,
        Token::Usdc,
        "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
        Token::Usdc,
        Network::Polygon,
    )
    .cover_fees(true);

    let quote = client(&server)
        .await
        .transfers
        .quotes
        .create(&input)
        .await
        .unwrap();

    assert_eq!(quote.id, "qu_000000000000");
    assert_eq!(quote.sender_amount, 1000);
    assert_eq!(quote.receiver_amount, 1000);
    assert_eq!(quote.flat_fee, 0);
    assert_eq!(quote.expires_at, Some(1712958191));
    assert_eq!(quote.partner_fee_amount, None);
}

#[tokio::test]
async fn create_sends_quote_id_and_parses_tracking() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/transfers"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(body_json(
            serde_json::json!({ "transfer_quote_id": "qu_000000000000" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "tr_000000000000",
            "status": "processing",
            "tracking_bridge_swap": tracking_step(),
            "tracking_complete": tracking_step(),
            "tracking_paymaster": tracking_step(),
            "tracking_transaction_monitoring": tracking_monitoring(),
            "tracking_partner_fee": tracking_step()
        })))
        .mount(&server)
        .await;

    let created = client(&server)
        .await
        .transfers
        .create(&CreateTransferInput::new("qu_000000000000"))
        .await
        .unwrap();

    assert_eq!(created.id, "tr_000000000000");
    assert_eq!(created.status, TransactionStatus::Processing);
    assert_eq!(created.tracking_complete.step, TrackingStatus::Processing);
}

#[tokio::test]
async fn list_sends_pagination_and_parses_paginated_envelope() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/transfers"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [transfer_body()],
            "pagination": {
                "has_more": true,
                "next_page": "tr_000000000000",
                "prev_page": null
            }
        })))
        .mount(&server)
        .await;

    let params = PaginationParams::new().limit(blindpay::Limit::Fifty);
    let resp = client(&server).await.transfers.list(&params).await.unwrap();

    assert_eq!(resp.data().len(), 1);
    assert_eq!(resp.data()[0].id, "tr_000000000000");
    assert_eq!(resp.data()[0].sender_token, Token::Usdc);
    assert_eq!(resp.data()[0].receiver_network, Network::Polygon);
    assert_eq!(resp.data()[0].first_name.as_deref(), Some("John"));
    assert!(resp.pagination().unwrap().has_more);
    assert_eq!(
        resp.pagination().unwrap().next_page.as_deref(),
        Some("tr_000000000000")
    );
}

#[tokio::test]
async fn get_parses_full_transfer() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/transfers/tr_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(transfer_body()))
        .mount(&server)
        .await;

    let transfer = client(&server)
        .await
        .transfers
        .get("tr_000000000000")
        .await
        .unwrap();

    assert_eq!(transfer.id, "tr_000000000000");
    assert_eq!(transfer.status, TransactionStatus::Processing);
    assert_eq!(transfer.transfer_quote_id, "qu_000000000000");
    assert_eq!(transfer.receiver_id, "re_000000000000");
    assert_eq!(transfer.external_id.as_deref(), Some("ext_1"));
    assert_eq!(transfer.partner_fee_amount, None);
    assert_eq!(
        transfer.tracking_transaction_monitoring.step,
        TrackingStatus::Processing
    );
}

#[tokio::test]
async fn get_track_uses_unauth_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/e/transfers/tr_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(transfer_body()))
        .mount(&server)
        .await;

    let transfer = client(&server)
        .await
        .transfers
        .get_track("tr_000000000000")
        .await
        .unwrap();

    assert_eq!(transfer.id, "tr_000000000000");
}

#[tokio::test]
async fn empty_id_fails_before_any_request() {
    let client = BlindPay::new("test-api-key", "in_test").unwrap();
    let err = client.transfers.get("   ").await.unwrap_err();
    assert!(matches!(err, Error::Config(_)));

    let err = client.transfers.get_track("").await.unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[tokio::test]
async fn create_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/transfers"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "transfer_quote_expired" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .transfers
        .create(&CreateTransferInput::new("qu_000000000000"))
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "transfer_quote_expired");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn quotes_create_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/transfer-quotes"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "kyc_not_approved" })),
        )
        .mount(&server)
        .await;

    let input = CreateTransferQuoteInput::new(
        "bl_000000000000",
        CurrencyType::Sender,
        1000,
        Token::Usdc,
        "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
        Token::Usdc,
        Network::Polygon,
    );

    let err = client(&server)
        .await
        .transfers
        .quotes
        .create(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "kyc_not_approved");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
