//! Integration tests for the `virtual_accounts` resource, backed by a mock
//! HTTP server.

use blindpay::{
    BankingPartner, BlindPay, CreateVirtualAccountInput, Error, KycStatus, Network, Token,
    UpdateVirtualAccountInput,
};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test-api-key", "in_test")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .expect("client should build")
}

fn virtual_account_json() -> serde_json::Value {
    serde_json::json!({
        "id": "va_123",
        "banking_partner": "jpmorgan",
        "kyc_status": "approved",
        "us": {
            "ach": { "routing_number": "021000021", "account_number": "111111111" },
            "wire": { "routing_number": "021000021", "account_number": "222222222" },
            "rtp": { "routing_number": "021000021", "account_number": "333333333" },
            "swift_bic_code": "TCCLGB3L",
            "swift_account_number": "GB50TCCL04140449730892",
            "account_type": "Business checking",
            "beneficiary": {
                "name": "Test Co",
                "address_line_1": "8 The Green, #19364",
                "address_line_2": "Dover, DE 19901"
            },
            "receiving_bank": {
                "name": "JPMorgan Chase",
                "address_line_1": "270 Park Ave",
                "address_line_2": "New York, NY, 10017-2070"
            },
            "swift_intermediary_bank": {
                "name": "JP Morgan Chase NA",
                "swift_code_bic": "CHASUS33",
                "routing_number": "021000021"
            }
        },
        "token": "USDC",
        "blockchain_wallet_id": "bw_123",
        "blockchain_wallet": { "network": "base", "address": "0xabc" }
    })
}

#[tokio::test]
async fn list_parses_response_and_sends_auth_headers() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts",
        ))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([virtual_account_json()])),
        )
        .mount(&server)
        .await;

    let accounts = client(&server)
        .await
        .virtual_accounts
        .list("cus_123")
        .await
        .unwrap();

    assert_eq!(accounts.len(), 1);
    let va = &accounts[0];
    assert_eq!(va.id, "va_123");
    assert_eq!(va.banking_partner, BankingPartner::Jpmorgan);
    assert_eq!(va.kyc_status, KycStatus::Approved);
    assert_eq!(va.token, Token::Usdc);
    assert_eq!(va.us.wire.account_number, "222222222");
    assert!(va.us.ach.is_some());
    assert_eq!(va.us.swift_bic_code.as_deref(), Some("TCCLGB3L"));
    assert_eq!(
        va.blockchain_wallet.as_ref().unwrap().network,
        Network::Base
    );
}

#[tokio::test]
async fn get_parses_single_account() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts/va_123",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(virtual_account_json()))
        .mount(&server)
        .await;

    let account = client(&server)
        .await
        .virtual_accounts
        .get("cus_123", "va_123")
        .await
        .unwrap();

    let va = account.expect("expected Some(VirtualAccount)");
    assert_eq!(va.id, "va_123");
    assert_eq!(va.blockchain_wallet_id.as_deref(), Some("bw_123"));
}

#[tokio::test]
async fn get_returns_none_for_json_null() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts/va_missing",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::Value::Null))
        .mount(&server)
        .await;

    let account = client(&server)
        .await
        .virtual_accounts
        .get("cus_123", "va_missing")
        .await
        .unwrap();

    assert!(account.is_none());
}

#[tokio::test]
async fn get_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts/va_missing",
        ))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "message": "virtual_account_not_found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .virtual_accounts
        .get("cus_123", "va_missing")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "virtual_account_not_found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn create_sends_body_and_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts",
        ))
        .and(header("authorization", "Bearer test-api-key"))
        .and(body_json(serde_json::json!({
            "banking_partner": "jpmorgan",
            "token": "USDC",
            "blockchain_wallet_id": "bw_123"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(virtual_account_json()))
        .mount(&server)
        .await;

    let input = CreateVirtualAccountInput::new(BankingPartner::Jpmorgan, Token::Usdc, "bw_123");
    let va = client(&server)
        .await
        .virtual_accounts
        .create("cus_123", &input)
        .await
        .unwrap();

    assert_eq!(va.id, "va_123");
    assert_eq!(va.banking_partner, BankingPartner::Jpmorgan);
}

#[tokio::test]
async fn update_sends_put_body_and_parses_success() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts/va_123",
        ))
        .and(body_json(serde_json::json!({
            "token": "USDC",
            "blockchain_wallet_id": "bw_456"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let input = UpdateVirtualAccountInput::new(Token::Usdc, "bw_456");
    let result = client(&server)
        .await
        .virtual_accounts
        .update("cus_123", "va_123", &input)
        .await
        .unwrap();

    assert!(result.success);
}

#[tokio::test]
async fn list_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts",
        ))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_json(serde_json::json!({ "message": "user_not_allowed" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .virtual_accounts
        .list("cus_123")
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
async fn create_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts",
        ))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "invalid_banking_partner" })),
        )
        .mount(&server)
        .await;

    let input = CreateVirtualAccountInput::new(BankingPartner::Citi, Token::Usdc, "bw_123");
    let err = client(&server)
        .await
        .virtual_accounts
        .create("cus_123", &input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "invalid_banking_partner");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn update_not_found_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/virtual-accounts/va_missing",
        ))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "message": "virtual_account_not_found" })),
        )
        .mount(&server)
        .await;

    let input = UpdateVirtualAccountInput::new(Token::Usdc, "bw_456");
    let err = client(&server)
        .await
        .virtual_accounts
        .update("cus_123", "va_missing", &input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "virtual_account_not_found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
