//! Integration tests for the `available` resource, backed by a mock HTTP server.

use blindpay::{BlindPay, Error, Rail, RequiredWhenOperator};
use wiremock::matchers::{header, method, path, query_param};
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
async fn get_rails_parses_response_and_sends_auth_headers() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("GET"))
        .and(path("/v1/available/rails"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "label": "Domestic Wire", "value": "wire", "country": "US" },
            { "label": "PIX", "value": "pix", "country": "BR" },
            { "label": "SEPA", "value": "sepa", "country": "DE" },
        ])))
        .mount(&server)
        .await;

    let rails = client(&server).await.available.get_rails().await.unwrap();

    assert_eq!(rails.len(), 3);
    assert_eq!(rails[0].label, "Domestic Wire");
    assert_eq!(rails[0].value, Rail::Wire);
    assert_eq!(rails[0].country, "US");
    assert_eq!(rails[2].value, Rail::Sepa);
}

#[tokio::test]
async fn unknown_rail_value_does_not_break_decoding() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/rails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "label": "Future Rail", "value": "future_rail", "country": "US" }
        ])))
        .mount(&server)
        .await;

    let rails = client(&server).await.available.get_rails().await.unwrap();
    assert_eq!(rails[0].value, Rail::Unknown("future_rail".to_string()));
    assert_eq!(rails[0].value.as_str(), "future_rail");
}

#[tokio::test]
async fn get_bank_details_sends_rail_query_and_parses_optional_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/bank-details"))
        .and(query_param("rail", "pix"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "label": "PIX Key",
                "regex": "",
                "key": "pix_key",
                "required": true
            },
            {
                "label": "Account Type",
                "regex": "",
                "key": "account_type",
                "items": [
                    { "label": "Checking", "value": "checking" },
                    { "label": "Savings", "value": "saving", "is_active": false }
                ],
                "required": null,
                "requiredWhen": {
                    "field": "country",
                    "operator": "in",
                    "values": ["US", "GB"]
                }
            }
        ])))
        .mount(&server)
        .await;

    let details = client(&server)
        .await
        .available
        .get_bank_details(Rail::Pix)
        .await
        .unwrap();

    assert_eq!(details.len(), 2);

    // First field: simple, required, no items, no conditional rule.
    assert_eq!(details[0].key, "pix_key");
    assert_eq!(details[0].required, Some(true));
    assert!(details[0].items.is_empty());
    assert!(details[0].required_when.is_none());

    // Second field: `required: null`, items present, conditional rule present.
    assert_eq!(details[1].required, None);
    assert_eq!(details[1].items.len(), 2);
    assert_eq!(details[1].items[1].value, "saving");
    assert_eq!(details[1].items[1].is_active, Some(false));

    let rule = details[1].required_when.as_ref().unwrap();
    assert_eq!(rule.field, "country");
    assert_eq!(rule.operator, RequiredWhenOperator::In);
    assert_eq!(rule.values, vec!["US".to_string(), "GB".to_string()]);
}

#[tokio::test]
async fn get_naics_codes_uses_label_and_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/naics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "label": "(339910) Jewelry and Silverware Manufacturing", "value": "339910" }
        ])))
        .mount(&server)
        .await;

    let codes = client(&server)
        .await
        .available
        .get_naics_codes()
        .await
        .unwrap();

    assert_eq!(codes.len(), 1);
    assert_eq!(codes[0].value, "339910");
    assert!(codes[0].label.contains("Jewelry"));
}

#[tokio::test]
async fn get_swift_code_bank_details_parses_camel_case_keys() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/swift/BOFAUS3NLMA"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "416",
                "bank": "BANK OF AMERICA, N.A.",
                "city": "NEW JERSEY",
                "branch": "LENDING SERVICES AND OPERATIONS (LSOP)",
                "swiftCode": "BOFAUS3NLMA",
                "swiftCodeLink": "https://bank.codes/swift-code/united-states/bofaus3nlma/",
                "country": "United States",
                "countrySlug": "united-states"
            }
        ])))
        .mount(&server)
        .await;

    let results = client(&server)
        .await
        .available
        .get_swift_code_bank_details("BOFAUS3NLMA")
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "416");
    assert_eq!(results[0].swift_code, "BOFAUS3NLMA");
    assert_eq!(results[0].country_slug, "united-states");
}

#[tokio::test]
async fn non_success_status_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/rails"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "User not allowed"
        })))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .available
        .get_rails()
        .await
        .unwrap_err();

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
async fn malformed_body_returns_decode_error_with_raw_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/rails"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .available
        .get_rails()
        .await
        .unwrap_err();

    match err {
        Error::Decode { body, .. } => assert_eq!(body, "not json"),
        other => panic!("expected Error::Decode, got {other:?}"),
    }
}

#[tokio::test]
async fn empty_body_decodes_to_empty_vec() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/rails"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let rails = client(&server).await.available.get_rails().await.unwrap();
    assert!(rails.is_empty());
}

#[tokio::test]
async fn empty_api_key_is_a_config_error() {
    let err = BlindPay::new("", "in_test").unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[tokio::test]
async fn empty_swift_code_fails_before_any_request() {
    let client = BlindPay::new("test-api-key", "in_test").unwrap();
    let err = client
        .available
        .get_swift_code_bank_details("   ")
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

#[tokio::test]
async fn error_body_without_message_synthesizes_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/rails"))
        .respond_with(
            ResponseTemplate::new(500).set_body_json(serde_json::json!({ "code": "oops" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .available
        .get_rails()
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 500);
            // No `message` field in the body, so the message is synthesized...
            assert!(api.message.starts_with("HTTP 500 error"));
            // ...but the raw body is always retained.
            assert!(api.raw_body.contains("oops"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn bank_details_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/bank-details"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "invalid rail" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .available
        .get_bank_details(Rail::Pix)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "invalid rail");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn naics_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/naics"))
        .respond_with(
            ResponseTemplate::new(503)
                .set_body_json(serde_json::json!({ "message": "unavailable" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .available
        .get_naics_codes()
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 503);
            assert_eq!(api.message, "unavailable");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn swift_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/available/swift/BOFAUS3NLMA"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({ "message": "not found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .available
        .get_swift_code_bank_details("BOFAUS3NLMA")
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
