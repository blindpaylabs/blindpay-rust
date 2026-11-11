//! Transport contract: request options, idempotency keys, the retry policy,
//! and error parsing (`tests/fixtures/errors`).
//!
//! Retries use a recording `retry_sleep`, so nothing really sleeps.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use blindpay::{BlindPay, Error, RequestOptions, RetrySleep};
use reqwest::header::{HeaderName, HeaderValue};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const INSTANCE: &str = "in_000000000000";
const RAILS: &str = "/v1/available/rails";
const OWNERSHIP: &str = "/v1/instances/in_000000000000/ownership";
const PARTNER_FEE: &str = "/v1/instances/in_000000000000/partner-fees/pf_000000000000";

type Delays = Arc<Mutex<Vec<Duration>>>;

fn recorder() -> (RetrySleep, Delays) {
    let delays: Delays = Arc::default();
    let sink = Arc::clone(&delays);
    let sleep: RetrySleep = Arc::new(move |d| {
        sink.lock().unwrap().push(d);
        Box::pin(async {})
    });
    (sleep, delays)
}

fn client_with(server: &MockServer, max_retries: u32) -> (BlindPay, Delays) {
    let (sleep, delays) = recorder();
    let client = BlindPay::builder("test_key", INSTANCE)
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(max_retries)
        .retry_sleep(sleep)
        .build()
        .expect("client builds");
    (client, delays)
}

fn success() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true }))
}

fn in_flight(retry_after: &str) -> ResponseTemplate {
    ResponseTemplate::new(409)
        .insert_header("Retry-After", retry_after)
        .set_body_json(serde_json::json!({
            "success": false,
            "message": "idempotency_key_in_flight",
            "errors": ["A request with this Idempotency-Key is still being processed"]
        }))
}

async fn requests(server: &MockServer) -> Vec<Request> {
    server.received_requests().await.unwrap_or_default()
}

fn key_of(req: &Request) -> Option<String> {
    req.headers
        .get("idempotency-key")
        .map(|v| v.to_str().unwrap().to_string())
}

/// Mounts `first` for one request, then `then` for the rest.
async fn mount_sequence(
    server: &MockServer,
    verb: &str,
    route: &str,
    first: ResponseTemplate,
    then: ResponseTemplate,
) {
    Mock::given(method(verb))
        .and(path(route))
        .respond_with(first)
        .up_to_n_times(1)
        .with_priority(1)
        .mount(server)
        .await;
    Mock::given(method(verb))
        .and(path(route))
        .respond_with(then)
        .with_priority(2)
        .mount(server)
        .await;
}

// ---------------------------------------------------------------------------
// Retries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_is_retried_on_503_then_succeeds_without_a_key() {
    let server = MockServer::start().await;
    mount_sequence(
        &server,
        "GET",
        RAILS,
        ResponseTemplate::new(503),
        ResponseTemplate::new(200).set_body_json(serde_json::json!([])),
    )
    .await;
    let (client, delays) = client_with(&server, 2);

    let rails = client
        .available
        .get_rails()
        .await
        .expect("retried to success");

    assert!(rails.is_empty());
    let reqs = requests(&server).await;
    assert_eq!(reqs.len(), 2);
    assert!(
        reqs.iter().all(|r| key_of(r).is_none()),
        "GETs carry no key"
    );
    assert_eq!(delays.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn get_retries_408_500_502_504_and_gives_up_after_max_retries() {
    for status in [408u16, 500, 502, 504] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(RAILS))
            .respond_with(ResponseTemplate::new(status))
            .mount(&server)
            .await;
        let (client, delays) = client_with(&server, 2);

        let err = client.available.get_rails().await.unwrap_err();

        assert_eq!(requests(&server).await.len(), 3, "status {status}");
        assert_eq!(delays.lock().unwrap().len(), 2);
        assert!(err.is_retryable());
        assert_eq!(err.status(), Some(status));
    }
}

#[tokio::test]
async fn mutation_is_not_retried_on_5xx_and_error_carries_the_key() {
    for status in [500u16, 502, 503] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(OWNERSHIP))
            .respond_with(ResponseTemplate::new(status))
            .mount(&server)
            .await;
        let (client, delays) = client_with(&server, 2);

        let err = client
            .instances
            .migrate_ownership("us_000000000000")
            .await
            .unwrap_err();

        let reqs = requests(&server).await;
        assert_eq!(reqs.len(), 1, "status {status}: exactly one request");
        assert!(delays.lock().unwrap().is_empty());
        let Error::Api(api) = &err else {
            panic!("expected Api error, got {err:?}")
        };
        assert!(api.retryable, "flagged retryable for a deliberate resend");
        let sent = key_of(&reqs[0]).expect("auto key sent");
        assert_eq!(api.idempotency_key.as_deref(), Some(sent.as_str()));
        assert_eq!(err.idempotency_key(), Some(sent.as_str()));
    }
}

#[tokio::test]
async fn mutation_is_retried_on_429_honouring_retry_after_with_the_same_key() {
    let server = MockServer::start().await;
    mount_sequence(
        &server,
        "POST",
        OWNERSHIP,
        ResponseTemplate::new(429)
            .insert_header("Retry-After", "2")
            .set_body_json(serde_json::json!({
                "success": false,
                "code": "AUTH_RATE_LIMITED",
                "message": "rate_limited"
            })),
        success(),
    )
    .await;
    let (client, delays) = client_with(&server, 2);

    client
        .instances
        .migrate_ownership("us_000000000000")
        .await
        .expect("retried to success");

    let reqs = requests(&server).await;
    assert_eq!(reqs.len(), 2);
    let first = key_of(&reqs[0]).expect("auto key on attempt 0");
    assert_eq!(first.len(), 36, "UUIDv4: {first}");
    assert_eq!(key_of(&reqs[1]).as_deref(), Some(first.as_str()));
    assert_eq!(
        reqs[0].body, reqs[1].body,
        "identical bytes on every attempt"
    );
    assert_eq!(*delays.lock().unwrap(), vec![Duration::from_secs(2)]);
}

#[tokio::test]
async fn in_flight_409_waits_retry_after_when_at_most_60s() {
    let server = MockServer::start().await;
    mount_sequence(&server, "DELETE", PARTNER_FEE, in_flight("1"), success()).await;
    let (client, delays) = client_with(&server, 2);

    client
        .partner_fees
        .delete("pf_000000000000")
        .await
        .expect("retried to success");

    let reqs = requests(&server).await;
    assert_eq!(reqs.len(), 2);
    assert!(
        key_of(&reqs[0]).is_some(),
        "body-less DELETE still gets a key"
    );
    assert_eq!(key_of(&reqs[0]), key_of(&reqs[1]));
    assert_eq!(*delays.lock().unwrap(), vec![Duration::from_secs(1)]);
}

#[tokio::test]
async fn in_flight_409_with_retry_after_over_60s_falls_back_to_backoff() {
    let server = MockServer::start().await;
    mount_sequence(&server, "DELETE", PARTNER_FEE, in_flight("120"), success()).await;
    let (client, delays) = client_with(&server, 2);

    client.partner_fees.delete("pf_000000000000").await.unwrap();

    let delays = delays.lock().unwrap();
    assert_eq!(delays.len(), 1);
    // backoff(0) = 0.5 s × U[0.5, 1.0)
    assert!(delays[0] >= Duration::from_millis(250) && delays[0] < Duration::from_millis(500));
}

#[tokio::test]
async fn mutation_timeout_then_success_reuses_the_key() {
    let server = MockServer::start().await;
    mount_sequence(
        &server,
        "POST",
        OWNERSHIP,
        success().set_delay(Duration::from_secs(5)),
        success(),
    )
    .await;
    let (client, delays) = client_with(&server, 2);
    let options = RequestOptions::new().timeout(Duration::from_millis(200));

    client
        .instances
        .migrate_ownership_with_options("us_000000000000", &options)
        .await
        .expect("timeout retried under the key");

    let reqs = requests(&server).await;
    assert_eq!(reqs.len(), 2);
    assert_eq!(key_of(&reqs[0]), key_of(&reqs[1]));
    assert!(key_of(&reqs[0]).is_some());
    assert_eq!(delays.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn network_error_surfaces_as_http_error_with_the_key() {
    // Nothing listens on this port: connection refused on every attempt.
    let (sleep, delays) = recorder();
    let client = BlindPay::builder("test_key", INSTANCE)
        .base_url("http://127.0.0.1:9/v1")
        .retry_sleep(sleep)
        .build()
        .unwrap();
    let options = RequestOptions::new().idempotency_key("caller-key-1");

    let err = client
        .instances
        .migrate_ownership_with_options("us_000000000000", &options)
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Http { .. }), "{err:?}");
    assert!(err.is_retryable());
    assert_eq!(err.idempotency_key(), Some("caller-key-1"));
    assert_eq!(
        delays.lock().unwrap().len(),
        2,
        "retried twice under the key"
    );
}

#[tokio::test]
async fn no_retry_on_400_or_422() {
    for (status, message) in [
        (400u16, "idempotency_key_invalid"),
        (422, "idempotency_key_payload_mismatch"),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(OWNERSHIP))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(serde_json::json!({ "success": false, "message": message })),
            )
            .mount(&server)
            .await;
        let (client, delays) = client_with(&server, 2);

        let err = client
            .instances
            .migrate_ownership("us_000000000000")
            .await
            .unwrap_err();

        assert_eq!(requests(&server).await.len(), 1);
        assert!(delays.lock().unwrap().is_empty());
        assert!(!err.is_retryable());
    }
}

#[tokio::test]
async fn get_with_a_terminal_5xx_code_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(RAILS))
        .respond_with(ResponseTemplate::new(502).set_body_json(serde_json::json!({
            "success": false,
            "code": "PAYINS_CARD_PAYMENT_PENDING",
            "message": "card_payment_pending"
        })))
        .mount(&server)
        .await;
    let (client, _) = client_with(&server, 2);

    let err = client.available.get_rails().await.unwrap_err();

    assert_eq!(requests(&server).await.len(), 1);
    assert!(!err.is_retryable());
}

#[tokio::test]
async fn max_retries_zero_disables_retries_and_auto_keys() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(RAILS))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(OWNERSHIP))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;
    let (client, delays) = client_with(&server, 0);

    client.available.get_rails().await.unwrap_err();
    client
        .instances
        .migrate_ownership("us_000000000000")
        .await
        .unwrap_err();

    let reqs = requests(&server).await;
    assert_eq!(reqs.len(), 2, "one attempt each");
    assert!(key_of(&reqs[1]).is_none(), "no auto key without retries");
    assert!(delays.lock().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Request options
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invalid_caller_keys_are_rejected_before_sending() {
    let server = MockServer::start().await;
    let (client, _) = client_with(&server, 2);

    for key in [String::new(), "k".repeat(256)] {
        let options = RequestOptions::new().idempotency_key(key);
        let err = client
            .instances
            .migrate_ownership_with_options("us_000000000000", &options)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Config(_)), "{err:?}");
    }
    assert!(
        requests(&server).await.is_empty(),
        "nothing reached the wire"
    );
}

#[tokio::test]
async fn caller_key_timeout_and_extra_headers_are_forwarded() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(OWNERSHIP))
        .respond_with(success())
        .mount(&server)
        .await;
    let (client, _) = client_with(&server, 2);
    let options = RequestOptions::new()
        .idempotency_key("k".repeat(255))
        .header(
            HeaderName::from_static("x-correlation-id"),
            HeaderValue::from_static("abc"),
        )
        .header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer stolen"),
        );

    client
        .instances
        .migrate_ownership_with_options("us_000000000000", &options)
        .await
        .unwrap();

    let reqs = requests(&server).await;
    let req = &reqs[0];
    assert_eq!(key_of(req), Some("k".repeat(255)));
    assert_eq!(req.headers.get("x-correlation-id").unwrap(), "abc");
    let auth: Vec<_> = req.headers.get_all("authorization").iter().collect();
    assert_eq!(
        auth,
        vec!["Bearer test_key"],
        "reserved headers can't be overridden"
    );
    assert_eq!(req.headers.get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn get_requests_send_no_content_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(RAILS))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let (client, _) = client_with(&server, 2);

    client.available.get_rails().await.unwrap();

    let reqs = requests(&server).await;
    assert!(reqs[0].headers.get("content-type").is_none());
    let ua = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));
    assert_eq!(reqs[0].headers.get("user-agent").unwrap(), ua.as_str());
}

#[test]
fn client_is_send_sync_and_clone() {
    fn assert_traits<T: Send + Sync + Clone + 'static>() {}
    assert_traits::<BlindPay>();
    assert_traits::<RequestOptions>();
    fn assert_error<T: Send + Sync + std::error::Error + 'static>() {}
    assert_error::<Error>();
}

// ---------------------------------------------------------------------------
// Errors: the shared `tests/fixtures/errors/*.json` bodies
// ---------------------------------------------------------------------------

fn error_fixture(name: &str) -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/errors")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap()
}

/// Serves an error fixture to a GET with retries off and returns the error.
async fn get_error(name: &str) -> Error {
    let fixture = error_fixture(name);
    let status = fixture["status"].as_u64().unwrap() as u16;
    let mut template = ResponseTemplate::new(status).insert_header("X-Request-Id", "req_123");
    for (k, v) in fixture["headers"].as_object().into_iter().flatten() {
        template = template.insert_header(k.as_str(), v.as_str().unwrap());
    }
    template = match (fixture.get("body"), fixture.get("raw_body")) {
        (Some(body), _) => template.set_body_json(body.clone()),
        (None, Some(raw)) => template.set_body_raw(raw.as_str().unwrap().to_string(), "text/html"),
        _ => template,
    };
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(RAILS))
        .respond_with(template)
        .mount(&server)
        .await;
    let (client, _) = client_with(&server, 0);
    client.available.get_rails().await.unwrap_err()
}

fn api(err: Error) -> blindpay::ApiError {
    match err {
        Error::Api(e) => *e,
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn error_envelope_exposes_every_field() {
    let e = api(get_error("api_error_envelope.json").await);
    assert_eq!(e.status, 404);
    assert_eq!(e.code.as_deref(), Some("CUSTOMERS_NOT_FOUND"));
    assert_eq!(e.message, "customer_not_found");
    assert_eq!(
        e.description.as_deref(),
        Some("The customer was not found.")
    );
    assert!(e.errors.is_empty());
    assert_eq!(
        e.trace_id.as_deref(),
        Some("0af7651916cd43dd8448eb211c80319c")
    );
    assert_eq!(e.request_id.as_deref(), Some("req_123"));
    assert!(!e.retryable);
    let shown = e.to_string();
    assert!(
        shown.contains("404") && shown.contains("CUSTOMERS_NOT_FOUND"),
        "{shown}"
    );
}

#[tokio::test]
async fn auth_error_without_trace_id_or_errors_decodes() {
    let e = api(get_error("auth_401_no_trace_id.json").await);
    assert_eq!(e.status, 401);
    assert_eq!(e.code.as_deref(), Some("AUTH_INVALID_API_KEY"));
    assert!(e.trace_id.is_none() && e.errors.is_empty());
    assert!(!e.retryable);
    assert!(!e.to_string().contains("test_key"), "never leaks the key");
}

#[tokio::test]
async fn internal_error_without_errors_is_retryable() {
    let e = api(get_error("internal_error_no_errors_500.json").await);
    assert_eq!(e.code.as_deref(), Some("INTERNAL_ERROR"));
    assert!(e.errors.is_empty());
    assert!(e.retryable);
}

#[tokio::test]
async fn idempotency_errors_have_no_code() {
    let e = api(get_error("idempotency_in_flight_409.json").await);
    assert_eq!(e.status, 409);
    assert!(e.code.is_none());
    assert_eq!(e.message, "idempotency_key_in_flight");
    assert_eq!(e.errors.len(), 1);
    assert!(e.retryable);
    assert_eq!(e.retry_after, Some(Duration::from_secs(1)));

    let e = api(get_error("idempotency_payload_mismatch_422.json").await);
    assert_eq!((e.status, e.retryable), (422, false));
    let e = api(get_error("idempotency_key_invalid_400.json").await);
    assert_eq!((e.status, e.retryable), (400, false));
}

#[tokio::test]
async fn non_json_502_yields_a_status_error_not_a_parse_error() {
    let e = api(get_error("non_json_502.json").await);
    assert_eq!(e.status, 502);
    assert_eq!(e.message, "HTTP 502 Bad Gateway");
    assert!(e.code.is_none());
    assert!(e.raw_body.contains("<h1>502 Bad Gateway</h1>"));
    assert!(e.retryable);
}

#[tokio::test]
async fn rate_limited_429_parses_retry_after() {
    let e = api(get_error("rate_limited_429.json").await);
    assert_eq!(e.code.as_deref(), Some("AUTH_RATE_LIMITED"));
    assert_eq!(e.retry_after, Some(Duration::from_secs(2)));
    assert!(e.retryable);
}

#[tokio::test]
async fn empty_204_is_success_with_no_data() {
    let fixture = error_fixture("empty_204.json");
    assert_eq!(fixture["status"], 204);
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(RAILS))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let (client, _) = client_with(&server, 0);
    assert!(client.available.get_rails().await.unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// A status line arrives but the body is cut off
// ---------------------------------------------------------------------------

/// A raw HTTP server that answers every request with `status` and a
/// `Content-Length` it never fulfils, then hangs up. Returns its base URL and
/// a connection counter.
async fn truncated_body_server(status: &str) -> (String, Arc<std::sync::atomic::AtomicUsize>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{\"mess"
    );
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            let head = head.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let _ = socket.write_all(head.as_bytes()).await;
                let _ = socket.shutdown().await;
            });
        }
    });
    (format!("http://{addr}"), hits)
}

fn truncated_client(base: String) -> BlindPay {
    let (sleep, _) = recorder();
    BlindPay::builder("test_key", INSTANCE)
        .base_url(base)
        .retry_sleep(sleep)
        .build()
        .unwrap()
}

#[tokio::test]
async fn a_mutation_whose_5xx_body_is_cut_off_is_not_resent() {
    let (base, hits) = truncated_body_server("503 Service Unavailable").await;
    let err = truncated_client(base)
        .instances
        .migrate_ownership("us_000000000000")
        .await
        .unwrap_err();
    // Classified by the status: a 503 mutation error, retryable, carrying its key.
    assert_eq!(err.status(), Some(503), "{err:?}");
    assert!(err.is_retryable());
    assert!(err.idempotency_key().is_some());
    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_get_whose_5xx_body_is_cut_off_is_retried_by_status() {
    let (base, hits) = truncated_body_server("503 Service Unavailable").await;
    let err = truncated_client(base)
        .available
        .get_rails()
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(503), "{err:?}");
    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 3);
}

#[tokio::test]
async fn a_cut_off_2xx_body_is_a_transport_error_and_never_resent() {
    let (base, hits) = truncated_body_server("200 OK").await;
    let err = truncated_client(base)
        .instances
        .migrate_ownership("us_000000000000")
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Http { .. }), "{err:?}");
    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_key_that_isnt_a_legal_header_value_is_never_sent() {
    let server = MockServer::start().await;
    let (client, _) = client_with(&server, 2);
    for key in ["bad\nkey", "caf\u{e9}", "nul\u{0}"] {
        let err = client
            .instances
            .migrate_ownership_with_options(
                "us_000000000000",
                &RequestOptions::new().idempotency_key(key),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Config(_)), "{key:?}: {err:?}");
    }
    assert!(requests(&server).await.is_empty());
}

// ---------------------------------------------------------------------------
// Row order: an unkeyed mutation is never retryable; a mutation 408 is final
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_unkeyed_mutation_5xx_is_not_retryable() {
    let server = MockServer::start().await;
    mount_sequence(
        &server,
        "POST",
        OWNERSHIP,
        ResponseTemplate::new(503),
        ResponseTemplate::new(503),
    )
    .await;
    let (client, _) = client_with(&server, 0); // no retries -> no auto key
    let err = client
        .instances
        .migrate_ownership("us_000000000000")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(503));
    assert!(err.idempotency_key().is_none());
    assert!(!err.is_retryable());
    assert_eq!(requests(&server).await.len(), 1);
}

#[tokio::test]
async fn a_mutation_408_is_neither_retried_nor_retryable() {
    let server = MockServer::start().await;
    mount_sequence(
        &server,
        "POST",
        OWNERSHIP,
        ResponseTemplate::new(408),
        ResponseTemplate::new(408),
    )
    .await;
    let (client, delays) = client_with(&server, 2);
    let err = client
        .instances
        .migrate_ownership("us_000000000000")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(408));
    assert!(!err.is_retryable());
    assert_eq!(requests(&server).await.len(), 1);
    assert!(delays.lock().unwrap().is_empty());
}

#[tokio::test]
async fn zero_timeouts_are_rejected_before_sending() {
    let server = MockServer::start().await;
    let err = BlindPay::builder("test_key", INSTANCE)
        .timeout(Duration::ZERO)
        .build()
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");

    let (client, _) = client_with(&server, 2);
    let err = client
        .instances
        .migrate_ownership_with_options(
            "us_000000000000",
            &RequestOptions::new().timeout(Duration::ZERO),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
    let err = client
        .with_timeout(Duration::ZERO)
        .available
        .get_rails()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
    assert!(requests(&server).await.is_empty());
}

/// Cancellation is dropping the future: a 429 / in-flight 409 wait is just an
/// `.await` on the retry sleep, so dropping the call ends it and nothing more
/// is sent.
#[tokio::test]
async fn dropping_the_call_during_a_429_wait_sends_nothing_more() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(OWNERSHIP))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "30"))
        .mount(&server)
        .await;
    let started = Arc::new(tokio::sync::Notify::new());
    let signal = Arc::clone(&started);
    let client = BlindPay::builder("test_key", INSTANCE)
        .base_url(server.uri())
        .retry_sleep(Arc::new(move |_| {
            let signal = Arc::clone(&signal);
            Box::pin(async move {
                signal.notify_one();
                std::future::pending::<()>().await;
            })
        }))
        .build()
        .unwrap();
    let call = client.instances.migrate_ownership("us_000000000000");
    tokio::select! {
        _ = call => panic!("the call should be waiting out Retry-After"),
        _ = started.notified() => {}
    }
    tokio::task::yield_now().await;
    assert_eq!(requests(&server).await.len(), 1);
}

#[tokio::test]
async fn a_2xx_decode_failure_keeps_the_status_and_key() {
    let server = MockServer::start().await;
    let bad = ResponseTemplate::new(201).set_body_string("not json");
    mount_sequence(&server, "POST", OWNERSHIP, bad.clone(), bad).await;
    let (client, _) = client_with(&server, 2);
    let err = client
        .instances
        .migrate_ownership_with_options(
            "us_000000000000",
            &RequestOptions::new().idempotency_key("own-1"),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Decode { .. }), "{err:?}");
    assert_eq!(err.status(), Some(201));
    assert_eq!(err.idempotency_key(), Some("own-1"));
    assert_eq!(requests(&server).await.len(), 1);
}
