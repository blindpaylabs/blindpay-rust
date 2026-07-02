//! Integration tests for the `quotes` resource, backed by a mock HTTP server.

use blindpay::{
    BlindPay, CreateQuoteInput, Currency, CurrencyType, Error, GetFxRateInput, Network, Token,
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
async fn create_sends_body_and_parses_quote() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/quotes"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .and(body_json(serde_json::json!({
            "bank_account_id": "ba_123",
            "currency_type": "sender",
            "cover_fees": true,
            "request_amount": 1000,
            "network": "sepolia",
            "token": "USDC",
            "description": "memo",
            "partner_fee_id": "pf_123"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "qu_123",
            "expires_at": 1712958191_i64,
            "commercial_quotation": 4.95,
            "blindpay_quotation": 4.85,
            "receiver_amount": 5240,
            "sender_amount": 1010,
            "partner_fee_amount": 150,
            "flat_fee": 50,
            "billing_fee_amount": 50,
            "receiver_local_amount": 1000,
            "description": "memo",
            "contract": {
                "abi": [{}],
                "address": "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
                "functionName": "approve",
                "blindpayContractAddress": "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
                "amount": "1000000000000000000",
                "network": { "name": "Ethereum", "chainId": 1 }
            }
        })))
        .mount(&server)
        .await;

    let quote = client(&server)
        .await
        .quotes
        .create(&CreateQuoteInput {
            bank_account_id: "ba_123".to_string(),
            currency_type: CurrencyType::Sender,
            cover_fees: true,
            request_amount: 1000,
            network: Network::Sepolia,
            token: Token::Usdc,
            description: Some("memo".to_string()),
            partner_fee_id: Some("pf_123".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(quote.id, "qu_123");
    assert_eq!(quote.sender_amount, 1010);
    assert_eq!(quote.partner_fee_amount, Some(150));
    assert_eq!(quote.flat_fee, Some(50));
    assert_eq!(quote.receiver_local_amount, Some(1000));

    let contract = quote.contract.unwrap();
    assert_eq!(contract.function_name, "approve");
    assert_eq!(
        contract.blindpay_contract_address,
        "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
    );
    assert_eq!(contract.amount, "1000000000000000000");
    assert_eq!(contract.network.name, "Ethereum");
    assert_eq!(contract.network.chain_id, 1);
}

#[tokio::test]
async fn create_omits_optional_fields_when_absent() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/quotes"))
        .and(body_json(serde_json::json!({
            "bank_account_id": "ba_123",
            "currency_type": "receiver",
            "cover_fees": false,
            "request_amount": 500,
            "network": "stellar",
            "token": "USDC"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "qu_456",
            "expires_at": 1712958191_i64,
            "commercial_quotation": 4.95,
            "blindpay_quotation": 4.85,
            "receiver_amount": 2480,
            "sender_amount": 500
        })))
        .mount(&server)
        .await;

    let quote = client(&server)
        .await
        .quotes
        .create(&CreateQuoteInput {
            bank_account_id: "ba_123".to_string(),
            currency_type: CurrencyType::Receiver,
            cover_fees: false,
            request_amount: 500,
            network: Network::Stellar,
            token: Token::Usdc,
            description: None,
            partner_fee_id: None,
        })
        .await
        .unwrap();

    assert_eq!(quote.id, "qu_456");
    assert_eq!(quote.partner_fee_amount, None);
    assert!(quote.contract.is_none());
    assert!(quote.description.is_none());
}

#[tokio::test]
async fn get_fx_rate_sends_body_and_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/quotes/fx"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(body_json(serde_json::json!({
            "from": "USDC",
            "to": "BRL",
            "request_amount": 1000,
            "currency_type": "sender"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "commercial_quotation": 4.95,
            "blindpay_quotation": 4.85,
            "result_amount": 4850,
            "instance_flat_fee": 50,
            "instance_percentage_fee": 0
        })))
        .mount(&server)
        .await;

    let fx = client(&server)
        .await
        .quotes
        .get_fx_rate(&GetFxRateInput {
            from: Token::Usdc,
            to: Currency::Brl,
            request_amount: 1000,
            currency_type: CurrencyType::Sender,
        })
        .await
        .unwrap();

    assert!((fx.commercial_quotation - 4.95).abs() < 1e-9);
    assert_eq!(fx.result_amount, 4850);
    assert_eq!(fx.instance_flat_fee, Some(50));
    assert_eq!(fx.instance_percentage_fee, 0);
}

#[tokio::test]
async fn get_fx_rate_allows_null_instance_flat_fee() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/quotes/fx"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "commercial_quotation": 4.95,
            "blindpay_quotation": 4.85,
            "result_amount": 4850,
            "instance_flat_fee": null,
            "instance_percentage_fee": 25
        })))
        .mount(&server)
        .await;

    let fx = client(&server)
        .await
        .quotes
        .get_fx_rate(&GetFxRateInput {
            from: Token::Usdc,
            to: Currency::Brl,
            request_amount: 1000,
            currency_type: CurrencyType::Sender,
        })
        .await
        .unwrap();

    assert_eq!(fx.instance_flat_fee, None);
    assert_eq!(fx.instance_percentage_fee, 25);
}

#[tokio::test]
async fn create_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/quotes"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "bank_account_not_approved" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .quotes
        .create(&CreateQuoteInput {
            bank_account_id: "ba_123".to_string(),
            currency_type: CurrencyType::Sender,
            cover_fees: true,
            request_amount: 1000,
            network: Network::Sepolia,
            token: Token::Usdc,
            description: None,
            partner_fee_id: None,
        })
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "bank_account_not_approved");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn get_fx_rate_non_success_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/quotes/fx"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "message": "fee_not_found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .quotes
        .get_fx_rate(&GetFxRateInput {
            from: Token::Usdc,
            to: Currency::Brl,
            request_amount: 1000,
            currency_type: CurrencyType::Sender,
        })
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "fee_not_found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
