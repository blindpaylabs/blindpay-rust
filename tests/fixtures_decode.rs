//! Spec-derived decode test.
//!
//! `tests/fixtures/` is a verbatim copy of the shared, spec-shaped fixture set
//! (`sdk-parity/reference/fixtures`, generated from the public OpenAPI spec).
//! For every operation in `tests/fixtures/index.json` this decodes every
//! response variant (`full`, `required_only`, `nullables_null`,
//! `unknown_enum`, `decimals`, and both list shapes) with the exact type the
//! SDK method returns, using the same `serde_json::from_slice` call as the HTTP
//! layer. Hand-written fixtures share the code's assumptions; these don't.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use blindpay::*;
use serde::de::DeserializeOwned;
use serde_json::Value;

type Outcome = std::result::Result<(), String>;
type Decoder = fn(&[u8]) -> Outcome;

/// Decodes `bytes` as `T`, exactly like `Inner::request` does.
fn decode_as<T: DeserializeOwned>(bytes: &[u8]) -> Outcome {
    serde_json::from_slice::<T>(bytes)
        .map(drop)
        .map_err(|e| e.to_string())
}

/// Decodes a list response. `ListResponse` is `#[serde(untagged)]`, whose
/// error hides the real cause, so on failure re-decode each item as `T` to
/// report the field that broke.
fn decode_list_as<T: DeserializeOwned>(bytes: &[u8]) -> Outcome {
    let Err(outer) = serde_json::from_slice::<ListResponse<T>>(bytes) else {
        return Ok(());
    };
    let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let items = match &value {
        Value::Array(items) => items.clone(),
        Value::Object(map) => {
            if let Some(p) = map.get("pagination") {
                serde_json::from_value::<Pagination>(p.clone())
                    .map_err(|e| format!("pagination: {e}"))?;
            }
            map.get("data")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        }
        _ => Vec::new(),
    };
    for (i, item) in items.into_iter().enumerate() {
        serde_json::from_value::<T>(item).map_err(|e| format!("item[{i}]: {e}"))?;
    }
    Err(format!("ListResponse: {outer}"))
}

/// `"METHOD /v1/path"` (as in `index.json`) → the SDK method's return type.
const DECODERS: &[(&str, Decoder)] = &[
    // available
    ("GET /v1/available/rails", decode_as::<Vec<RailEntry>>),
    (
        "GET /v1/available/bank-details",
        decode_as::<Vec<BankDetail>>,
    ),
    ("GET /v1/available/naics", decode_as::<Vec<NaicsCode>>),
    (
        "GET /v1/available/swift/{swift}",
        decode_as::<Vec<SwiftCodeBankDetail>>,
    ),
    // bank accounts
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts",
        decode_as::<Vec<BankAccount>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{id}",
        decode_as::<BankAccount>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{id}",
        decode_as::<Success>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts",
        decode_as::<BankAccount>,
    ),
    // customers
    (
        "GET /v1/instances/{instance_id}/customers",
        decode_list_as::<Customer>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers",
        decode_as::<CreateCustomerResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{id}",
        decode_as::<Customer>,
    ),
    (
        "PUT /v1/instances/{instance_id}/customers/{id}",
        decode_as::<Success>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{id}",
        decode_as::<Success>,
    ),
    (
        "GET /v1/instances/{instance_id}/limits/customers/{id}",
        decode_as::<GetCustomerLimitsResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/limit-increase",
        decode_as::<Vec<LimitIncrease>>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/limit-increase",
        decode_as::<RequestLimitIncreaseResponse>,
    ),
    // fees
    (
        "GET /v1/instances/{instance_id}/billing/fees",
        decode_as::<Fee>,
    ),
    // instances
    ("GET /v1/instances/{id}/members", decode_as::<Vec<Member>>),
    ("PUT /v1/instances/{id}", decode_as::<Success>),
    ("DELETE /v1/instances/{id}", decode_as::<Success>),
    (
        "PUT /v1/instances/{id}/members/{user_id}",
        decode_as::<Success>,
    ),
    (
        "DELETE /v1/instances/{id}/members/{user_id}",
        decode_as::<Success>,
    ),
    ("POST /v1/instances/{id}/ownership", decode_as::<Success>),
    (
        "POST /v1/e/instances/{instance_id}/tos",
        decode_as::<InitiateTosResponse>,
    ),
    // webhook endpoints
    (
        "GET /v1/instances/{instance_id}/webhook-endpoints",
        decode_as::<Vec<WebhookEndpoint>>,
    ),
    (
        "POST /v1/instances/{instance_id}/webhook-endpoints",
        decode_as::<CreateWebhookEndpointResponse>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/webhook-endpoints/{id}",
        decode_as::<Success>,
    ),
    (
        "GET /v1/instances/{instance_id}/webhook-endpoints/{id}/secret",
        decode_as::<WebhookEndpointSecret>,
    ),
    (
        "GET /v1/instances/{instance_id}/webhook-endpoints/portal-access",
        decode_as::<PortalAccess>,
    ),
    // partner fees
    (
        "GET /v1/instances/{instance_id}/partner-fees",
        decode_as::<Vec<PartnerFee>>,
    ),
    (
        "GET /v1/instances/{instance_id}/partner-fees/{id}",
        decode_as::<PartnerFee>,
    ),
    (
        "POST /v1/instances/{instance_id}/partner-fees",
        decode_as::<PartnerFee>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/partner-fees/{id}",
        decode_as::<Success>,
    ),
    // payins
    (
        "POST /v1/instances/{instance_id}/payins/evm",
        decode_as::<CreatePayinResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/payins",
        decode_list_as::<Payin>,
    ),
    (
        "GET /v1/instances/{instance_id}/payins/{id}",
        decode_as::<Payin>,
    ),
    ("GET /v1/e/payins/{id}", decode_as::<Payin>),
    (
        "POST /v1/instances/{instance_id}/payin-quotes",
        decode_as::<PayinQuote>,
    ),
    (
        "POST /v1/instances/{instance_id}/payin-quotes/fx",
        decode_as::<PayinQuoteFx>,
    ),
    // payouts
    (
        "POST /v1/instances/{instance_id}/payouts/evm",
        decode_as::<CreatePayoutResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/solana",
        decode_as::<CreatePayoutResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/stellar",
        decode_as::<CreatePayoutResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/stellar/authorize",
        decode_as::<AuthorizeStellarTokenResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/{id}/documents",
        decode_as::<SubmitPayoutDocumentsResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/payouts",
        decode_list_as::<Payout>,
    ),
    (
        "GET /v1/instances/{instance_id}/payouts/{id}",
        decode_as::<Payout>,
    ),
    ("GET /v1/e/payouts/{id}", decode_as::<Payout>),
    // quotes
    (
        "POST /v1/instances/{instance_id}/quotes",
        decode_as::<Quote>,
    ),
    (
        "POST /v1/instances/{instance_id}/quotes/fx",
        decode_as::<QuoteFx>,
    ),
    // transfers
    (
        "POST /v1/instances/{instance_id}/transfers",
        decode_as::<CreateTransferResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/transfers",
        decode_list_as::<Transfer>,
    ),
    (
        "GET /v1/instances/{instance_id}/transfers/{id}",
        decode_as::<Transfer>,
    ),
    ("GET /v1/e/transfers/{id}", decode_as::<Transfer>),
    (
        "POST /v1/instances/{instance_id}/transfer-quotes",
        decode_as::<TransferQuote>,
    ),
    // upload
    ("POST /v1/upload", decode_as::<UploadResponse>),
    // virtual accounts
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts",
        decode_as::<Vec<VirtualAccount>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}",
        decode_as::<Option<VirtualAccount>>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts",
        decode_as::<VirtualAccount>,
    ),
    (
        "PUT /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}",
        decode_as::<Success>,
    ),
    // blockchain wallets
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets",
        decode_as::<Vec<BlockchainWallet>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/sign-message",
        decode_as::<BlockchainWalletMessage>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}",
        decode_as::<BlockchainWallet>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets",
        decode_as::<BlockchainWallet>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}",
        decode_as::<Success>,
    ),
    (
        "POST /v1/instances/{instance_id}/create-asset-trustline",
        decode_as::<CreateAssetTrustlineResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/mint-usdb-stellar",
        decode_as::<Success>,
    ),
    (
        "POST /v1/instances/{instance_id}/mint-usdb-solana",
        decode_as::<MintUsdbSolanaResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/prepare-delegate-solana",
        decode_as::<PrepareSolanaDelegationResponse>,
    ),
    // custodial wallets
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets",
        decode_as::<Vec<Wallet>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}",
        decode_as::<Wallet>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}/balance",
        decode_as::<WalletBalance>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/wallets",
        decode_as::<Wallet>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}",
        decode_as::<Success>,
    ),
    // offramp wallets
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets",
        decode_as::<Vec<OfframpWallet>>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets",
        decode_as::<OfframpWallet>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}",
        decode_as::<OfframpWallet>,
    ),
];

/// Surface ops the SDK doesn't implement yet (milestone M3). Each one moves
/// into [`DECODERS`] when its method lands.
const NOT_YET_IMPLEMENTED: &[&str] = &[
    "GET /v1/instances/{id}",
    "GET /v1/instances/{instance_id}/onboarding/business_details",
    "GET /v1/instances/{instance_id}/onboarding/business_profile",
    "GET /v1/instances/{instance_id}/onboarding/ownership_documents",
    "GET /v1/instances/{instance_id}/rfi",
    "POST /v1/instances/{instance_id}/rfi",
    "GET /v1/instances/{instance_id}/customers/{customer_id}/rfi",
    "POST /v1/instances/{instance_id}/customers/{customer_id}/rfi",
    "POST /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/plaid",
    "PATCH /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}",
    "PUT /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}",
    "POST /v1/instances/{instance_id}/payables",
    "GET /v1/instances/{instance_id}/payables",
    "GET /v1/instances/{instance_id}/payables/{id}",
    "DELETE /v1/instances/{instance_id}/payables/{id}",
    "POST /v1/presign",
    "POST /v1/upload/analyze",
    "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}/deposits",
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_json(rel: &str) -> Value {
    let path = fixtures_dir().join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn index_operations() -> serde_json::Map<String, Value> {
    read_json("index.json")["operations"]
        .as_object()
        .expect("index.json has an `operations` object")
        .clone()
}

/// Loads the fixture file `index.json` maps `op` to.
fn fixture_for(op: &str) -> Value {
    let ops = index_operations();
    let file = ops
        .get(op)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{op}: not in index.json"));
    read_json(file)
}

/// Decodes the 200 response variant `name` of `op` as `T`. A bare-array body
/// (list ops) decodes its first item.
fn variant<T: DeserializeOwned>(op: &str, name: &str) -> T {
    let fixture = fixture_for(op);
    let body = &fixture["responses"]["200"]["variants"][name];
    assert!(!body.is_null(), "{op}: no `{name}` variant");
    let body = match body {
        Value::Array(items) => items[0].clone(),
        other => other.clone(),
    };
    serde_json::from_value(body).unwrap_or_else(|e| panic!("{op} {name}: {e}"))
}

#[test]
fn every_surface_op_is_decoded_or_explicitly_pending() {
    let index: BTreeSet<String> = index_operations().keys().cloned().collect();
    assert_eq!(index.len(), 94, "fixture index should list the 94 SDK ops");

    let mapped: BTreeSet<String> = DECODERS.iter().map(|(op, _)| op.to_string()).collect();
    let pending: BTreeSet<String> = NOT_YET_IMPLEMENTED
        .iter()
        .map(|op| op.to_string())
        .collect();
    assert_eq!(mapped.len(), DECODERS.len(), "duplicate op in DECODERS");
    assert!(
        mapped.is_disjoint(&pending),
        "op both mapped and pending: {:?}",
        mapped.intersection(&pending).collect::<Vec<_>>()
    );

    let covered: BTreeSet<String> = mapped.union(&pending).cloned().collect();
    let unknown: Vec<_> = covered.difference(&index).collect();
    let missing: Vec<_> = index.difference(&covered).collect();
    assert!(unknown.is_empty(), "ops not in index.json: {unknown:?}");
    assert!(missing.is_empty(), "index ops with no decoder: {missing:?}");
}

#[test]
fn every_response_variant_decodes() {
    let mut failures = Vec::new();
    let mut decoded = 0usize;

    for (op, decoder) in DECODERS {
        let fixture = fixture_for(op);
        let responses = fixture["responses"]
            .as_object()
            .unwrap_or_else(|| panic!("{op}: no `responses` object"));
        for (status, response) in responses {
            if response["empty_body"] == Value::Bool(true) {
                // Mirror the HTTP layer's empty-body fallback.
                let ok = decoder(b"null").or_else(|_| decoder(b"[]"));
                if let Err(e) = ok {
                    failures.push(format!("{op} [{status} empty]: {e}"));
                }
                decoded += 1;
                continue;
            }
            let variants = response["variants"]
                .as_object()
                .unwrap_or_else(|| panic!("{op} {status}: neither `variants` nor `empty_body`"));
            assert!(!variants.is_empty(), "{op} {status}: no variants");
            for (name, body) in variants {
                let bytes = serde_json::to_vec(body).expect("re-serialize fixture");
                if let Err(e) = decoder(&bytes) {
                    failures.push(format!("{op} [{status} {name}]: {e}"));
                }
                decoded += 1;
            }
        }
    }

    println!(
        "decoded {decoded} variants across {} ops ({} failed)",
        DECODERS.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {decoded} fixture variants failed to decode:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A decode-only test can't catch a misnamed *optional* field: it just comes
/// back `None`. Assert the renamed `customer_*` fields (and the other fields
/// this fix touched) are actually populated from the spec's `full` shape.
#[test]
fn renamed_and_retyped_fields_are_populated_from_full_variants() {
    let payout: Payout = variant("GET /v1/instances/{instance_id}/payouts/{id}", "full");
    assert!(!payout.customer_id.is_empty());
    assert!(payout.customer_local_amount.is_some());
    assert!(payout.commercial_quotation.is_some());
    assert!(payout.blindpay_quotation.is_some());
    assert!(payout.total_fee_amount.is_some());

    let created: CreatePayoutResponse =
        variant("POST /v1/instances/{instance_id}/payouts/evm", "full");
    assert!(created.customer_id.is_some());

    let payin: Payin = variant("GET /v1/instances/{instance_id}/payins/{id}", "full");
    assert!(!payin.customer_id.is_empty());
    assert!(payin.created_at.is_some());
    assert!(payin.commercial_quotation.is_some());

    let created: CreatePayinResponse =
        variant("POST /v1/instances/{instance_id}/payins/evm", "full");
    assert!(created.customer_id.is_some());
    assert!(created.blindpay_bank_details.routing_number.is_some());

    let transfer: Transfer = variant("GET /v1/instances/{instance_id}/transfers/{id}", "full");
    assert!(!transfer.customer_id.is_empty());
    assert!(!transfer.customer_wallet_address.is_empty());
    assert!(transfer.tracking_partner_fee.is_some());
    assert!(transfer.address.is_some());

    let wallet: BlockchainWallet = variant(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}",
        "full",
    );
    assert!(!wallet.customer_id.is_empty());
    assert!(wallet.name.is_some() && wallet.network.is_some());
    assert!(wallet.is_account_abstraction.is_some());

    let request: LimitIncrease = variant(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/limit-increase",
        "full",
    );
    assert!(!request.customer_id.is_empty());

    let offramp: OfframpWallet = variant(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}",
        "full",
    );
    assert!(offramp.customer_id.is_some());

    let quote: Quote = variant("POST /v1/instances/{instance_id}/quotes", "full");
    assert!(quote.customer_local_amount.is_some());
    assert!(quote.expires_at.is_some());

    let va: VirtualAccount = variant(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}",
        "full",
    );
    assert!(va.us.is_some() && va.co.is_some() && va.mx.is_some());
    assert!(va.banking_partner.is_some() && va.kyc_status.is_some());
}

/// The `decimals` variant must keep FX rates fractional, not truncate them.
#[test]
fn payout_fx_rates_keep_their_fraction() {
    let payout: Payout = variant("GET /v1/instances/{instance_id}/payouts/{id}", "decimals");
    assert_eq!(payout.commercial_quotation, Some(1234.56));
    assert_eq!(payout.blindpay_quotation, Some(1234.56));
}
