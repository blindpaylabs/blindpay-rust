//! Integration tests for the `upload` resource, backed by a mock HTTP server.

use blindpay::{BlindPay, Error, UploadBucket};
use wiremock::matchers::{header, header_regex, method, path, query_param};
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
async fn upload_sends_multipart_and_parses_file_url() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("POST"))
        .and(path("/v1/upload"))
        .and(query_param("instance_id", "in_test"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "file_url": "https://storage.blindpay.com/uploads/abc123.pdf"
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .await
        .upload
        .upload(
            "document.pdf",
            b"test content".to_vec(),
            UploadBucket::Onboarding,
        )
        .await
        .unwrap();

    assert_eq!(
        response.file_url,
        "https://storage.blindpay.com/uploads/abc123.pdf"
    );
}

#[tokio::test]
async fn upload_does_not_set_json_content_type() {
    let server = MockServer::start().await;

    // reqwest sets `multipart/form-data; boundary=...` for a multipart body; if
    // the transport wrongly forced `application/json`, this matcher would miss.
    Mock::given(method("POST"))
        .and(path("/v1/upload"))
        .and(header_regex("content-type", "^multipart/form-data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "file_url": "https://storage.blindpay.com/uploads/avatar.png"
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .await
        .upload
        .upload("avatar.png", b"\x89PNG".to_vec(), UploadBucket::Avatar)
        .await
        .unwrap();

    assert_eq!(
        response.file_url,
        "https://storage.blindpay.com/uploads/avatar.png"
    );
}

#[tokio::test]
async fn upload_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/upload"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "invalid file" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .upload
        .upload("bad.pdf", b"oops".to_vec(), UploadBucket::LimitIncrease)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "invalid file");
            assert!(api.raw_body.contains("invalid file"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
