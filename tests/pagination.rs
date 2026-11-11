//! Auto-pagination (plan C5, D3/D4, Appendix A7) and client concurrency (C12).

use std::path::Path;
use std::time::Duration;

use blindpay::{BlindPay, Error, Limit, ListPayoutsParams, PaginationParams};
use serde_json::{Value, json};
use wiremock::matchers::{method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PAYOUTS: &str = "/v1/instances/in_000000000000/payouts";

fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test_key", "in_000000000000")
        .base_url(format!("{}/v1", server.uri()))
        .retry_sleep(std::sync::Arc::new(|_| Box::pin(async {})))
        .build()
        .unwrap()
}

/// A spec-shaped payout (the `full` fixture item) with the given id.
fn payout(id: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let index: Value =
        serde_json::from_slice(&std::fs::read(root.join("index.json")).unwrap()).unwrap();
    let file = index["operations"]["GET /v1/instances/{instance_id}/payouts/{id}"]
        .as_str()
        .unwrap();
    let fixture: Value = serde_json::from_slice(&std::fs::read(root.join(file)).unwrap()).unwrap();
    let mut item = fixture["responses"]["200"]["variants"]["full"].clone();
    item["id"] = json!(id);
    item
}

fn page(ids: &[&str], next: Option<&str>) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "data": ids.iter().map(|id| payout(id)).collect::<Vec<_>>(),
        "pagination": { "has_more": next.is_some(), "next_page": next, "prev_page": null },
    }))
}

#[tokio::test]
async fn walks_every_page_following_next_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(PAYOUTS))
        .and(query_param("limit", "100"))
        .and(query_param("status", "completed"))
        .and(query_param_is_missing("starting_after"))
        .respond_with(page(&["po_1", "po_2"], Some("po_2")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(PAYOUTS))
        .and(query_param("limit", "100"))
        .and(query_param("starting_after", "po_2"))
        .respond_with(page(&["po_3"], None))
        .expect(1)
        .mount(&server)
        .await;

    let params = ListPayoutsParams::new().status(blindpay::TransactionStatus::Completed);
    let mut pager = client(&server).payouts.list_auto_paging(&params);
    let mut ids = Vec::new();
    while let Some(item) = pager.next().await {
        ids.push(item.unwrap().id);
    }
    assert_eq!(ids, ["po_1", "po_2", "po_3"]);
    assert!(pager.next().await.is_none());
}

#[tokio::test]
async fn keeps_the_callers_limit_and_collects_all() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_000000000000/transfers"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": { "has_more": false, "next_page": null, "prev_page": null },
        })))
        .expect(1)
        .mount(&server)
        .await;
    let all = client(&server)
        .transfers
        .list_auto_paging(&PaginationParams::new().limit(Limit::Ten))
        .collect_all()
        .await
        .unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn a_bare_array_is_a_single_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(PAYOUTS))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([payout("po_1")])))
        .expect(1)
        .mount(&server)
        .await;
    let all = client(&server)
        .payouts
        .list_auto_paging(&ListPayoutsParams::new())
        .collect_all()
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn ending_before_is_rejected_without_a_request() {
    let server = MockServer::start().await;
    let mut pager = client(&server)
        .payouts
        .list_auto_paging(&ListPayoutsParams::new().ending_before("po_9"));
    assert!(matches!(pager.next().await, Some(Err(Error::Config(_)))));
    assert!(pager.next().await.is_none());
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn an_error_page_stops_the_pager() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(PAYOUTS))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false, "message": "bad_request"
        })))
        .mount(&server)
        .await;
    let mut pager = client(&server)
        .payouts
        .list_auto_paging(&ListPayoutsParams::new());
    assert!(matches!(pager.next().await, Some(Err(Error::Api(_)))));
    assert!(pager.next().await.is_none());
}

#[tokio::test]
async fn a_list_item_decode_error_names_the_field() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(PAYOUTS))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "id": 1 }])))
        .mount(&server)
        .await;
    let err = client(&server)
        .payouts
        .list(&ListPayoutsParams::new())
        .await
        .unwrap_err();
    let Error::Decode { source, .. } = err else {
        panic!("expected a decode error, got {err:?}");
    };
    assert!(!source.to_string().contains("untagged"), "{source}");
}

#[test]
fn the_client_is_clone_send_sync() {
    fn assert_traits<T: Clone + Send + Sync + 'static>() {}
    assert_traits::<BlindPay>();
    assert_traits::<blindpay::RequestOptions>();
    fn assert_send<T: Send>() {}
    assert_send::<blindpay::Pager<blindpay::Payout>>();
    assert_send::<Error>();
}

#[tokio::test]
async fn dropping_the_future_cancels_a_pending_retry_sleep() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    // The default sleep is tokio's: a 5xx GET backs off for >= 0.25s. Dropping
    // the request future (here via a timeout) stops it mid-sleep.
    let client = BlindPay::builder("test_key", "in_000000000000")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .unwrap();
    let started = std::time::Instant::now();
    let res = tokio::time::timeout(Duration::from_millis(100), client.available.get_rails()).await;
    assert!(res.is_err(), "the request should still be sleeping");
    assert!(started.elapsed() < Duration::from_millis(200));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
