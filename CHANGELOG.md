# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/blindpaylabs/blindpay-rust/compare/v0.1.0...v0.2.0) - 2026-11-12

### Added

- review_reason, typed error codes, trusted-publishing-ready release job ([#6](https://github.com/blindpaylabs/blindpay-rust/pull/6))
- [**breaking**] add BlindPay::with_timeout and validate the instance id
- [**breaking**] upload takes an UploadFile with a content type
- [**breaking**] accept latitude, longitude and additional_info on customer create
- catch up response fields and enum values with the spec
- [**breaking**] add the 18 missing surface operations (94/94)
- add list auto-pagers and report list item decode errors
- [**breaking**] replace svix with a native webhook verifier and typed events
- [**breaking**] add request options, idempotency keys, retries and structured API errors
- [**breaking**] sync WebhookEvent names with the spec
- implement the BlindPay Rust SDK

### Fixed

- [**breaking**] evaluate the retry rules in order; keep status and key on decode errors
- [**breaking**] root the base URL at the host and percent-encode the instance id
- resend only requests that provably got no response
- [**breaking**] align request shapes with the spec (audit §9.4)
- [**breaking**] keep instance_percentage_fee as integer basis points
- [**breaking**] decode non-US virtual accounts (co/mx blocks, optional us)
- [**breaking**] make nullable or optional response fields Option
- [**breaking**] type fractional response numbers as f64
- [**breaking**] rename receiver_* wire fields to customer_*
- model FX quotation fields as f64, not i64

### Other

- plain deprecation reasons, webhook Decode guidance, CLAUDE.md rules
- type closed-set fields as open enums; add CreateFasterPaymentsInput::new
- deprecate the response fields the spec no longer has (D13)
- make the retry-sleep cancellation test deterministic
- rewrite the README for the parity release
- add CLAUDE.md and stop ignoring agent files
- declare MSRV 1.85 and keep tests out of the published crate
- run the 14 shared webhook vectors and decode every webhook fixture
- add a spec contract check (Directions A/B/C)
- finish the receiver -> customer doc-comment rename
- re-copy regenerated spec fixtures and drop the defect patch
- decode every shared spec fixture variant
- move shared TransfersType/TrackingStatus/ProviderName into common
- drop unimplemented blocking feature
- release 0.2.0 (0.1.0 already taken on crates.io)
- align README with the other BlindPay SDKs
- Initial commit
