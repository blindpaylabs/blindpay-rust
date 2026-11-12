//! Secrets and PII stay out of everything a user might print or log: the
//! client's and builder's `Debug`, transport errors (`Display`, `Debug` and
//! the whole `source()` chain) and webhook verification errors.

use blindpay::{BlindPay, Error, ListCustomersParams, construct_webhook_event};

const API_KEY: &str = "sk_live_SECRET_api_key_123";
const WEBHOOK_SECRET: &str = "whsec_c2VjcmV0X3dlYmhvb2tfa2V5X2Zvcl90ZXN0cw==";

/// Every text form of an error: Display, Debug and each `source()`.
fn all_text(err: &(dyn std::error::Error + 'static)) -> String {
    let mut out = format!("{err}\n{err:?}");
    let mut source = err.source();
    while let Some(s) = source {
        out.push_str(&format!("\n{s}\n{s:?}"));
        source = s.source();
    }
    out
}

#[test]
fn client_and_builder_debug_redact_the_api_key() {
    let builder = BlindPay::builder(API_KEY, "in_000000000000");
    let builder_dbg = format!("{builder:?}");
    let client = builder.build().unwrap();
    let client_dbg = format!("{client:?} {client:#?}");
    for text in [builder_dbg, client_dbg] {
        assert!(!text.contains(API_KEY), "api key leaked: {text}");
        assert!(text.contains("<redacted>"), "{text}");
    }
}

#[tokio::test]
async fn transport_errors_carry_no_query_string() {
    // A port nothing listens on: the request gets no response.
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let client = BlindPay::builder(API_KEY, "in_000000000000")
        .base_url(format!("http://127.0.0.1:{port}"))
        .max_retries(0)
        .build()
        .unwrap();
    let params = ListCustomersParams::new()
        .full_name("Jane PII Doe")
        .customer_id("re_pii_customer");
    let err = client.customers.list(&params).await.unwrap_err();
    assert!(matches!(err, Error::Http { .. }), "{err:?}");

    let text = all_text(&err);
    for leak in ["Jane", "PII", "re_pii_customer", "full_name", "?", API_KEY] {
        assert!(!text.contains(leak), "`{leak}` leaked:\n{text}");
    }
    // The path is kept for debugging.
    assert!(
        text.contains("/v1/instances/in_000000000000/customers"),
        "{text}"
    );
}

#[test]
fn webhook_errors_never_echo_the_secret() {
    let mut headers = http::HeaderMap::new();
    headers.insert("svix-id", "msg_1".parse().unwrap());
    headers.insert("svix-timestamp", "1".parse().unwrap());
    headers.insert("svix-signature", "v1,AAAA".parse().unwrap());
    for secret in [WEBHOOK_SECRET, "whsec_not*base64!"] {
        let err = construct_webhook_event(b"{}", &headers, secret).unwrap_err();
        let text = all_text(&err);
        let body = secret.trim_start_matches("whsec_");
        assert!(!text.contains(body), "secret leaked: {text}");
    }
}
