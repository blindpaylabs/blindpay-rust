//! Shared helpers for the integration tests.

#![allow(dead_code)] // each test crate uses a subset

use std::sync::Arc;

use blindpay::BlindPay;
use wiremock::MockServer;

/// A client for `server` with the given key and instance. Retries back off
/// instantly (tests/transport.rs covers the delays), so no test really sleeps.
pub fn client_for(server: &MockServer, api_key: &str, instance_id: &str) -> BlindPay {
    BlindPay::builder(api_key, instance_id)
        .base_url(server.uri())
        .retry_sleep(Arc::new(|_| Box::pin(async {})))
        .build()
        .expect("client should build")
}

/// The client most tests use: key `test_key`, instance `in_000000000000`.
pub fn client(server: &MockServer) -> BlindPay {
    client_for(server, "test_key", "in_000000000000")
}
