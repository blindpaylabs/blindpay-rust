//! E2: fund a BlindPay-managed wallet with a payin, then pay out from it by
//! `wallet_id`.
//!
//! Sequence: create a managed wallet (`bl_`) → payin quote crediting it →
//! execute the payin → wait until it completes → payout quote to a bank
//! account → payout from the managed wallet (`wallet_id`, no on-chain
//! approval) → wait for a final status.
//!
//! Meant for a development instance (test token USDB, no real money). There, a
//! payout quote for `66600` (666.00) ends `failed` and `77700` ends `refunded`;
//! set `BLINDPAY_PAYOUT_AMOUNT` to try them. Production webhooks are the better
//! way to wait; this example polls slowly to stay within the development rate
//! limit.
//!
//! Run with:
//!
//! ```sh
//! BLINDPAY_API_KEY=... BLINDPAY_INSTANCE_ID=in_... \
//! BLINDPAY_CUSTOMER_ID=re_... BLINDPAY_BANK_ACCOUNT_ID=ba_... \
//! cargo run --example managed_wallet_payout
//! ```

use std::time::Duration;

use blindpay::{
    BlindPay, CreateEvmPayoutInput, CreatePayinInput, CreatePayinQuoteInput, CreateQuoteInput,
    CreateWalletInput, CurrencyType, Network, PaymentMethod, RequestOptions, Token,
    TransactionStatus,
};

const NETWORK: Network = Network::BaseSepolia;
const POLL_EVERY: Duration = Duration::from_secs(5);

fn env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    std::env::var(name).map_err(|_| format!("set {name}").into())
}

fn is_final(status: &TransactionStatus) -> bool {
    matches!(
        status,
        TransactionStatus::Completed | TransactionStatus::Failed | TransactionStatus::Refunded
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BlindPay::new(env("BLINDPAY_API_KEY")?, env("BLINDPAY_INSTANCE_ID")?)?;
    let customer_id = env("BLINDPAY_CUSTOMER_ID")?;
    let bank_account_id = env("BLINDPAY_BANK_ACCOUNT_ID")?;
    let payout_amount: i64 = std::env::var("BLINDPAY_PAYOUT_AMOUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000); // 100.00, in cents

    // 1. A BlindPay-managed wallet for the customer.
    let wallet = client
        .wallets
        .custodial
        .create(
            &customer_id,
            &CreateWalletInput {
                network: NETWORK,
                name: "Example treasury".into(),
                external_id: None,
            },
        )
        .await?;
    println!("wallet {} on {}", wallet.id, wallet.network);

    // 2. A payin quote that credits the managed wallet (wallet_id), then the payin.
    let payin_quote = client
        .payins
        .quotes
        .create(&CreatePayinQuoteInput {
            wallet_id: Some(wallet.id.clone()),
            currency_type: CurrencyType::Sender,
            request_amount: 100_000, // R$ 1,000.00
            payment_method: PaymentMethod::Pix,
            token: Token::Usdb,
            ..CreatePayinQuoteInput::default()
        })
        .await?;
    let payin = client
        .payins
        .create_evm_with_options(
            &CreatePayinInput {
                payin_quote_id: payin_quote.id.clone(),
            },
            &RequestOptions::new().idempotency_key(format!("payin-{}", payin_quote.id)),
        )
        .await?;
    println!("payin {} is {}", payin.id, payin.status);

    // 3. Wait for the funds (about 30 s on a development instance).
    let mut status = payin.status;
    while !is_final(&status) {
        tokio::time::sleep(POLL_EVERY).await;
        status = client.payins.get(&payin.id).await?.status;
    }
    if status != TransactionStatus::Completed {
        return Err(format!("payin {} ended {status}", payin.id).into());
    }
    println!("payin completed");

    // 4. A payout quote to the bank account, paid from the managed wallet.
    let quote = client
        .quotes
        .create(
            &CreateQuoteInput::new(NETWORK, Token::Usdb)
                .bank_account_id(&bank_account_id)
                .currency_type(CurrencyType::Sender)
                .request_amount(payout_amount)
                .cover_fees(false),
        )
        .await?;
    let payout = client
        .payouts
        .create_evm_with_options(
            &CreateEvmPayoutInput::from_wallet(&quote.id, &wallet.id),
            &RequestOptions::new().idempotency_key(format!("payout-{}", quote.id)),
        )
        .await?;
    println!("payout {} is {}", payout.id, payout.status);

    // 5. Wait for a final status. In a service, react to payout.complete instead.
    let mut status = payout.status;
    while !is_final(&status) {
        tokio::time::sleep(POLL_EVERY).await;
        status = client.payouts.get(&payout.id).await?.status;
    }
    match status {
        TransactionStatus::Completed => println!("payout completed"),
        TransactionStatus::Failed => println!("payout failed (66600 forces this on dev)"),
        TransactionStatus::Refunded => println!("payout refunded (77700 forces this on dev)"),
        other => println!("payout ended {other}"),
    }
    Ok(())
}
