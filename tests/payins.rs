//! Integration tests for the `payins` resource and its `quotes` sub-resource,
//! backed by a mock HTTP server.

use blindpay::resources::payins::{TrackingStatus, TransfersType};
use blindpay::{
    AccountClass, BlindPay, CreatePayinInput, CreatePayinQuoteInput, Currency, CurrencyType, Error,
    Limit, ListPayinsParams, PayerRules, PayinQuoteFxInput, PaymentMethod, PseDocumentType, Token,
    TransactionStatus,
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

#[tokio::test]
async fn create_quote_sends_body_and_parses_response() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payin-quotes"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .and(body_json(serde_json::json!({
            "blockchain_wallet_id": "bw_123",
            "currency_type": "sender",
            "cover_fees": true,
            "request_amount": 1000,
            "payment_method": "pix",
            "token": "USDC",
            "payer_rules": { "pix_allowed_tax_ids": ["123.456.789-09"] }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pq_000000000000",
            "expires_at": 1712958191,
            "commercial_quotation": 4.95,
            "blindpay_quotation": 5.05,
            "receiver_amount": 1010,
            "sender_amount": 5240,
            "partner_fee_amount": 150,
            "flat_fee": 50,
            "billing_fee_amount": 50,
            "is_otc": false
        })))
        .mount(&server)
        .await;

    let input = CreatePayinQuoteInput {
        blockchain_wallet_id: Some("bw_123".to_string()),
        currency_type: CurrencyType::Sender,
        cover_fees: true,
        request_amount: 1000,
        payment_method: PaymentMethod::Pix,
        token: Token::Usdc,
        payer_rules: Some(PayerRules {
            pix_allowed_tax_ids: Some(vec!["123.456.789-09".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let quote = client(&server)
        .await
        .payins
        .quotes
        .create(&input)
        .await
        .unwrap();

    assert_eq!(quote.id, "pq_000000000000");
    assert_eq!(quote.sender_amount, 5240);
    assert_eq!(quote.receiver_amount, 1010);
    assert_eq!(quote.partner_fee_amount, Some(150));
    assert_eq!(quote.is_otc, Some(false));
}

#[tokio::test]
async fn get_fx_rate_sends_body_and_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payin-quotes/fx"))
        .and(body_json(serde_json::json!({
            "from": "BRL",
            "to": "USDC",
            "request_amount": 1000,
            "currency_type": "sender"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "commercial_quotation": 4.95,
            "blindpay_quotation": 5.05,
            "result_amount": 1010,
            "instance_flat_fee": 50,
            "instance_percentage_fee": 100
        })))
        .mount(&server)
        .await;

    let fx = client(&server)
        .await
        .payins
        .quotes
        .get_fx_rate(&PayinQuoteFxInput {
            from: Currency::Brl,
            to: Token::Usdc,
            request_amount: 1000,
            currency_type: CurrencyType::Sender,
        })
        .await
        .unwrap();

    assert_eq!(fx.result_amount, 1010);
    assert_eq!(fx.instance_flat_fee, 50);
    assert_eq!(fx.instance_percentage_fee, 100);
}

#[tokio::test]
async fn create_evm_sends_payin_quote_id_and_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payins/evm"))
        .and(body_json(serde_json::json!({
            "payin_quote_id": "pq_000000000000"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pi_000000000000",
            "status": "processing",
            "pix_code": "00020101...6304BCAB",
            "memo_code": null,
            "clabe": null,
            "partner_fee": 25000,
            "tracking_complete": { "step": "processing" },
            "tracking_payment": { "step": "processing" },
            "tracking_transaction": { "step": "processing", "external_id": null },
            "billing_fee_amount": 50,
            "transaction_fee_amount": 100,
            "blindpay_bank_details": {
                "routing_number": "121145349",
                "account_number": "621327727210181",
                "account_type": "Business checking",
                "beneficiary": {
                    "name": "BlindPay, Inc.",
                    "address_line_1": "8 The Green, #19364",
                    "address_line_2": "Dover, DE 19901"
                },
                "receiving_bank": {
                    "name": "CFSB",
                    "address_line_1": "1 Letterman Drive",
                    "address_line_2": "San Francisco, CA 94129"
                }
            },
            "receiver_id": "re_000000000000",
            "receiver_amount": 1010,
            "payment_method": "pix",
            "sender_amount": 5240
        })))
        .mount(&server)
        .await;

    let payin = client(&server)
        .await
        .payins
        .create_evm(&CreatePayinInput {
            payin_quote_id: "pq_000000000000".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(payin.id, "pi_000000000000");
    assert_eq!(payin.status, TransactionStatus::Processing);
    assert_eq!(payin.partner_fee, Some(25000));
    assert_eq!(payin.payment_method, Some(PaymentMethod::Pix));
    assert_eq!(payin.blindpay_bank_details.routing_number, "121145349");
    assert_eq!(payin.receiver_amount, Some(1010));
}

#[tokio::test]
async fn list_sends_pagination_and_filters_and_parses_paginated() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payins"))
        .and(query_param("limit", "50"))
        .and(query_param("status", "completed"))
        .and(query_param("customer_id", "re_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "pi_000000000000",
                    "receiver_id": "re_000000000000",
                    "status": "completed",
                    "payin_quote_id": "pq_000000000000",
                    "instance_id": "in_test",
                    "tracking_transaction": {
                        "step": "completed",
                        "status": "completed",
                        "external_id": "12345678",
                        "provider_name": "HSBC"
                    },
                    "tracking_payment": { "step": "completed" },
                    "tracking_complete": { "step": "completed", "transaction_hash": "0xabc" },
                    "created_at": "2024-01-01T00:00:00.000Z",
                    "updated_at": "2024-01-02T00:00:00.000Z",
                    "type": "individual",
                    "payment_method": "pix",
                    "sender_amount": 5240,
                    "receiver_amount": 1010,
                    "token": "USDC",
                    "commercial_quotation": 4.95,
                    "blindpay_quotation": 5.05,
                    "currency": "BRL",
                    "network": "base"
                }
            ],
            "pagination": { "has_more": false, "next_page": null, "prev_page": null }
        })))
        .mount(&server)
        .await;

    let params = ListPayinsParams::new()
        .limit(Limit::Fifty)
        .status(TransactionStatus::Completed)
        .customer_id("re_000000000000");

    let resp = client(&server).await.payins.list(&params).await.unwrap();

    assert_eq!(resp.data().len(), 1);
    let payin = &resp.data()[0];
    assert_eq!(payin.id, "pi_000000000000");
    assert_eq!(payin.account_class, AccountClass::Individual);
    assert_eq!(payin.status, TransactionStatus::Completed);
    assert_eq!(payin.tracking_transaction.step, TrackingStatus::Completed);
    assert_eq!(
        payin.tracking_transaction.external_id.as_deref(),
        Some("12345678")
    );
    assert_eq!(
        payin
            .tracking_transaction
            .provider_name
            .as_ref()
            .map(|p| p.as_str()),
        Some("HSBC")
    );
    assert!(resp.pagination().is_some());
    assert!(!resp.pagination().unwrap().has_more);
}

#[tokio::test]
async fn list_handles_bare_array_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payins"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "pi_000000000000",
                "receiver_id": "re_000000000000",
                "status": "processing",
                "payin_quote_id": "pq_000000000000",
                "instance_id": "in_test",
                "tracking_transaction": { "step": "processing", "external_id": null },
                "tracking_payment": { "step": "processing" },
                "tracking_complete": { "step": "processing" },
                "created_at": "2024-01-01T00:00:00.000Z",
                "updated_at": "2024-01-01T00:00:00.000Z",
                "type": "business",
                "payment_method": "wire",
                "sender_amount": 10000,
                "receiver_amount": 9800,
                "token": "USDC",
                "commercial_quotation": 1.0,
                "blindpay_quotation": 1.02,
                "currency": "USD"
            }
        ])))
        .mount(&server)
        .await;

    let resp = client(&server)
        .await
        .payins
        .list(&ListPayinsParams::new())
        .await
        .unwrap();

    assert_eq!(resp.data().len(), 1);
    assert_eq!(resp.data()[0].account_class, AccountClass::Business);
    assert!(resp.pagination().is_none());
}

#[tokio::test]
async fn get_parses_full_payin() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payins/pi_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pi_000000000000",
            "receiver_id": "re_000000000000",
            "pix_code": "00020101...6304BCAB",
            "status": "on_hold",
            "payin_quote_id": "pq_000000000000",
            "instance_id": "in_test",
            "tracking_transaction": {
                "step": "processing",
                "external_id": null,
                "pse_instruction": {
                    "payment_link": "https://pse.example.com/payment/abc123",
                    "fid": "fid_abc123",
                    "full_name": "Juan Perez",
                    "tax_id": "1234567890",
                    "document_type": "CC",
                    "phone": "+573001234567",
                    "email": "juan@example.com"
                }
            },
            "tracking_payment": { "step": "on_hold" },
            "tracking_complete": { "step": "processing" },
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-01T00:00:00.000Z",
            "type": "individual",
            "payment_method": "pse",
            "sender_amount": 5240,
            "receiver_amount": 1010,
            "token": "USDC",
            "commercial_quotation": 4.95,
            "blindpay_quotation": 5.05,
            "currency": "COP",
            "pse_payment_link": "https://pse.example.com/payment/abc123",
            "pse_document_type": "CC"
        })))
        .mount(&server)
        .await;

    let payin = client(&server)
        .await
        .payins
        .get("pi_000000000000")
        .await
        .unwrap();

    assert_eq!(payin.id, "pi_000000000000");
    assert_eq!(payin.status, TransactionStatus::OnHold);
    assert_eq!(payin.currency, Currency::Cop);
    let pse = payin.tracking_transaction.pse_instruction.as_ref().unwrap();
    assert_eq!(pse.document_type, PseDocumentType::Cc);
    assert_eq!(pse.fid, "fid_abc123");
    assert_eq!(payin.pse_document_type, Some(PseDocumentType::Cc));
    assert_eq!(
        payin.pse_payment_link.as_deref(),
        Some("https://pse.example.com/payment/abc123")
    );
}

#[tokio::test]
async fn get_parses_transfers_instruction_and_null_tracking_fields() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payins/pi_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pi_000000000000",
            "receiver_id": "re_000000000000",
            "status": "processing",
            "payin_quote_id": "pq_000000000000",
            "instance_id": "in_test",
            "tracking_transaction": {
                "step": "processing",
                "status": null,
                "external_id": null,
                "provider_name": null,
                "transfers_instruction": {
                    "account": "0000003100012389237485",
                    "type": "CVU",
                    "tax_id": "20123456783"
                }
            },
            "tracking_payment": { "step": "processing" },
            "tracking_complete": { "step": "processing" },
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-01T00:00:00.000Z",
            "type": "individual",
            "payment_method": "transfers",
            "sender_amount": 5240,
            "receiver_amount": 1010,
            "token": "USDC",
            "commercial_quotation": 4.95,
            "blindpay_quotation": 5.05,
            "currency": "ARS"
        })))
        .mount(&server)
        .await;

    let payin = client(&server)
        .await
        .payins
        .get("pi_000000000000")
        .await
        .unwrap();

    assert_eq!(payin.tracking_transaction.status, None);
    assert_eq!(payin.tracking_transaction.provider_name, None);
    let transfers = payin
        .tracking_transaction
        .transfers_instruction
        .as_ref()
        .unwrap();
    assert_eq!(transfers.transfers_type, TransfersType::Cvu);
    assert_eq!(transfers.account, "0000003100012389237485");
}

#[tokio::test]
async fn get_track_uses_unauthenticated_e_route() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/e/payins/pi_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pi_000000000000",
            "receiver_id": "re_000000000000",
            "status": "completed",
            "payin_quote_id": "pq_000000000000",
            "instance_id": "in_test",
            "tracking_transaction": { "step": "completed", "external_id": null },
            "tracking_payment": { "step": "completed" },
            "tracking_complete": { "step": "completed" },
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-01T00:00:00.000Z",
            "type": "individual",
            "payment_method": "pix",
            "sender_amount": 5240,
            "receiver_amount": 1010,
            "token": "USDC",
            "commercial_quotation": 4.95,
            "blindpay_quotation": 5.05,
            "currency": "BRL"
        })))
        .mount(&server)
        .await;

    let payin = client(&server)
        .await
        .payins
        .get_track("pi_000000000000")
        .await
        .unwrap();

    assert_eq!(payin.id, "pi_000000000000");
    assert_eq!(payin.status, TransactionStatus::Completed);
}

#[tokio::test]
async fn unknown_payment_method_does_not_break_decoding() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payins/pi_000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pi_000000000000",
            "receiver_id": "re_000000000000",
            "status": "processing",
            "payin_quote_id": "pq_000000000000",
            "instance_id": "in_test",
            "tracking_transaction": { "step": "processing", "external_id": null },
            "tracking_payment": { "step": "processing" },
            "tracking_complete": { "step": "processing" },
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-01T00:00:00.000Z",
            "type": "individual",
            "payment_method": "future_method",
            "sender_amount": 5240,
            "receiver_amount": 1010,
            "token": "USDC",
            "commercial_quotation": 4.95,
            "blindpay_quotation": 5.05,
            "currency": "BRL"
        })))
        .mount(&server)
        .await;

    let payin = client(&server)
        .await
        .payins
        .get("pi_000000000000")
        .await
        .unwrap();

    assert_eq!(
        payin.payment_method,
        PaymentMethod::Unknown("future_method".to_string())
    );
}

#[tokio::test]
async fn create_quote_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/payin-quotes"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "invalid request amount" })),
        )
        .mount(&server)
        .await;

    let input = CreatePayinQuoteInput {
        currency_type: CurrencyType::Sender,
        cover_fees: false,
        request_amount: 100,
        payment_method: PaymentMethod::Pix,
        token: Token::Usdc,
        ..Default::default()
    };

    let err = client(&server)
        .await
        .payins
        .quotes
        .create(&input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "invalid request amount");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn get_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/payins/pi_missing"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({ "message": "not found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .payins
        .get("pi_missing")
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
