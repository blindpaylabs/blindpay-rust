//! Fetches and prints the available payment rails.
//!
//! Run with:
//!
//! ```sh
//! BLINDPAY_API_KEY=... BLINDPAY_INSTANCE_ID=... cargo run --example get_rails
//! ```

use blindpay::BlindPay;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("BLINDPAY_API_KEY")?;
    let instance_id = std::env::var("BLINDPAY_INSTANCE_ID")?;

    let client = BlindPay::new(api_key, instance_id)?;

    let rails = client.available.get_rails().await?;
    println!("Found {} payment rails:", rails.len());
    for rail in rails {
        println!("- {} ({}) in {}", rail.label, rail.value, rail.country);
    }

    Ok(())
}
