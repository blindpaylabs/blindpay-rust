//! Integration tests for the `payouts` resource, backed by a mock HTTP server.

use blindpay::{
    AuthorizeStellarTokenInput, BlindPay, CreateEvmPayoutInput, CreateSolanaPayoutInput,
    CreateStellarPayoutInput, Error, Limit, ListPayoutsParams, PayoutCompleteStatus,
    PayoutPaymentProviderStatus, PayoutTrackingStep, ProviderName, Rail,
    SubmitPayoutDocumentsInput, Token, TransactionDocumentType, TransactionStatus,
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

fn create_out_body() -> serde_json::Value {
    serde_json::json!({
        "id": "pa_000000000000",
        "status": "processing",
        "sender_wallet_address": "0x123456789",
        "billing_fee_amount": 100,
        "transaction_fee_amount": 250,
        "partner_fee": -25000,
        "tracking_transaction": { "step": "processing", "transaction_hash": "0xabc" },
        "tracking_payment": { "step": "on_hold", "provider_name": "HSBC" },
        "tracking_complete": { "step": "pending_review" },
        "receiver_id": "re_000000000000",
        "bank_account_id": "ba_000000000000",
        "offramp_wallet_id": null
    })
}

#[tokio::test]
async fn create_evm_posts_quote_and_wallet_and_parses_response() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/evm"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .and(body_json(serde_json::json!({
            "quote_id": "qu_000000000000",
            "sender_wallet_address": "0x123456789"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_out_body()))
        .mount(&server)
        .await;

    let input = CreateEvmPayoutInput::new("qu_000000000000", "0x123456789");
    let payout = client(&server)
        .await
        .payouts
        .create_evm(&input)
        .await
        .unwrap();

    assert_eq!(payout.id, "pa_000000000000");
    assert_eq!(payout.status, TransactionStatus::Processing);
    assert_eq!(payout.sender_wallet_address, "0x123456789");
    assert_eq!(payout.billing_fee_amount, Some(100));
    assert_eq!(payout.partner_fee, Some(-25000));
    assert_eq!(payout.receiver_id.as_deref(), Some("re_000000000000"));
    assert_eq!(payout.bank_account_id.as_deref(), Some("ba_000000000000"));
    assert_eq!(
        payout.tracking_complete.unwrap().step,
        PayoutTrackingStep::PendingReview
    );
}

#[tokio::test]
async fn create_solana_includes_optional_signed_transaction() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/solana"))
        .and(body_json(serde_json::json!({
            "quote_id": "qu_000000000000",
            "sender_wallet_address": "9xQeWv...",
            "signed_transaction": "AAA...Zey8y0A"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_out_body()))
        .mount(&server)
        .await;

    let input = CreateSolanaPayoutInput::new("qu_000000000000", "9xQeWv...")
        .signed_transaction("AAA...Zey8y0A");
    let payout = client(&server)
        .await
        .payouts
        .create_solana(&input)
        .await
        .unwrap();

    assert_eq!(payout.id, "pa_000000000000");
}

#[tokio::test]
async fn create_stellar_omits_unset_signed_transaction() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/stellar"))
        .and(body_json(serde_json::json!({
            "quote_id": "qu_000000000000",
            "sender_wallet_address": "GABC..."
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_out_body()))
        .mount(&server)
        .await;

    let input = CreateStellarPayoutInput::new("qu_000000000000", "GABC...");
    let payout = client(&server)
        .await
        .payouts
        .create_stellar(&input)
        .await
        .unwrap();

    assert_eq!(payout.status, TransactionStatus::Processing);
}

#[tokio::test]
async fn authorize_stellar_token_returns_transaction_hash() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/stellar/authorize"))
        .and(body_json(serde_json::json!({
            "quote_id": "qu_000000000000",
            "sender_wallet_address": "GABC..."
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "transaction_hash": "0xdeadbeef"
        })))
        .mount(&server)
        .await;

    let input = AuthorizeStellarTokenInput::new("qu_000000000000", "GABC...");
    let res = client(&server)
        .await
        .payouts
        .authorize_stellar_token(&input)
        .await
        .unwrap();

    assert_eq!(res.transaction_hash, "0xdeadbeef");
}

#[tokio::test]
async fn submit_documents_posts_to_payout_id_and_omits_unset_description() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/payouts/pa_000000000000/documents",
        ))
        .and(body_json(serde_json::json!({
            "transaction_document_type": "invoice",
            "transaction_document_id": "INV-12345",
            "transaction_document_file": "https://example.com/document.pdf"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let input = SubmitPayoutDocumentsInput {
        transaction_document_type: TransactionDocumentType::Invoice,
        transaction_document_id: "INV-12345".to_string(),
        transaction_document_file: "https://example.com/document.pdf".to_string(),
        description: None,
    };
    let res = client(&server)
        .await
        .payouts
        .submit_documents("pa_000000000000", &input)
        .await
        .unwrap();

    assert!(res.success);
}

#[tokio::test]
async fn list_sends_pagination_and_rail_filter_and_parses_joined_payout() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payouts"))
        .and(query_param("limit", "50"))
        .and(query_param("payment_method", "pix"))
        .and(query_param("status", "completed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "receiver_id": "re_000000000000",
                    "id": "pa_000000000000",
                    "status": "completed",
                    "sender_wallet_address": "0x123456789",
                    "quote_id": "qu_000000000000",
                    "instance_id": "in_test",
                    "network": "polygon",
                    "token": "USDC",
                    "currency": "BRL",
                    "sender_amount": 101000,
                    "receiver_amount": 500000,
                    "type": "pix",
                    "pix_key": "user@example.com",
                    "transaction_document_type": "invoice",
                    "created_at": "2021-01-01T00:00:00Z",
                    "updated_at": "2021-01-02T00:00:00Z"
                }
            ],
            "pagination": { "has_more": false, "next_page": null, "prev_page": null }
        })))
        .mount(&server)
        .await;

    let params = ListPayoutsParams::new()
        .limit(Limit::Fifty)
        .payment_method(Rail::Pix)
        .status(TransactionStatus::Completed);
    let res = client(&server).await.payouts.list(&params).await.unwrap();

    assert_eq!(res.data().len(), 1);
    let payout = &res.data()[0];
    assert_eq!(payout.id, "pa_000000000000");
    assert_eq!(payout.status, TransactionStatus::Completed);
    assert_eq!(payout.token, Some(Token::Usdc));
    assert_eq!(payout.account_rail, Some(Rail::Pix));
    assert_eq!(payout.pix_key.as_deref(), Some("user@example.com"));
    assert_eq!(
        payout.transaction_document_type,
        Some(TransactionDocumentType::Invoice)
    );
    assert!(!res.pagination().unwrap().has_more);
}

#[tokio::test]
async fn get_parses_joined_payout() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payouts/pa_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "receiver_id": "re_000000000000",
            "id": "pa_000000000000",
            "status": "processing",
            "sender_wallet_address": "0x123456789",
            "quote_id": "qu_000000000000",
            "instance_id": "in_test",
            "first_name": "John",
            "last_name": "Doe",
            "has_virtual_account": true,
            "sender_legal_name": "LRB TRADING INVESTMENT L.L.C",
            "tracking_payment": {
                "step": "completed",
                "provider_name": "JPMorgan Chase",
                "provider_status": "sent",
                "estimated_time_of_arrival": "1_business_day",
                "recipient_name": "Jane Roe"
            },
            "tracking_complete": {
                "step": "completed",
                "status": "paid"
            },
            "jpm_track_data": {
                "jpm_processing_status": "COMPLETED",
                "extended_tracking_status": null
            }
        })))
        .mount(&server)
        .await;

    let payout = client(&server)
        .await
        .payouts
        .get("pa_000000000000")
        .await
        .unwrap();

    assert_eq!(payout.id, "pa_000000000000");
    assert_eq!(payout.first_name.as_deref(), Some("John"));
    assert_eq!(payout.has_virtual_account, Some(true));
    let payment = payout.tracking_payment.unwrap();
    assert_eq!(payment.step, PayoutTrackingStep::Completed);
    assert_eq!(payment.recipient_name.as_deref(), Some("Jane Roe"));
    assert_eq!(
        payment.provider_name,
        Some(ProviderName::from("JPMorgan Chase"))
    );
    assert_eq!(
        payment.provider_status,
        Some(PayoutPaymentProviderStatus::Sent)
    );
    assert_eq!(
        payout.tracking_complete.unwrap().status,
        Some(PayoutCompleteStatus::Paid)
    );
    assert!(payout.jpm_track_data.is_some());
}

#[tokio::test]
async fn get_track_uses_unauthenticated_e_route() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/e/payouts/pa_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "receiver_id": "re_000000000000",
            "id": "pa_000000000000",
            "status": "completed",
            "sender_wallet_address": "0x123456789",
            "quote_id": "qu_000000000000",
            "instance_id": "in_test"
        })))
        .mount(&server)
        .await;

    let payout = client(&server)
        .await
        .payouts
        .get_track("pa_000000000000")
        .await
        .unwrap();

    assert_eq!(payout.id, "pa_000000000000");
    assert_eq!(payout.status, TransactionStatus::Completed);
}

#[tokio::test]
async fn unknown_tracking_step_does_not_break_decoding() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payouts/pa_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "receiver_id": "re_000000000000",
            "id": "pa_000000000000",
            "status": "on_hold",
            "sender_wallet_address": "0x123456789",
            "quote_id": "qu_000000000000",
            "instance_id": "in_test",
            "tracking_transaction": { "step": "future_step" }
        })))
        .mount(&server)
        .await;

    let payout = client(&server)
        .await
        .payouts
        .get("pa_000000000000")
        .await
        .unwrap();

    assert_eq!(
        payout.tracking_transaction.unwrap().step,
        PayoutTrackingStep::Unknown("future_step".to_string())
    );
}

#[tokio::test]
async fn create_evm_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/evm"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "quote expired"
        })))
        .mount(&server)
        .await;

    let input = CreateEvmPayoutInput::new("qu_000000000000", "0x123456789");
    let err = client(&server)
        .await
        .payouts
        .create_evm(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "quote expired");
            assert!(api.raw_body.contains("quote expired"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn get_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payouts/pa_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "message": "payout not found"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .payouts
        .get("pa_missing")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "payout not found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn create_solana_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/solana"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "invalid signed transaction"
        })))
        .mount(&server)
        .await;

    let input = CreateSolanaPayoutInput::new("qu_000000000000", "9xQeWv...");
    let err = client(&server)
        .await
        .payouts
        .create_solana(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "invalid signed transaction");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn create_stellar_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/stellar"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "trustline not authorized"
        })))
        .mount(&server)
        .await;

    let input = CreateStellarPayoutInput::new("qu_000000000000", "GABC...");
    let err = client(&server)
        .await
        .payouts
        .create_stellar(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 422);
            assert_eq!(api.message, "trustline not authorized");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn authorize_stellar_token_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payouts/stellar/authorize"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "quote not found"
        })))
        .mount(&server)
        .await;

    let input = AuthorizeStellarTokenInput::new("qu_000000000000", "GABC...");
    let err = client(&server)
        .await
        .payouts
        .authorize_stellar_token(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "quote not found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn submit_documents_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/payouts/pa_000000000000/documents",
        ))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "unsupported document type"
        })))
        .mount(&server)
        .await;

    let input = SubmitPayoutDocumentsInput {
        transaction_document_type: TransactionDocumentType::Invoice,
        transaction_document_id: "INV-12345".to_string(),
        transaction_document_file: "https://example.com/document.pdf".to_string(),
        description: None,
    };
    let err = client(&server)
        .await
        .payouts
        .submit_documents("pa_000000000000", &input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "unsupported document type");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn list_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payouts"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "message": "unauthorized"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .payouts
        .list(&ListPayoutsParams::new())
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 401);
            assert_eq!(api.message, "unauthorized");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn get_track_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/e/payouts/pa_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "message": "payout not found"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .payouts
        .get_track("pa_missing")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "payout not found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
