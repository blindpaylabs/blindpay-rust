//! Request bodies and queries for the inputs fixed in audit §9.4 (plan A8):
//! absent optionals stay absent, and the new spec fields reach the wire.

use blindpay::{
    AccountClass, BankingPartner, BlindPay, BreBGender, BreBInput, CreateEvmPayoutInput,
    CreateFasterPaymentsInput, CreatePayinQuoteInput, CreateTransferQuoteInput,
    CreateVirtualAccountInput, CurrencyType, Network, Token,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const BASE: &str = "/v1/instances/in_000000000000";

fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test_key", "in_000000000000")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .unwrap()
}

/// Mounts a catch-all that answers 500 (mutations aren't retried on 5xx, so
/// exactly one request is recorded).
async fn capture(server: &MockServer) {
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(500))
        .mount(server)
        .await;
}

async fn only_body(server: &MockServer) -> serde_json::Value {
    let reqs: Vec<Request> = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1, "exactly one request");
    serde_json::from_slice(&reqs[0].body).unwrap()
}

#[tokio::test]
async fn managed_wallet_payout_sends_wallet_id_only() {
    let server = MockServer::start().await;
    capture(&server).await;
    let input = CreateEvmPayoutInput::from_wallet("qu_000000000000", "bl_000000000000");
    let _ = client(&server).payouts.create_evm(&input).await;
    assert_eq!(
        only_body(&server).await,
        json!({ "quote_id": "qu_000000000000", "wallet_id": "bl_000000000000" })
    );
}

#[tokio::test]
async fn payin_track_sends_checkout_token_only_when_given() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/e/payins/pi_000000000000"))
        .and(query_param("checkout_token", "ct_1"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;
    let _ = client(&server)
        .payins
        .get_track("pi_000000000000", Some("ct_1"))
        .await;
    server.verify().await;
}

#[tokio::test]
async fn faster_payments_account_is_typed_faster_payments() {
    let server = MockServer::start().await;
    capture(&server).await;
    let input = CreateFasterPaymentsInput {
        name: "GBP account".into(),
        account_class: AccountClass::Individual,
        faster_payments_sort_code: "123456".into(),
        faster_payments_account_number: "12345678".into(),
        faster_payments_beneficiary_legal_name: "Jane Doe".into(),
        faster_payments_beneficiary_address_line_1: "1 High St".into(),
        faster_payments_beneficiary_city: "London".into(),
        faster_payments_beneficiary_country: "GB".into(),
        faster_payments_beneficiary_address_line_2: None,
        faster_payments_beneficiary_state_province_region: None,
        faster_payments_beneficiary_postal_code: Some("SW1A 1AA".into()),
    };
    let _ = client(&server)
        .customers
        .bank_accounts
        .create_faster_payments("re_000000000000", &input)
        .await;
    let body = only_body(&server).await;
    assert_eq!(body["type"], "faster_payments");
    assert_eq!(body["faster_payments_sort_code"], "123456");
    assert!(
        body.get("faster_payments_beneficiary_address_line_2")
            .is_none()
    );
}

#[tokio::test]
async fn virtual_account_sends_partner_fee_and_bre_b() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "{BASE}/customers/re_000000000000/virtual-accounts"
        )))
        .and(body_json(json!({
            "banking_partner": "bre_b",
            "token": "USDC",
            "blockchain_wallet_id": "bw_000000000000",
            "partner_fee_id": "fe_000000000000",
            "bre_b": { "gender": "female" }
        })))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;
    let input = CreateVirtualAccountInput::new(
        BankingPartner::from("bre_b"),
        Token::Usdc,
        "bw_000000000000",
    )
    .partner_fee_id("fe_000000000000")
    .bre_b(BreBInput::new(BreBGender::Female));
    let _ = client(&server)
        .virtual_accounts
        .create("re_000000000000", &input)
        .await;
    server.verify().await;
}

#[tokio::test]
async fn payin_quote_omits_cover_fees_and_sends_funding_account() {
    let server = MockServer::start().await;
    capture(&server).await;
    let input = CreatePayinQuoteInput {
        request_amount: 1000,
        cover_fees: None,
        ..CreatePayinQuoteInput::default()
    }
    .funding_bank_account_id("ba_000000000000");
    let _ = client(&server).payins.quotes.create(&input).await;
    let body = only_body(&server).await;
    assert!(body.get("cover_fees").is_none());
    assert_eq!(body["funding_bank_account_id"], "ba_000000000000");
}

#[tokio::test]
async fn transfer_quote_sends_customer_fields_and_partner_fees() {
    let server = MockServer::start().await;
    capture(&server).await;
    let input = CreateTransferQuoteInput::new(
        "bl_000000000000",
        CurrencyType::Sender,
        1000,
        Token::Usdc,
        "0xabc",
        Token::Usdc,
        Network::Base,
    )
    .partner_percentage_fee(100)
    .partner_flat_fee(50);
    let _ = client(&server).transfers.quotes.create(&input).await;
    let body = only_body(&server).await;
    assert_eq!(body["customer_wallet_address"], "0xabc");
    assert_eq!(body["customer_network"], "base");
    assert_eq!(body["partner_percentage_fee"], 100);
    assert_eq!(body["partner_flat_fee"], 50);
}
