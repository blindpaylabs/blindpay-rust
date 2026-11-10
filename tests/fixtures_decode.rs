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
fn dec<T: DeserializeOwned>(bytes: &[u8]) -> Outcome {
    serde_json::from_slice::<T>(bytes)
        .map(drop)
        .map_err(|e| e.to_string())
}

/// Decodes a list response. `ListResponse` is `#[serde(untagged)]`, whose
/// error hides the real cause, so on failure re-decode each item as `T` to
/// report the field that broke.
fn dec_list<T: DeserializeOwned>(bytes: &[u8]) -> Outcome {
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
    ("GET /v1/available/rails", dec::<Vec<RailEntry>>),
    ("GET /v1/available/bank-details", dec::<Vec<BankDetail>>),
    ("GET /v1/available/naics", dec::<Vec<NaicsCode>>),
    (
        "GET /v1/available/swift/{swift}",
        dec::<Vec<SwiftCodeBankDetail>>,
    ),
    // bank accounts
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts",
        dec::<Vec<BankAccount>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{id}",
        dec::<BankAccount>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{id}",
        dec::<Success>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts",
        dec::<BankAccount>,
    ),
    // customers
    (
        "GET /v1/instances/{instance_id}/customers",
        dec_list::<Customer>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers",
        dec::<CreateCustomerResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{id}",
        dec::<Customer>,
    ),
    (
        "PUT /v1/instances/{instance_id}/customers/{id}",
        dec::<Success>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{id}",
        dec::<Success>,
    ),
    (
        "GET /v1/instances/{instance_id}/limits/customers/{id}",
        dec::<GetCustomerLimitsResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/limit-increase",
        dec::<Vec<LimitIncrease>>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/limit-increase",
        dec::<RequestLimitIncreaseResponse>,
    ),
    // fees
    ("GET /v1/instances/{instance_id}/billing/fees", dec::<Fee>),
    // instances
    ("GET /v1/instances/{id}/members", dec::<Vec<Member>>),
    ("PUT /v1/instances/{id}", dec::<Success>),
    ("DELETE /v1/instances/{id}", dec::<Success>),
    ("PUT /v1/instances/{id}/members/{user_id}", dec::<Success>),
    (
        "DELETE /v1/instances/{id}/members/{user_id}",
        dec::<Success>,
    ),
    ("POST /v1/instances/{id}/ownership", dec::<Success>),
    (
        "POST /v1/e/instances/{instance_id}/tos",
        dec::<InitiateTosResponse>,
    ),
    // webhook endpoints
    (
        "GET /v1/instances/{instance_id}/webhook-endpoints",
        dec::<Vec<WebhookEndpoint>>,
    ),
    (
        "POST /v1/instances/{instance_id}/webhook-endpoints",
        dec::<CreateWebhookEndpointResponse>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/webhook-endpoints/{id}",
        dec::<Success>,
    ),
    (
        "GET /v1/instances/{instance_id}/webhook-endpoints/{id}/secret",
        dec::<WebhookEndpointSecret>,
    ),
    (
        "GET /v1/instances/{instance_id}/webhook-endpoints/portal-access",
        dec::<PortalAccess>,
    ),
    // partner fees
    (
        "GET /v1/instances/{instance_id}/partner-fees",
        dec::<Vec<PartnerFee>>,
    ),
    (
        "GET /v1/instances/{instance_id}/partner-fees/{id}",
        dec::<PartnerFee>,
    ),
    (
        "POST /v1/instances/{instance_id}/partner-fees",
        dec::<PartnerFee>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/partner-fees/{id}",
        dec::<Success>,
    ),
    // payins
    (
        "POST /v1/instances/{instance_id}/payins/evm",
        dec::<CreatePayinResponse>,
    ),
    ("GET /v1/instances/{instance_id}/payins", dec_list::<Payin>),
    ("GET /v1/instances/{instance_id}/payins/{id}", dec::<Payin>),
    ("GET /v1/e/payins/{id}", dec::<Payin>),
    (
        "POST /v1/instances/{instance_id}/payin-quotes",
        dec::<PayinQuote>,
    ),
    (
        "POST /v1/instances/{instance_id}/payin-quotes/fx",
        dec::<PayinQuoteFx>,
    ),
    // payouts
    (
        "POST /v1/instances/{instance_id}/payouts/evm",
        dec::<CreatePayoutResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/solana",
        dec::<CreatePayoutResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/stellar",
        dec::<CreatePayoutResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/stellar/authorize",
        dec::<AuthorizeStellarTokenResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/payouts/{id}/documents",
        dec::<SubmitPayoutDocumentsResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/payouts",
        dec_list::<Payout>,
    ),
    (
        "GET /v1/instances/{instance_id}/payouts/{id}",
        dec::<Payout>,
    ),
    ("GET /v1/e/payouts/{id}", dec::<Payout>),
    // quotes
    ("POST /v1/instances/{instance_id}/quotes", dec::<Quote>),
    ("POST /v1/instances/{instance_id}/quotes/fx", dec::<QuoteFx>),
    // transfers
    (
        "POST /v1/instances/{instance_id}/transfers",
        dec::<CreateTransferResponse>,
    ),
    (
        "GET /v1/instances/{instance_id}/transfers",
        dec_list::<Transfer>,
    ),
    (
        "GET /v1/instances/{instance_id}/transfers/{id}",
        dec::<Transfer>,
    ),
    ("GET /v1/e/transfers/{id}", dec::<Transfer>),
    (
        "POST /v1/instances/{instance_id}/transfer-quotes",
        dec::<TransferQuote>,
    ),
    // upload
    ("POST /v1/upload", dec::<UploadResponse>),
    // virtual accounts
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts",
        dec::<Vec<VirtualAccount>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}",
        dec::<Option<VirtualAccount>>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts",
        dec::<VirtualAccount>,
    ),
    (
        "PUT /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}",
        dec::<Success>,
    ),
    // blockchain wallets
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets",
        dec::<Vec<BlockchainWallet>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/sign-message",
        dec::<BlockchainWalletMessage>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}",
        dec::<BlockchainWallet>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets",
        dec::<BlockchainWallet>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}",
        dec::<Success>,
    ),
    (
        "POST /v1/instances/{instance_id}/create-asset-trustline",
        dec::<CreateAssetTrustlineResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/mint-usdb-stellar",
        dec::<Success>,
    ),
    (
        "POST /v1/instances/{instance_id}/mint-usdb-solana",
        dec::<MintUsdbSolanaResponse>,
    ),
    (
        "POST /v1/instances/{instance_id}/prepare-delegate-solana",
        dec::<PrepareSolanaDelegationResponse>,
    ),
    // custodial wallets
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets",
        dec::<Vec<Wallet>>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}",
        dec::<Wallet>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}/balance",
        dec::<WalletBalance>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/wallets",
        dec::<Wallet>,
    ),
    (
        "DELETE /v1/instances/{instance_id}/customers/{customer_id}/wallets/{id}",
        dec::<Success>,
    ),
    // offramp wallets
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets",
        dec::<Vec<OfframpWallet>>,
    ),
    (
        "POST /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets",
        dec::<OfframpWallet>,
    ),
    (
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}",
        dec::<OfframpWallet>,
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

/// Known defects in the shared fixtures: integer-cent fields the spec types as
/// bare `number` with no field-level example, so the generator synthesized
/// `12.34`. The API source rounds every one of them to an integer, and the
/// plan's D16 forbids retyping cents as floats. The test truncates exactly
/// these values (and fails if an entry stops matching, so the list can't rot).
/// Reported upstream as a `gen_fixtures.py` issue.
const FIXTURE_DEFECTS: &[(&str, &[&str], &str)] = &[
    (
        "GET /v1/instances/{instance_id}/limits/customers/{id}",
        &[
            "/limits/payin/daily",
            "/limits/payin/monthly",
            "/limits/payout/daily",
            "/limits/payout/monthly",
        ],
        "cents; `Math.round` in get.instances.[instance_id].limits.customers.[id].ts:114-130 \
         (schema-level example is 500000)",
    ),
    (
        "POST /v1/instances/{instance_id}/quotes/fx",
        &["/result_amount"],
        "cents; sender/receiver_amount via Math.round/floor in shared/helpers/use-quote.ts:82-98",
    ),
    (
        "POST /v1/instances/{instance_id}/payin-quotes/fx",
        &["/result_amount"],
        "cents; sender/receiver_amount via Math.round/floor in shared/helpers/use-payin-quote.ts:106-126",
    ),
];

/// Truncates the known-defect values in `body`; returns how many it changed.
fn patch_fixture_defects(op: &str, body: &mut Value) -> usize {
    let mut patched = 0;
    for (_, pointers, _) in FIXTURE_DEFECTS.iter().filter(|(o, _, _)| *o == op) {
        for pointer in *pointers {
            // No let-chains: MSRV is 1.85.
            let Some(slot) = body.pointer_mut(pointer) else {
                continue;
            };
            if let Some(f) = slot.as_f64().filter(|f| f.fract() != 0.0) {
                *slot = Value::from(f.trunc() as i64);
                patched += 1;
            }
        }
    }
    patched
}

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
    let ops = index_operations();
    let mut failures = Vec::new();
    let mut decoded = 0usize;
    let mut patched_ops = BTreeSet::new();

    for (op, decoder) in DECODERS {
        let file = ops[*op].as_str().expect("index value is a path");
        let fixture = read_json(file);
        let responses = fixture["responses"]
            .as_object()
            .unwrap_or_else(|| panic!("{file}: no `responses` object"));
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
                .unwrap_or_else(|| panic!("{file} {status}: neither `variants` nor `empty_body`"));
            assert!(!variants.is_empty(), "{file} {status}: no variants");
            for (name, body) in variants {
                let mut body = body.clone();
                if patch_fixture_defects(op, &mut body) > 0 {
                    patched_ops.insert(*op);
                }
                let bytes = serde_json::to_vec(&body).expect("re-serialize fixture");
                if let Err(e) = decoder(&bytes) {
                    failures.push(format!("{op} [{status} {name}]: {e}"));
                }
                decoded += 1;
            }
        }
    }

    for (op, _, reason) in FIXTURE_DEFECTS {
        assert!(
            patched_ops.contains(op),
            "FIXTURE_DEFECTS entry for {op} ({reason}) no longer matches; remove it"
        );
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

/// Decodes the `full` variant of `op` as `T`. For list ops (bare array, or
/// the `full.b1` branch) it decodes the first item.
fn full<T: DeserializeOwned>(op: &str) -> T {
    let ops = index_operations();
    let fixture = read_json(ops[op].as_str().expect("index value is a path"));
    let variants = &fixture["responses"]["200"]["variants"];
    let body = variants.get("full").unwrap_or(&variants["full.b1"]);
    let body = match body {
        Value::Array(items) => items[0].clone(),
        other => other.clone(),
    };
    serde_json::from_value(body).unwrap_or_else(|e| panic!("{op} full: {e}"))
}

/// A decode-only test can't catch a misnamed *optional* field: it just comes
/// back `None`. Assert the renamed `customer_*` fields (and the other fields
/// this fix touched) are actually populated from the spec's `full` shape.
#[test]
fn renamed_and_retyped_fields_are_populated_from_full_variants() {
    let payout: Payout = full("GET /v1/instances/{instance_id}/payouts/{id}");
    assert!(!payout.customer_id.is_empty());
    assert!(payout.customer_local_amount.is_some());
    assert!(payout.commercial_quotation.is_some());
    assert!(payout.blindpay_quotation.is_some());
    assert!(payout.total_fee_amount.is_some());

    let created: CreatePayoutResponse = full("POST /v1/instances/{instance_id}/payouts/evm");
    assert!(created.customer_id.is_some());

    let payin: Payin = full("GET /v1/instances/{instance_id}/payins/{id}");
    assert!(!payin.customer_id.is_empty());
    assert!(payin.created_at.is_some());
    assert!(payin.commercial_quotation.is_some());

    let created: CreatePayinResponse = full("POST /v1/instances/{instance_id}/payins/evm");
    assert!(created.customer_id.is_some());
    assert!(created.blindpay_bank_details.routing_number.is_some());

    let transfer: Transfer = full("GET /v1/instances/{instance_id}/transfers/{id}");
    assert!(!transfer.customer_id.is_empty());
    assert!(!transfer.customer_wallet_address.is_empty());
    assert!(transfer.tracking_partner_fee.is_some());
    assert!(transfer.address.is_some());

    let wallet: BlockchainWallet =
        full("GET /v1/instances/{instance_id}/customers/{customer_id}/blockchain-wallets/{id}");
    assert!(!wallet.customer_id.is_empty());
    assert!(wallet.name.is_some() && wallet.network.is_some());
    assert!(wallet.is_account_abstraction.is_some());

    let request: LimitIncrease =
        full("GET /v1/instances/{instance_id}/customers/{customer_id}/limit-increase");
    assert!(!request.customer_id.is_empty());

    let offramp: OfframpWallet = full(
        "GET /v1/instances/{instance_id}/customers/{customer_id}/bank-accounts/{bank_account_id}/offramp-wallets/{id}",
    );
    assert!(offramp.customer_id.is_some());

    let quote: Quote = full("POST /v1/instances/{instance_id}/quotes");
    assert!(quote.customer_local_amount.is_some());
    assert!(quote.expires_at.is_some());

    let va: VirtualAccount =
        full("GET /v1/instances/{instance_id}/customers/{customer_id}/virtual-accounts/{id}");
    assert!(va.us.is_some() && va.co.is_some() && va.mx.is_some());
    assert!(va.banking_partner.is_some() && va.kyc_status.is_some());
}

/// The `decimals` variant must keep FX rates fractional, not truncate them.
#[test]
fn payout_fx_rates_keep_their_fraction() {
    let ops = index_operations();
    let fixture = read_json(
        ops["GET /v1/instances/{instance_id}/payouts/{id}"]
            .as_str()
            .unwrap(),
    );
    let body = fixture["responses"]["200"]["variants"]["decimals"].clone();
    let payout: Payout = serde_json::from_value(body).unwrap();
    assert_eq!(payout.commercial_quotation, Some(1234.56));
    assert_eq!(payout.blindpay_quotation, Some(1234.56));
}
