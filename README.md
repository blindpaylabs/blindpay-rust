# BlindPay Rust SDK

The official Rust SDK for [BlindPay](https://blindpay.com) — Stablecoin API for global payments.

## Installation

```bash
cargo add blindpay
```

Or add it to your `Cargo.toml`:

```toml
[dependencies]
blindpay = "0.1"
```

The SDK is async and runtime-agnostic; the examples below use [Tokio](https://tokio.rs):

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Authentication

To get started, you will need both your API key and your instance id. You can obtain them from the BlindPay dashboard at [https://app.blindpay.com/instances/{instanceId}/api-keys](https://app.blindpay.com/instances/{instanceId}/api-keys).

```rust,no_run
use blindpay::BlindPay;

let blindpay = BlindPay::new("your-api-key-here", "your-instance-id-here")?;
# let _ = blindpay;
# Ok::<(), blindpay::Error>(())
```

> **Note**
> All API calls use the provided API key and instance id.

## Quick Start

### Check for available rails

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

## Response format

Every method returns a [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html), so you handle success and failure with ordinary Rust control flow — `?`, `match`, or the `Result` combinators.

- **Success** resolves to the typed response (for example `Vec<RailEntry>` or `Customer`).
- **Failure** is a `blindpay::Error`. API responses with a non-2xx status are `Error::Api`, which carries the HTTP `status`, the API `message`, and the raw response `body`. Other variants cover transport (`Error::Http`), response decoding (`Error::Decode`), and client configuration (`Error::Config`).

## Error handling

Always handle the `Result`. Match on the error to react to specific cases:

```rust,no_run
use blindpay::{BlindPay, Error};

#[tokio::main]
async fn main() {
    let blindpay = BlindPay::new("your-api-key-here", "your-instance-id-here").unwrap();

    match blindpay.available.get_rails().await {
        Ok(rails) => println!("Success: {} rails", rails.len()),
        Err(Error::Api(err)) => eprintln!("API error {}: {}", err.status, err.message),
        Err(err) => eprintln!("Error: {err}"),
    }
}
```

## Configuration

Use the builder for a custom timeout, base URL, or a preconfigured `reqwest::Client`:

```rust,no_run
use std::time::Duration;
use blindpay::BlindPay;

let blindpay = BlindPay::builder("your-api-key-here", "your-instance-id-here")
    .timeout(Duration::from_secs(10))
    .build()?;
# let _ = blindpay;
# Ok::<(), blindpay::Error>(())
```

### Cargo features

- `rustls-tls` *(default)* — TLS via [rustls](https://github.com/rustls/rustls).
- `native-tls` — TLS via the platform's native library.

For detailed API documentation, visit:
- [BlindPay API documentation](https://blindpay.com/docs/getting-started/overview)
- [API Reference](https://api.blindpay.com/reference)

## Support

- 📧 Email: [alves@blindpay.com](mailto:alves@blindpay.com)
- 🐛 Issues: [GitHub Issues](https://github.com/blindpaylabs/blindpay-rust/issues)

## License

This project is licensed under the MIT License — see the [LICENSE](https://github.com/blindpaylabs/blindpay-rust/blob/main/LICENSE) file for details.

Made with ❤️ by the [BlindPay](https://blindpay.com) team
