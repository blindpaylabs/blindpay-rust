# BlindPay Rust SDK

The official Rust SDK for [BlindPay](https://blindpay.com): stablecoin API for global payments.

## Installation

```toml
[dependencies]
blindpay = "0.2"
```

> **Publication pending.** Version 0.2 isn't on crates.io yet: the `blindpay` crate name there
> currently holds an unrelated 0.1.0 release, so `cargo add blindpay` would install the wrong crate.
> Until the name is transferred, depend on this repository:
> `blindpay = { git = "https://github.com/blindpaylabs/blindpay-rust" }`.

The SDK is async and uses [Tokio](https://tokio.rs) (reqwest's runtime, also used for retry
backoff). The examples use:

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Minimum supported Rust version: **1.85**.

## Authentication

You need your API key and instance id, from the BlindPay dashboard at
[https://app.blindpay.com/instances/{instanceId}/api-keys](https://app.blindpay.com/instances/{instanceId}/api-keys).

```rust,no_run
use blindpay::BlindPay;

let blindpay = BlindPay::new("your-api-key-here", "your-instance-id-here")?;
# let _ = blindpay;
# Ok::<(), blindpay::Error>(())
```

Every call uses that key and instance. The client never prints the key: its `Debug` output
redacts it.

## Quick start

```rust,no_run
use blindpay::BlindPay;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blindpay = BlindPay::new("your-api-key-here", "your-instance-id-here")?;

    let rails = blindpay.available.get_rails().await?;
    for rail in &rails {
        println!("{} ({}) in {}", rail.label, rail.value, rail.country);
    }
    Ok(())
}
```

## Configuration

```rust,no_run
use std::time::Duration;
use blindpay::BlindPay;

let blindpay = BlindPay::builder("your-api-key-here", "your-instance-id-here")
    .timeout(Duration::from_secs(30)) // per attempt; default 60 s
    .max_retries(2)                   // default 2 (3 attempts); 0 disables retries
    .build()?;
# let _ = blindpay;
# Ok::<(), blindpay::Error>(())
```

- `base_url(..)`: point the client at a proxy or a mock server (default
  `https://api.blindpay.com/v1`). That URL receives your API key, so only use hosts you trust.
- `http_client(reqwest::Client)`: bring your own client (proxies, custom TLS). The per-attempt
  timeout still applies.
- `retry_sleep(..)`: replace the backoff sleep (handy in tests).

A built `BlindPay` is immutable and `Clone + Send + Sync`: clone it into tasks freely; clones share
one connection pool.

### Cargo features

- `rustls-tls` *(default)*: TLS via [rustls](https://github.com/rustls/rustls).
- `native-tls`: TLS via the platform's native library
  (`default-features = false, features = ["native-tls"]`).

## Errors

Every method returns `blindpay::Result<T>`. A `blindpay::Error` is one of:

- `Error::Api`: the API answered with a non-2xx status. The `ApiError` carries `status`, `code`
  (stable, UPPERCASE, e.g. `PAYOUTS_INSUFFICIENT_BALANCE`; match on this), `message`,
  `description`, `errors`, `trace_id`, `request_id` (the `X-Request-Id` header), the
  `idempotency_key` the request used, `retryable`, `retry_after` and the raw body. Only `status`
  and `message` are always present.
- `Error::Http`: no response arrived (DNS, connection, TLS, timeout). It also carries the
  `idempotency_key`.
- `Error::Decode`: a 2xx body didn't match the expected type.
- `Error::Config`: invalid configuration or argument; nothing was sent.

```rust,no_run
use blindpay::{BlindPay, Error};

# async fn run(blindpay: BlindPay) {
match blindpay.available.get_rails().await {
    Ok(rails) => println!("{} rails", rails.len()),
    Err(Error::Api(err)) if err.code.as_deref() == Some("AUTH_RATE_LIMITED") => {
        eprintln!("slow down; retry after {:?}", err.retry_after);
    }
    Err(Error::Api(err)) => eprintln!("API error {}: {} (trace {:?})", err.status, err.message, err.trace_id),
    Err(err) => eprintln!("error: {err}"),
}
# }
```

Error messages never include your API key or request bodies.

## Idempotency and retries

Every mutating method has a `*_with_options` variant that takes `RequestOptions`
(idempotency key, per-request timeout, extra headers):

```rust,no_run
use blindpay::{BlindPay, CreateEvmPayoutInput, RequestOptions};

# async fn run(blindpay: BlindPay) -> blindpay::Result<()> {
let payout = blindpay
    .payouts
    .create_evm_with_options(
        &CreateEvmPayoutInput::from_wallet("qu_000000000000", "bl_000000000000"),
        &RequestOptions::new().idempotency_key("payout-invoice-2041"),
    )
    .await?;
# let _ = payout;
# Ok(())
# }
```

- A key must be 1–255 characters; an empty or longer key is rejected before anything is sent
  (the API silently ignores an empty key).
- With retries on (the default), the SDK generates a UUIDv4 key before the first attempt of every
  JSON mutation you don't give a key to, so a retry can never execute twice. File uploads
  (multipart) get no key and are never retried.
- **What is retried:** `GET`s on connection errors, timeouts, 408, 429, 500, 502, 503 and 504.
  Mutations only when no response arrived, on 429, and on `409 idempotency_key_in_flight`, always
  with the same key and the same bytes. `Retry-After` up to 60 s is honoured; otherwise backoff is
  `min(8 s, 0.5 s × 2^attempt)` with jitter.
- **Mutations are never retried on a 5xx**: the API releases the key on a 5xx, so a resend would
  run the handler again. You get an error with `is_retryable() == true` and the key.
  **If a create or payout returns a 5xx, look the resource up (by quote, external id, or a list
  filter) before resending. Resend with the same `idempotency_key` from the error.**

```rust,no_run
# use blindpay::{BlindPay, CreateEvmPayoutInput, RequestOptions};
# async fn run(blindpay: BlindPay, input: CreateEvmPayoutInput) -> blindpay::Result<()> {
match blindpay.payouts.create_evm(&input).await {
    Err(err) if err.is_retryable() => {
        let key = err.idempotency_key().map(str::to_owned);
        // Check whether the payout exists first; then, only if it doesn't:
        if let Some(key) = key {
            let options = RequestOptions::new().idempotency_key(key);
            blindpay.payouts.create_evm_with_options(&input, &options).await?;
        }
    }
    other => { other?; }
}
# Ok(())
# }
```

## Pagination

Lists return `ListResponse<T>`: the API sends a bare array when you pass no pagination params,
and a `{data, pagination}` envelope when you do. Read both the same way with `.data()` /
`.into_data()`; `.pagination()` has `has_more` and the `next_page` cursor (pass it back as
`starting_after`).

Unpaginated lists return **every row**, so for payouts, payins, customers, transfers, payables
and wallet deposits prefer the auto-pager. It requests 100 per page (unless you set `limit`) and
follows the cursor:

```rust,no_run
use blindpay::{BlindPay, ListPayoutsParams, TransactionStatus};

# async fn run(blindpay: BlindPay) -> blindpay::Result<()> {
let params = ListPayoutsParams::new().status(TransactionStatus::Completed);
let mut payouts = blindpay.payouts.list_auto_paging(&params);
while let Some(payout) = payouts.next().await {
    let payout = payout?;
    println!("{} {}", payout.id, payout.status);
}
# Ok(())
# }
```

`Pager::next_page()` yields whole pages and `collect_all()` gathers everything.

## Webhooks

Verify deliveries with the standalone verifier: it needs no client, only your endpoint's signing
secret (`whsec_…`, from `instances.webhook_endpoints.get_secret`). Always pass the **raw** request
bytes (axum `Bytes`, actix `web::Bytes`, hyper `body::to_bytes`), never re-serialized JSON.

```rust,no_run
use blindpay::{TransactionStatus, WebhookEventPayload, construct_webhook_event};

fn handle(headers: &http::HeaderMap, raw_body: &[u8], secret: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Dedupe on the `svix-id` header: deliveries can repeat.
    let event = construct_webhook_event(raw_body, headers, secret)?;
    match event {
        // A failed payout can arrive on payout.update *or* payout.complete:
        // branch on status, not on the event name.
        WebhookEventPayload::PayoutUpdate(p) | WebhookEventPayload::PayoutComplete(p)
            if p.status == TransactionStatus::Failed => println!("payout {} failed", p.id),
        WebhookEventPayload::PayoutComplete(p) => println!("payout {} is {}", p.id, p.status),
        WebhookEventPayload::Unknown(raw) => println!("unhandled event: {raw}"),
        _ => {}
    }
    Ok(())
}
```

`blindpay::webhooks::verify(raw_body, headers, secret)` returns a plain `bool` instead. Both
check the HMAC-SHA256 signature in constant time and reject timestamps more than 5 minutes off
(`verify_at` / `construct_webhook_event_at` take an explicit tolerance and clock). Event names this
version doesn't know decode to `WebhookEventPayload::Unknown` with the raw JSON. Never log the
`whsec_` secret.

## Uploads

```rust,no_run
use blindpay::{AnalyzeDocumentType, BlindPay, UploadBucket, UploadFile};

# async fn run(blindpay: BlindPay, pdf: Vec<u8>) -> blindpay::Result<()> {
let file = UploadFile::new("statement.pdf", pdf).content_type("application/pdf");
let uploaded = blindpay.upload.upload(file.clone(), UploadBucket::Documents).await?;
let signed = blindpay.upload.presign(&uploaded.file_url).await?; // temporary URL
let check = blindpay
    .upload
    .analyze(
        file,
        AnalyzeDocumentType::BankStatement,
        Some(&serde_json::json!({ "full_name": "Jane Doe" })),
    )
    .await?;
println!("{} expires {}; approval {}", signed.file_url, signed.expires_at, check.approval_rate);
# Ok(())
# }
```

Uploads authenticate against your own instance (`?instance_id=` is sent for you) and need a
completed onboarding.

## Units

- Payout, payin, quote and limit amounts are **integer cents** (`10000` = 100.00).
- Partner percentage fees are **basis points** (`100` = 1%); flat fees are cents.
- Wallet balances are **whole tokens** (`10.5` = 10.5 USDC). Quotations are decimals.

## Testing against a development instance

Development instances settle with the test token USDB and move no real money. Payout amounts
`66600` fail and `77700` refund; dev payins complete in about 30 s; quotes expire after 5 minutes
(OTC payin quotes after 10 s; SEPA may expire earlier); dev instances allow 40 requests per minute
(429 with `Retry-After`, which the SDK honours).

## Workflows

### Webhook endpoint setup

```rust,no_run
use blindpay::{BlindPay, CreateWebhookEndpointInput, WebhookEvent};

# async fn run(blindpay: BlindPay) -> blindpay::Result<()> {
let endpoint = blindpay
    .instances
    .webhook_endpoints
    .create(&CreateWebhookEndpointInput::new(
        "https://example.com/webhooks/blindpay",
        vec![WebhookEvent::PayoutUpdate, WebhookEvent::PayoutComplete],
    ))
    .await?;
let secret = blindpay.instances.webhook_endpoints.get_secret(&endpoint.id).await?;
// Store secret.key in your secrets manager; verify deliveries as shown above.
# let _ = secret;
# Ok(())
# }
```

### Managed-wallet payout

```rust,no_run
use blindpay::{BlindPay, CreateEvmPayoutInput, CreateQuoteInput, CurrencyType, Network, RequestOptions, Token};

# async fn run(blindpay: BlindPay) -> blindpay::Result<()> {
let quote = blindpay
    .quotes
    .create(
        &CreateQuoteInput::new(Network::Base, Token::Usdc)
            .bank_account_id("ba_000000000000")
            .currency_type(CurrencyType::Sender)
            .request_amount(100_000) // 1,000.00
            .cover_fees(false),
    )
    .await?;
let payout = blindpay
    .payouts
    .create_evm_with_options(
        &CreateEvmPayoutInput::from_wallet(&quote.id, "bl_000000000000"),
        &RequestOptions::new().idempotency_key(format!("payout-{}", quote.id)),
    )
    .await?;
// Then follow payout.update / payout.complete webhooks and branch on status.
# let _ = payout;
# Ok(())
# }
```

### Fiat payin

```rust,no_run
use blindpay::{BlindPay, CreatePayinInput, CreatePayinQuoteInput, CurrencyType, PaymentMethod, Token};

# async fn run(blindpay: BlindPay) -> blindpay::Result<()> {
let quote = blindpay
    .payins
    .quotes
    .create(&CreatePayinQuoteInput {
        blockchain_wallet_id: Some("bw_000000000000".into()),
        currency_type: CurrencyType::Sender,
        request_amount: 50_000, // R$ 500.00
        payment_method: PaymentMethod::Pix,
        token: Token::Usdc,
        ..CreatePayinQuoteInput::default()
    })
    .await?;
let payin = blindpay
    .payins
    .create_evm(&CreatePayinInput { payin_quote_id: quote.id })
    .await?;
# let _ = payin;
# Ok(())
# }
```

### Reconciliation

```rust,no_run
use blindpay::{BlindPay, ListPayoutsParams, TransactionStatus};

# async fn run(blindpay: BlindPay) -> blindpay::Result<()> {
let failed = blindpay
    .payouts
    .list_auto_paging(
        &ListPayoutsParams::new()
            .customer_id("re_000000000000")
            .status(TransactionStatus::Failed),
    )
    .collect_all()
    .await?;
println!("{} failed payouts to review", failed.len());
# Ok(())
# }
```

## Upgrading from 0.2

0.2.0 was never published, but its API changed in this release. Every change below either fixes a
call that could not work against the live API or adds parity with the other BlindPay SDKs.

| 0.2 | Now |
|---|---|
| `receiver_*` fields and filters (`receiver_id`, `receiver_name`, `receiver_wallet_address`, …) | `customer_*` (the API's names) |
| Payout/quote FX rates and other fractional amounts typed `i64` | `f64`; nullable/optional response fields are `Option` |
| `VirtualAccount.us` required | `Option`, plus `co` / `mx` blocks |
| `Error::Api(ApiError)` | `Error::Api(Box<ApiError>)` (fields via deref), with `code`, `description`, `errors`, `trace_id`, `request_id`, `idempotency_key`, `retryable`, `retry_after` |
| `Error::Http(reqwest::Error)` | `Error::Http { source, idempotency_key }` |
| 30 s timeout, no retries | 60 s per attempt, 2 retries (see above); `.max_retries(0)` restores single-shot |
| `svix` dependency, `verify_webhook_signature` only | native verifier; `construct_webhook_event` returns a typed `WebhookEventPayload` |
| `CreateQuoteInput { bank_account_id: String, currency_type, cover_fees: bool, request_amount: i64, .. }` | `CreateQuoteInput::new(network, token).bank_account_id(..)` (or `.payable_id(..)`); those fields are `Option` |
| `CreateEvmPayoutInput { sender_wallet_address: String, .. }` (also Solana/Stellar) | `sender_wallet_address: Option<String>` + `wallet_id`; `::new(quote, address)` still works, `::from_wallet(quote, wallet_id)` is new |
| `CreatePayinQuoteInput.cover_fees: bool` | `Option<bool>` |
| `UpdateCustomerInput { ..Default::default() }` | `UpdateCustomerInput::new(email, country)` (the API requires both) |
| `RequestLimitIncreaseInput { supporting_document_type, supporting_document_file, .. }` | `RequestLimitIncreaseInput::new(per_tx, daily, monthly).supporting_document(kind, url)` |
| `transfers.list(&PaginationParams)` | `transfers.list(&ListTransfersParams)` (`From<PaginationParams>` provided) |
| `payins.get_track(id)` | `payins.get_track(id, checkout_token: Option<&str>)` |
| `upload.upload(file_name, bytes, bucket)` | `upload.upload(UploadFile::new(file_name, bytes).content_type(..), bucket)` |
| Stale response fields (`tracking_partner_fee` on payouts/payins, `PartnerFee.created_at`, …) | deprecated (not in the API spec); removed in a later release |
| Any `instance_id` string | must be a plain path segment (`in_…`); others are rejected by `build()` |
| `WebhookEvent::Receiver*` | removed (the API no longer sends them); new `payable.*`, `wallet.update`, `blockchainWallet.update` events |
| `.offset(..)` on list params | deprecated: the API ignores it; use `starting_after` |

Before / after, for the most common change:

```rust,no_run
# use blindpay::{CreateQuoteInput, CurrencyType, Network, Token};
// 0.2:
// let input = CreateQuoteInput { bank_account_id: "ba_…".into(), currency_type: CurrencyType::Sender,
//     cover_fees: true, request_amount: 1000, network: Network::Base, token: Token::Usdc,
//     description: None, partner_fee_id: None };
// now:
let input = CreateQuoteInput::new(Network::Base, Token::Usdc)
    .bank_account_id("ba_000000000000")
    .currency_type(CurrencyType::Sender)
    .cover_fees(true)
    .request_amount(1000);
# let _ = input;
```

The `/e/` tracking methods (`payouts.get_track`, `payins.get_track`, `transfers.get_track`) serve
BlindPay's own web and iOS apps: server-side calls get 401 in production. Use `get(id)` instead.

## More

- [BlindPay API documentation](https://blindpay.com/docs/getting-started/overview)
- [API reference](https://api.blindpay.com/reference)

## Support

- Email: [support@blindpay.com](mailto:support@blindpay.com)
- Issues: [GitHub Issues](https://github.com/blindpaylabs/blindpay-rust/issues)

## License

MIT. See [LICENSE](https://github.com/blindpaylabs/blindpay-rust/blob/main/LICENSE).
