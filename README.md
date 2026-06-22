# blindpay-rust

Official Rust SDK for the [BlindPay](https://www.blindpay.com) payments API.

## Installation

```toml
[dependencies]
blindpay = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The SDK is async and runtime-agnostic; the examples below use `tokio`.

## Quick start

```rust,no_run
use blindpay::{BlindPay, Rail};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get your API key and instance ID from the BlindPay dashboard.
    let client = BlindPay::new("your-api-key", "your-instance-id")?;

    // List the available payment rails.
    let rails = client.available.get_rails().await?;
    for rail in &rails {
        println!("{} ({}) — {}", rail.label, rail.value, rail.country);
    }

    // Fetch the bank-detail fields required for a specific rail.
    let fields = client.available.get_bank_details(Rail::Pix).await?;
    for field in &fields {
        println!("{}: required={:?}", field.label, field.required);
    }

    Ok(())
}
```

## Error handling

Every method returns `blindpay::Result<T>`. API responses with a non-2xx status
surface as `Error::Api`, carrying the HTTP status, the API's message, and the
raw response body:

```rust,no_run
use blindpay::{BlindPay, Error};

#[tokio::main]
async fn main() {
    let client = BlindPay::new("your-api-key", "your-instance-id").unwrap();
    match client.available.get_rails().await {
        Ok(rails) => println!("{} rails", rails.len()),
        Err(Error::Api(err)) => eprintln!("api error {}: {}", err.status, err.message),
        Err(err) => eprintln!("error: {err}"),
    }
}
```

## Configuration

Use the builder for a custom base URL, timeout, or `reqwest::Client`:

```rust,no_run
use std::time::Duration;
use blindpay::BlindPay;

let _client = BlindPay::builder("your-api-key", "your-instance-id")
    .timeout(Duration::from_secs(10))
    .build()
    .unwrap();
```

## Working with customers

```rust,no_run
use blindpay::{BlindPay, ListBankAccountsParams, ListCustomersParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BlindPay::new("your-api-key", "your-instance-id")?;

    // List customers (paginated).
    let page = client.customers.list(&ListCustomersParams::default()).await?;
    for customer in page.data() {
        println!("{}: {:?}", customer.id, customer.kyc_status);
    }

    // Fetch a single customer and its bank accounts.
    let customer = client.customers.get("cu_000000000000").await?;
    println!("{:?}", customer.kyc_status);

    let accounts = client
        .customers
        .bank_accounts
        .list("cu_000000000000", &ListBankAccountsParams::default())
        .await?;
    println!("{} bank account(s)", accounts.len());

    Ok(())
}
```

## Working with payouts

```rust,no_run
use blindpay::{BlindPay, ListPayoutsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BlindPay::new("your-api-key", "your-instance-id")?;

    // List payouts (paginated).
    let page = client.payouts.list(&ListPayoutsParams::default()).await?;
    for payout in page.data() {
        println!("{} — {:?}", payout.id, payout.status);
    }

    // Fetch a single payout and its live tracking status.
    let payout = client.payouts.get("po_000000000000").await?;
    let tracked = client.payouts.get_track("po_000000000000").await?;
    println!("{:?} / {:?}", payout.status, tracked.status);

    Ok(())
}
```

## Resources

Each resource is a field on the [`BlindPay`] client. Some expose nested
sub-resources (e.g. `client.customers.bank_accounts`, `client.wallets.blockchain`).

| Resource                       | Methods                                                                                                                                                                                                                                                                                          |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `available`                    | `get_rails`, `get_bank_details`, `get_naics_codes`, `get_swift_code_bank_details`                                                                                                                                                                                                                |
| `instances`                    | `get_members`, `update`, `delete`, `update_member_role`, `delete_member`, `migrate_ownership`                                                                                                                                                                                                    |
| `instances.tos`                | `initiate`                                                                                                                                                                                                                                                                                       |
| `instances.webhook_endpoints`  | `list`, `create`, `delete`, `get_secret`, `get_portal_access_url`                                                                                                                                                                                                                                |
| `partner_fees`                 | `list`, `get`, `create`, `delete`                                                                                                                                                                                                                                                                |
| `fees`                         | `get`                                                                                                                                                                                                                                                                                            |
| `customers`                    | `list`, `create_individual_with_standard_kyc`, `create_individual_with_enhanced_kyc`, `create_business_with_standard_kyb`, `get`, `update`, `delete`, `get_limits`, `get_limit_increase_requests`, `request_limit_increase`                                                                       |
| `customers.bank_accounts`      | `list`, `get`, `delete`, `create_pix`, `create_pix_safe`, `create_ted`, `create_spei_bitso`, `create_transfers_bitso`, `create_ach_cop_bitso`, `create_ach`, `create_wire`, `create_rtp`, `create_international_swift`, `create_sepa`                                                             |
| `wallets.blockchain`           | `list`, `get`, `get_wallet_message`, `create_with_address`, `create_with_hash`, `delete`                                                                                                                                                                                                         |
| `wallets.custodial`            | `list`, `get`, `get_balance`, `create`, `delete`                                                                                                                                                                                                                                                 |
| `wallets.offramp`              | `list`, `get`, `create`                                                                                                                                                                                                                                                                          |
| `virtual_accounts`             | `list`, `get`, `create`, `update`                                                                                                                                                                                                                                                                |
| `quotes`                       | `create`, `get_fx_rate`                                                                                                                                                                                                                                                                          |
| `payins`                       | `create_evm`, `list`, `get`, `get_track`                                                                                                                                                                                                                                                         |
| `payins.quotes`                | `create`, `get_fx_rate`                                                                                                                                                                                                                                                                          |
| `payouts`                      | `create_evm`, `create_solana`, `create_stellar`, `authorize_stellar_token`, `submit_documents`, `list`, `get`, `get_track`                                                                                                                                                                       |
| `transfers`                    | `create`, `list`, `get`, `get_track`                                                                                                                                                                                                                                                             |
| `transfers.quotes`             | `create`                                                                                                                                                                                                                                                                                         |
| `upload`                       | `upload`                                                                                                                                                                                                                                                                                         |

## Cargo features

- `rustls-tls` *(default)* — TLS via [rustls](https://github.com/rustls/rustls).
- `native-tls` — TLS via the platform's native library.

## Releasing

Releases are automated with [release-plz](https://release-plz.dev). On every push
to `main` it opens a release PR (version bump + changelog); merging that PR tags
the release and publishes to crates.io. Publishing requires the
`CARGO_REGISTRY_TOKEN` repository secret to be set (a crates.io API token).

## License

Licensed under the [MIT license](https://github.com/blindpaylabs/blindpay-rust/blob/main/LICENSE).
