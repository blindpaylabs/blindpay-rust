//! E1: a webhook receiver. It verifies each delivery against the raw body,
//! dispatches on the event type and ignores deliveries it has already handled
//! (dedupe on the `svix-id` header).
//!
//! It runs a minimal HTTP/1.1 server on Tokio so it needs no web framework;
//! in a real service, hand the same `handle_delivery` the raw body and headers
//! from axum, actix-web or hyper.
//!
//! Run with:
//!
//! ```sh
//! BLINDPAY_WEBHOOK_SECRET=whsec_... cargo run --example webhook_receiver
//! ```
//!
//! then register `https://<your public URL>/webhooks/blindpay` with
//! `instances.webhook_endpoints.create` (a tunnel such as ngrok works for local
//! testing).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use blindpay::webhooks::{WebhookEventPayload, WebhookVerificationError, construct_webhook_event};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Delivery ids already handled. Use a database table with a unique key in
/// production, so the dedupe survives restarts and works across replicas.
type Seen = Arc<Mutex<HashSet<String>>>;

/// Verifies, dedupes and dispatches one delivery; returns the HTTP status to
/// answer with. Svix redelivers anything that doesn't get a 2xx.
fn handle_delivery(headers: &HeaderMap, raw_body: &[u8], secret: &str, seen: &Seen) -> StatusCode {
    let event = match construct_webhook_event(raw_body, headers, secret) {
        Ok(event) => event,
        // The signature was valid but the payload didn't match its event
        // type: acknowledge it so Svix doesn't redeliver it forever.
        Err(WebhookVerificationError::Decode(err)) => {
            eprintln!("verified delivery with an unexpected shape: {err}");
            return StatusCode::OK;
        }
        // Not authentic (bad signature, stale timestamp, missing headers).
        Err(err) => {
            eprintln!("rejected delivery: {err}");
            return StatusCode::BAD_REQUEST;
        }
    };

    let id = headers
        .get("svix-id")
        .or_else(|| headers.get("webhook-id"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    if !seen.lock().unwrap().insert(id.clone()) {
        println!("duplicate delivery {id}, already handled");
        return StatusCode::OK;
    }

    match event {
        WebhookEventPayload::PayoutUpdate(payout) | WebhookEventPayload::PayoutComplete(payout) => {
            // A failed payout can arrive on either event: branch on status.
            println!("payout {} is {}", payout.id, payout.status);
        }
        WebhookEventPayload::PayinComplete(payin) => {
            println!("payin {} is {}", payin.id, payin.status);
        }
        WebhookEventPayload::WalletInbound(deposit) => {
            println!(
                "{} {} landed in wallet {}",
                deposit.token.amount, deposit.token.symbol, deposit.id
            );
        }
        WebhookEventPayload::CustomerUpdate(customer) => {
            println!("customer {} KYC is {}", customer.id, customer.kyc_status);
        }
        WebhookEventPayload::LimitIncreaseUpdate(request) => {
            println!(
                "limit increase {} is {} ({})",
                request.id,
                request.status,
                request.review_reason.as_deref().unwrap_or("no review note")
            );
        }
        WebhookEventPayload::Unknown(raw) => {
            println!(
                "event this SDK version doesn't model: {}",
                raw["webhook_event"]
            );
        }
        other => println!("{} delivery {id}", other.event()),
    }
    StatusCode::OK
}

/// Reads one HTTP/1.1 request: its headers and exactly `Content-Length` body
/// bytes, untouched (verification needs the raw body).
async fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<(HeaderMap, Vec<u8>)>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
        if buf.len() > 64 * 1024 {
            return Ok(None);
        }
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&chunk[..n]);
    };

    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut headers = HeaderMap::new();
    for line in head.lines().skip(1).filter(|l| !l.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.trim().as_bytes()),
            HeaderValue::from_str(value.trim()),
        ) {
            headers.append(name, value);
        }
    }
    let length: usize = headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok()?.parse().ok())
        .unwrap_or(0);
    if length > 1024 * 1024 {
        return Ok(None);
    }
    let mut body = buf[head_end..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok(None);
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);
    Ok(Some((headers, body)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = std::env::var("BLINDPAY_WEBHOOK_SECRET")?;
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = TcpListener::bind(&addr).await?;
    let seen: Seen = Arc::default();
    println!("listening on http://{addr}/webhooks/blindpay");

    loop {
        let (mut stream, _) = listener.accept().await?;
        let (secret, seen) = (secret.clone(), Arc::clone(&seen));
        tokio::spawn(async move {
            let status = match read_request(&mut stream).await {
                Ok(Some((headers, body))) => handle_delivery(&headers, &body, &secret, &seen),
                _ => StatusCode::BAD_REQUEST,
            };
            let response = format!(
                "HTTP/1.1 {} {}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
                status.as_u16(),
                status.canonical_reason().unwrap_or("")
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
