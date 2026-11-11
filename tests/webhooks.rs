//! Webhook verification: the 14 shared Svix vectors, and every
//! spec-shaped webhook fixture decoded through the typed event enum.

use std::path::{Path, PathBuf};
use std::time::Duration;

use blindpay::WebhookEvent;
use blindpay::webhooks::{
    WebhookEventPayload, WebhookVerificationError, construct_webhook_event_at, verify_at,
};
use serde_json::Value;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read(path: PathBuf) -> Value {
    serde_json::from_slice(&std::fs::read(&path).unwrap())
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn header_map(headers: &Value) -> http::HeaderMap {
    let mut map = http::HeaderMap::new();
    for (name, value) in headers.as_object().unwrap() {
        map.insert(
            http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            value.as_str().unwrap().parse().unwrap(),
        );
    }
    map
}

#[test]
fn all_14_svix_vectors_match() {
    let doc = read(fixtures().join("webhook-vectors.json"));
    let tolerance = Duration::from_secs(doc["tolerance_seconds"].as_u64().unwrap());
    let vectors = doc["vectors"].as_array().unwrap();
    assert_eq!(vectors.len(), 14);

    let mut passed = 0;
    for v in vectors {
        let name = v["name"].as_str().unwrap();
        let headers = header_map(&v["headers"]);
        let payload = v["payload"].as_str().unwrap().as_bytes();
        let secret = v["secret"].as_str().unwrap();
        let now = v["now"].as_i64().unwrap();
        let expect_valid = v["expect"] == "valid";

        let ok = verify_at(payload, &headers, secret, tolerance, now);
        assert_eq!(ok, expect_valid, "vector {name}");

        // The vector payloads are minimal (not spec-shaped), so a known event
        // may verify and then fail to decode; an unknown one must not fail.
        let event = construct_webhook_event_at(payload, &headers, secret, tolerance, now);
        match (expect_valid, event) {
            (true, Ok(event)) => {
                if let Some(expected) = v["event"].as_str() {
                    assert_eq!(event.event(), WebhookEvent::from(expected), "vector {name}");
                }
            }
            (true, Err(WebhookVerificationError::Decode(_))) => {
                let raw: Value = serde_json::from_slice(payload).unwrap();
                let event = raw["webhook_event"].as_str().unwrap_or_default();
                assert!(
                    !matches!(WebhookEvent::from(event), WebhookEvent::Unknown(_)),
                    "vector {name}: an unknown event must decode to Unknown"
                );
            }
            (false, Err(WebhookVerificationError::Verification(_))) => {}
            (_, other) => panic!("vector {name}: unexpected {other:?}"),
        }
        passed += 1;
    }
    assert_eq!(passed, 14);
}

#[test]
fn verify_never_panics_on_a_bad_secret() {
    let doc = read(fixtures().join("webhook-vectors.json"));
    let v = &doc["vectors"][0];
    let headers = header_map(&v["headers"]);
    let payload = v["payload"].as_str().unwrap().as_bytes();
    let now = v["now"].as_i64().unwrap();
    for secret in ["", "whsec_", "whsec_!!!not-base64", "not-base64-either%"] {
        assert!(!verify_at(
            payload,
            &headers,
            secret,
            Duration::from_secs(300),
            now
        ));
    }
}

#[test]
fn every_webhook_fixture_variant_decodes_to_its_typed_event() {
    let index = read(fixtures().join("index.json"));
    let webhooks: Vec<_> = index["webhooks"]
        .as_object()
        .expect("index.webhooks")
        .iter()
        .filter(|(name, _)| !matches!(WebhookEvent::from(name.as_str()), WebhookEvent::Unknown(_)))
        .collect();
    assert_eq!(webhooks.len(), 28, "spec has 28 webhook events");

    let mut decoded = 0;
    for (name, file) in webhooks {
        let fixture = read(fixtures().join(file.as_str().unwrap()));
        for (variant, body) in fixture["variants"].as_object().unwrap() {
            let event: WebhookEventPayload = serde_json::from_value(body.clone())
                .unwrap_or_else(|e| panic!("{name} [{variant}]: {e}"));
            assert!(
                !matches!(event, WebhookEventPayload::Unknown(_)),
                "{name} [{variant}] fell through to Unknown"
            );
            assert_eq!(event.event().as_str(), name, "{name} [{variant}]");
            decoded += 1;
        }
    }
    assert!(decoded >= 28 * 5, "decoded {decoded} variants");
}

#[test]
fn an_unknown_event_routes_to_the_unknown_variant() {
    let fixture = read(fixtures().join("webhooks/_unknown_event.json"));
    let body = fixture["variants"]["full"].clone();
    let event: WebhookEventPayload = serde_json::from_value(body.clone()).unwrap();
    match &event {
        WebhookEventPayload::Unknown(raw) => assert_eq!(raw, &body),
        other => panic!("expected Unknown, got {other:?}"),
    }
    assert_eq!(event.event().as_str(), "zz.future_event");
}
