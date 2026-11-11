//! The 18 ops added for 94/94 coverage, exercised through their real methods:
//! each call must hit the exact method + path (+ the
//! `instance_id` query where the route needs it) and decode every
//! spec-shaped fixture variant the mock serves back.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use blindpay::*;
use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const IN: &str = "in_000000000000";
const RE: &str = "re_000000000000";
const BA: &str = "ba_000000000000";
const ID: &str = "id_000000000000";

type Call = fn(BlindPay) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

fn variants(op: &str) -> Vec<(String, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let read = |p: &Path| -> Value { serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap() };
    let index = read(&root.join("index.json"));
    let file = index["operations"][op]
        .as_str()
        .unwrap_or_else(|| panic!("{op} not in index.json"));
    let fixture = read(&root.join(file));
    fixture["responses"]["200"]["variants"]
        .as_object()
        .unwrap_or_else(|| panic!("{op}: no 200 variants"))
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

async fn check(
    op: &str,
    http_method: &str,
    wire_path: &str,
    needs_instance_query: bool,
    call: Call,
) {
    let server = MockServer::start().await;
    let client = BlindPay::builder("test_key", IN)
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(0)
        .build()
        .unwrap();
    let all = variants(op);
    assert!(!all.is_empty());
    for (name, body) in all {
        server.reset().await;
        let mut mock = Mock::given(method(http_method)).and(path(wire_path));
        if needs_instance_query {
            mock = mock.and(query_param("instance_id", IN));
        }
        mock.respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        if let Err(e) = call(client.clone()).await {
            panic!("{op} [{name}]: {e}");
        }
        server.verify().await;
    }
}

macro_rules! call {
    (|$c:ident| $body:expr) => {
        |$c: BlindPay| Box::pin(async move { $body.await.map(drop) })
    };
}

fn file() -> UploadFile {
    UploadFile::new("doc.pdf", b"%PDF".to_vec()).content_type("application/pdf")
}

#[tokio::test]
async fn instances_get_onboarding_and_rfi() {
    check(
        "GET /v1/instances/{id}",
        "GET",
        "/v1/instances/in_000000000000",
        false,
        call!(|c| c.instances.get()),
    )
    .await;
    check(
        "GET /v1/instances/{instance_id}/onboarding/business_details",
        "GET",
        "/v1/instances/in_000000000000/onboarding/business_details",
        false,
        call!(|c| c.instances.onboarding.get_business_details()),
    )
    .await;
    check(
        "GET /v1/instances/{instance_id}/onboarding/business_profile",
        "GET",
        "/v1/instances/in_000000000000/onboarding/business_profile",
        false,
        call!(|c| c.instances.onboarding.get_business_profile()),
    )
    .await;
    check(
        "GET /v1/instances/{instance_id}/onboarding/ownership_documents",
        "GET",
        "/v1/instances/in_000000000000/onboarding/ownership_documents",
        false,
        call!(|c| c.instances.onboarding.get_ownership_documents()),
    )
    .await;
    check(
        "GET /v1/instances/{instance_id}/rfi",
        "GET",
        "/v1/instances/in_000000000000/rfi",
        false,
        call!(|c| c.instances.rfi.get()),
    )
    .await;
    check(
        "POST /v1/instances/{instance_id}/rfi",
        "POST",
        "/v1/instances/in_000000000000/rfi",
        false,
        call!(|c| c.instances.rfi.submit(&RfiAnswers::new())),
    )
    .await;
}

#[tokio::test]
async fn customer_rfi_and_plaid() {
    check(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/rfi",
        "GET",
        "/v1/instances/in_000000000000/customers/re_000000000000/rfi",
        false,
        call!(|c| c.customers.rfi.get(RE)),
    )
    .await;
    check(
        "POST /v1/instances/{instance_id}/customers/{customer_id}/rfi",
        "POST",
        "/v1/instances/in_000000000000/customers/re_000000000000/rfi",
        false,
        call!(|c| c.customers.rfi.submit(RE, &RfiAnswers::new())),
    )
    .await;
    check(
        "POST /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/plaid",
        "POST",
        "/v1/instances/in_000000000000/customers/re_000000000000/bank-accounts/plaid",
        false,
        call!(|c| c
            .customers
            .bank_accounts
            .create_plaid_link(RE, &CreatePlaidLinkInput::new())),
    )
    .await;
}

#[tokio::test]
async fn wallet_ops() {
    check("PATCH /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}", "PATCH",
        "/v1/instances/in_000000000000/customers/re_000000000000/blockchain-wallets/id_000000000000", false,
        call!(|c| c.wallets.blockchain.set_self_custody(RE, ID, true))).await;
    check("PUT /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}", "PUT",
        "/v1/instances/in_000000000000/customers/re_000000000000/bank-accounts/ba_000000000000/offramp-wallets/id_000000000000", false,
        call!(|c| c.wallets.offramp.update(RE, BA, ID, &UpdateOfframpWalletInput::new().clear_partner_fee()))).await;
    check(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}/deposits",
        "GET",
        "/v1/instances/in_000000000000/customers/re_000000000000/wallets/id_000000000000/deposits",
        false,
        call!(|c| c
            .wallets
            .custodial
            .get_deposits(RE, ID, &ListWalletDepositsParams::new())),
    )
    .await;
}

#[tokio::test]
async fn payables() {
    check(
        "POST /v1/instances/{instance_id}/payables",
        "POST",
        "/v1/instances/in_000000000000/payables",
        false,
        call!(|c| c
            .payables
            .create(&CreatePayableInput::new(RE, Currency::Brl))),
    )
    .await;
    check(
        "GET /v1/instances/{instance_id}/payables",
        "GET",
        "/v1/instances/in_000000000000/payables",
        false,
        call!(|c| c.payables.list(&ListPayablesParams::new())),
    )
    .await;
    check(
        "GET /v1/instances/{instance_id}/payables/{id}",
        "GET",
        "/v1/instances/in_000000000000/payables/id_000000000000",
        false,
        call!(|c| c.payables.get(ID)),
    )
    .await;
    check(
        "DELETE /v1/instances/{instance_id}/payables/{id}",
        "DELETE",
        "/v1/instances/in_000000000000/payables/id_000000000000",
        false,
        call!(|c| c.payables.delete(ID)),
    )
    .await;
}

#[tokio::test]
async fn upload_presign_and_analyze() {
    check(
        "POST /v1/presign",
        "POST",
        "/v1/presign",
        true,
        call!(|c| c.upload.presign("https://files.blindpay.com/x.pdf")),
    )
    .await;
    check(
        "POST /v1/upload/analyze",
        "POST",
        "/v1/upload/analyze",
        true,
        call!(|c| c
            .upload
            .analyze(file(), AnalyzeDocumentType::ProofOfAddress, None)),
    )
    .await;
}

#[tokio::test]
async fn rfi_get_returns_none_for_null() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_000000000000/rfi"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("null", "application/json"))
        .mount(&server)
        .await;
    let client = BlindPay::builder("test_key", IN)
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .unwrap();
    assert!(client.instances.rfi.get().await.unwrap().is_none());
}

#[tokio::test]
async fn bodies_of_new_mutations() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let client = BlindPay::builder("test_key", IN)
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .unwrap();
    let mut answers = RfiAnswers::new();
    answers.insert("source_of_funds".into(), RfiAnswer::from("salary"));
    answers.insert(
        "docs".into(),
        RfiAnswer::from(vec!["https://f/1.pdf".to_string()]),
    );
    let _ = client.customers.rfi.submit(RE, &answers).await;
    let _ = client
        .wallets
        .blockchain
        .set_self_custody(RE, ID, false)
        .await;
    let _ = client
        .wallets
        .offramp
        .update(
            RE,
            BA,
            ID,
            &UpdateOfframpWalletInput::new().clear_partner_fee(),
        )
        .await;
    let _ = client
        .wallets
        .offramp
        .update(RE, BA, ID, &UpdateOfframpWalletInput::new())
        .await;
    let _ = client.upload.presign("https://f/1.pdf").await;

    let reqs = server.received_requests().await.unwrap();
    let body = |i: usize| -> Value { serde_json::from_slice(&reqs[i].body).unwrap() };
    assert_eq!(reqs.len(), 5);
    // RFI answers are the whole body, not wrapped.
    assert_eq!(
        body(0),
        serde_json::json!({ "docs": ["https://f/1.pdf"], "source_of_funds": "salary" })
    );
    assert_eq!(body(1), serde_json::json!({ "is_self_custody": false }));
    assert_eq!(body(2), serde_json::json!({ "partner_fee_id": null }));
    assert_eq!(body(3), serde_json::json!({}));
    assert_eq!(
        body(4),
        serde_json::json!({ "file_url": "https://f/1.pdf" })
    );
    assert_eq!(reqs[4].url.query(), Some("instance_id=in_000000000000"));
    // Every JSON mutation carried an auto-generated idempotency key.
    assert!(
        reqs.iter()
            .all(|r| r.headers.contains_key("idempotency-key"))
    );
}

#[tokio::test]
async fn multipart_analyze_sends_no_key_and_is_sent_once() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/upload/analyze"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    let client = BlindPay::builder("test_key", IN)
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .unwrap();
    let meta = serde_json::json!({ "full_name": "Jane Doe" });
    let err = client
        .upload
        .analyze(file(), AnalyzeDocumentType::IdentityDocument, Some(&meta))
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(503));
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1, "multipart is never retried");
    assert!(!reqs[0].headers.contains_key("idempotency-key"));
    let ct = reqs[0].headers["content-type"].to_str().unwrap();
    assert!(ct.starts_with("multipart/form-data; boundary="), "{ct}");
    let raw = String::from_utf8_lossy(&reqs[0].body);
    assert!(raw.contains("name=\"type\"\r\n\r\nidentity_document"));
    assert!(raw.contains("name=\"metadata\"\r\n\r\n{\"full_name\":\"Jane Doe\"}"));
    assert!(raw.contains("filename=\"doc.pdf\"") && raw.contains("application/pdf"));
}
