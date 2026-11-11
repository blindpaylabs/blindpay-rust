# CLAUDE.md: blindpay-rust SDK

Agent and contributor reference. Read it before changing code. The public
OpenAPI spec (`.api-sync/spec-snapshot.json`) decides every wire shape; the
API source in `blindpay-v2` wins when the spec and the code disagree.

## Commands

Rust isn't required on the host; every command also runs in Docker
(`rust:1`, with `CARGO_TARGET_DIR` outside the mount).

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features                     # unit, integration, contract, doc tests
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo check --no-default-features --features native-tls
cargo +1.85 check --all-features              # MSRV (rust-version in Cargo.toml)
cargo test --test contract -- --nocapture     # contract check + its warnings
cargo package --list                          # must list no tests/ or fixtures
```

```sh
docker run --rm -v "$PWD":/src -w /src -v blindpay-cargo-registry:/usr/local/cargo/registry \
  -e CARGO_TARGET_DIR=/tmp/target rust:1 sh -c \
  'rustup component add clippy rustfmt >/dev/null 2>&1; cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features'
```

## Project structure

```
src/
  lib.rs            crate docs (= README.md, so README snippets are doctests) and root re-exports
  client.rs         BlindPay + BlindPayBuilder (base_url, timeout, max_retries, http_client, retry_sleep)
  http.rs           the only transport: headers, idempotency keys, retry loop (Appendix A2), error parsing
  options.rs        RequestOptions (idempotency_key, timeout, headers)
  error.rs          Error { Config, Api(Box<ApiError>), Decode, Http }, ApiError
  internal.rs       encode_path_segment
  common/           open_enum! macro, shared scalars/enums, pagination (ListResponse, Pagination,
                    *Params), pager.rs (Pager<T>, the auto-pager)
  webhooks/         native Svix verifier (verify, verify_at, construct_webhook_event[_at]),
                    events.rs (WebhookEventPayload, keyed on `webhook_event`, Unknown fallback)
  resources/<name>/ mod.rs (handle + methods) and types.rs (wire types); sub-resources are
                    sibling files (instances/{tos,onboarding,webhook_endpoints}.rs, ...)
  resources/rfi/    RFI types and the InstanceRfis / CustomerRfis handles
tests/
  <resource>.rs     wiremock tests per resource
  transport.rs      retry table, idempotency keys, error fixtures (tests/fixtures/errors)
  pagination.rs     auto-pager, Send + Sync, cancellation
  request_shapes.rs request bodies/queries for spec-driven inputs
  new_ops.rs        every fixture variant served through the real method (A8)
  webhooks.rs       the 14 shared Svix vectors + every webhook fixture
  fixtures_decode.rs every response variant of all 94 ops decodes with its SDK type
  contract.rs       serde types vs the spec (Directions A/B/C, below)
  request_options_guard.rs  every mutating method forwards RequestOptions
  fixtures/         verbatim copy of sdk-parity/reference/fixtures (+ webhook-vectors.json); never hand-edit
.api-sync/
  spec-snapshot.json       the public spec this SDK targets
  contract-map.json        Rust type -> spec schema (or op request/response)
  contract-allowlist.json  `direction_a` (Rust fields not in the spec) and `unmodelled`
                           (spec fields left out on purpose), each with a reason
```

## Conventions

### Types
- **Enums are open.** Every categorical field uses `open_enum!` (known variants + `Unknown(String)`,
  `#[non_exhaustive]`). An unknown value must never fail a decode. Free-form codes with huge
  domains (`swift_payment_code`, NAICS `business_industry`) stay `String`/newtypes.
- **Response structs**: `#[derive(Debug, Clone, PartialEq, Deserialize)] #[non_exhaustive]`.
  Anything the spec marks optional *or* nullable is `#[serde(default)] pub x: Option<T>`. A bare
  `T` only for required, non-nullable fields (Direction C enforces this). Never
  `deny_unknown_fields`.
- **Request structs**: `#[derive(Serialize)]`, `pub` fields, a `new(required…)` constructor and
  chainable setters. Optional fields are `Option<T>` with
  `#[serde(skip_serializing_if = "Option::is_none")]`: absent, never `null`. Tri-state fields
  (`null` clears, e.g. offramp `partner_fee_id`) are `Option<Option<T>>`.
- **Units**: payout/payin/quote/limit amounts are integer cents (`i64`); partner percentage fees are
  basis points; wallet balances are whole tokens (`f64`); quotations and other genuinely decimal
  values are `f64`. Say the unit in the doc comment.
- **Wire names** follow the spec exactly; camelCase keys (`functionName`, `chainId`, …) use
  `#[serde(rename)]`. No new typed fields for provider-named spec properties (`aiprise_*`,
  `checkbook_*`, …): list them under `unmodelled` in the allowlist instead.
- "Exactly one of" inputs (payout `sender_wallet_address` / `wallet_id`, quote `bank_account_id` /
  `payable_id`) are two `Option`s plus a constructor per case; never validate client-side.

### Methods
- `snake_case` verbs mirroring the Node SDK names (`create_evm`, `get_track`, `list_auto_paging`).
- Path ids are `impl AsRef<str>` and always go through `encode_path_segment`.
- **Every method's doc comment has a backticked `METHOD /path` line** (e.g.
  ``/// `POST /instances/{instance_id}/payouts/evm` ``). The options guard test and the coverage
  script read it.
- **Every mutating method** (POST/PUT/PATCH/DELETE) has a `<name>_with_options(.., options:
  &RequestOptions)` sibling that calls `request_with`/`post_multipart_with`, and the plain method
  delegates with `&RequestOptions::default()`. `tests/request_options_guard.rs` fails otherwise.
- Lists return `ListResponse<T>` (envelope or bare array). Cursor-paginated lists (payouts, payins,
  customers, transfers, payables, wallet deposits) also get `list_auto_paging` returning `Pager<T>`.
  Don't add `offset` to new params (the API ignores it).
- Never hand-roll HTTP: everything goes through `Inner::{get, get_query, request_with,
  post_multipart_with}` so retries, keys and error parsing apply.

### Transport rules (don't regress)
- Idempotency keys: caller key validated (1–255 chars) before sending; otherwise a UUIDv4 is minted
  before the first attempt of every JSON mutation when retries are on. Multipart never gets one.
- Retries follow the plan's Appendix A2 table: GETs on connection errors/timeouts/408/429/5xx;
  mutations only on no-response, 429 and 409 `idempotency_key_in_flight`, **never on 5xx**.
  `Retry-After` ≤ 60 s is honoured, otherwise jittered backoff. Tests use `retry_sleep`, never real
  sleeps.
- `BlindPay` is `Clone + Send + Sync` and immutable; per-request options never touch shared state.
- Never put the API key, webhook secret or a request body into an error message or `Debug` output.

## Generate before you hand-write

There is no Rust api-sync generator yet (a follow-up). Until then:
1. Read the schema in `.api-sync/spec-snapshot.json` (use `jq`/`python3`; it's 2.5 MB).
2. Hand-write the type following the conventions above, and map it in
   `.api-sync/contract-map.json` (`schema`, or `op` + `part`).
3. Run `cargo test --test contract -- --nocapture`: zero violations, zero warnings (or an
   `unmodelled` entry with a reason).

When the spec changes: replace `.api-sync/spec-snapshot.json` and `tests/fixtures/` with the new
reference copies, then fix what the contract and fixture tests report.

## How to: add a method

1. Types in `src/resources/<name>/types.rs` (request + response), re-exported from the resource
   `mod.rs` and from `src/lib.rs` (every public type is importable from the crate root).
2. The method in `mod.rs`, with the `METHOD /path` doc line, `encode_path_segment` on ids, and a
   `_with_options` sibling if it mutates.
3. Map the types in `.api-sync/contract-map.json`; add the op's decoder to
   `tests/fixtures_decode.rs`.
4. A wiremock test asserting method, exact path, query and body, and decoding the spec fixture.

## How to: add a resource

Create `src/resources/<name>/{mod.rs,types.rs}`, declare it in `src/resources/mod.rs`, add a field
to `BlindPay` and construct it in `BlindPayBuilder::build` (`src/client.rs`), re-export from
`src/lib.rs`, then follow "add a method".

## How to: modify types

- Add a response field: `#[serde(default)] Option<T>` unless the spec requires it and forbids null.
- Add an enum value: a variant in the `open_enum!` block (non-breaking; enums are
  `#[non_exhaustive]`).
- Rename/retype/remove a field or make a request field required: breaking. Use `feat!:`/`fix!:` and
  add the change to the README's "Upgrading" section.

## Releases

release-plz computes the version from **Conventional Commits** (`feat:`, `fix:`, `docs:`, `test:`,
`build:`, `ci:`; `!` for breaking changes; the crate is 0.x, so breaking changes bump the minor).
Don't edit `CHANGELOG.md` or the version by hand, and never publish locally. The User-Agent
(`blindpay-rust/<version>`) comes from `CARGO_PKG_VERSION`.

Rule of thumb for floors: MSRV is 1.85 (edition 2024). Don't raise it, and don't add runtime
dependencies without a reason in the PR (current: reqwest, serde, serde_json, thiserror, http,
tokio `time` for retry sleeps, uuid `v4`, hmac + sha2 + base64 for webhooks).
