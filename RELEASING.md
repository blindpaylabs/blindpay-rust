# Releasing blindpay (Rust)

## Version and changelog

- `version` in `Cargo.toml` is the only source of truth. The User-Agent (`blindpay-rust/<version>`)
  comes from it through `CARGO_PKG_VERSION`.
- [release-plz](https://release-plz.dev) owns the version bump and `CHANGELOG.md`, computed from
  Conventional Commits. Don't edit either by hand. On 0.x, a breaking change (`feat!:`, `fix!:`)
  bumps the minor; `feat:` and `fix:` bump the patch.

## What a merge to `main` triggers

`.github/workflows/release-plz.yml` runs two jobs on every push to `main`:

- `release-pr` opens or updates the release PR (`chore: release vX.Y.Z`) with the bump and the
  changelog. GitHub holds CI on bot PRs, so a maintainer clicks "Approve and run" on it.
- `release` publishes any `Cargo.toml` version crates.io doesn't have yet, then tags `vX.Y.Z` and
  creates the GitHub release. Merging the release PR is therefore how you release, and **the first
  push to `main` after publishing credentials exist publishes whatever `main` holds**.

## Credentials (who owns what)

The `blindpay` crate on crates.io is owned by BernardoSM (its 0.1.0 is BlindPay's own
"Hello, world!" name placeholder). Repository secrets need an admin (BernardoSM or ericviana).
The `release` job supports two paths and picks one per run:

1. **Trusted publishing (preferred, no secret).** A crate owner opens crates.io → `blindpay` →
   Settings → Trusted Publishing and adds repository `blindpaylabs/blindpay-rust`, workflow
   `release-plz.yml`, no environment. The job has `id-token: write`; release-plz exchanges the OIDC
   token for a short-lived crates.io token. This only happens when `CARGO_REGISTRY_TOKEN` is not set
   at all, so the job leaves it undefined when the secret is missing. After adding the publisher, an
   admin sets the repository variable `CRATES_IO_TRUSTED_PUBLISHING` to `true`
   (`gh variable set CRATES_IO_TRUSTED_PUBLISHING --body true -R blindpaylabs/blindpay-rust`).
2. **Token.** A crate owner creates a crates.io API token scoped to `publish-update` for `blindpay`;
   an admin stores it as the `CARGO_REGISTRY_TOKEN` repository secret. When it exists, the job passes it.

Until one of them is in place, the `release` job skips publishing with a warning annotation
("crates.io publishing not configured") and stays green, so `main` isn't red on every push. Adding co-owners
(`cargo owner --add github:blindpaylabs:<team>`) removes the single-owner dependency.

## Getting 0.2.0 out (order matters)

1. Merge the feature PRs that belong in 0.2.0.
2. `release-pr` refreshes the release PR. Approve its CI, review `CHANGELOG.md`, merge it.
3. A crate owner sets up trusted publishing or the token (above).
4. Dispatch `release-plz.yml` on `main`, or push any commit (next section).

## Until 0.2.0 is on crates.io

`cargo add blindpay` resolves the 0.1.0 placeholder until 0.2.0 is published, so early users depend
on a pinned commit of this repository (there are no tags before the first publish). Use the latest
`main` commit, for example the 0.2.0 release commit:

```toml
blindpay = { git = "https://github.com/blindpaylabs/blindpay-rust", rev = "de57cdd9c4b27b28a7b821eb95061b533e605904" }
```

This note lives here, not in the README: the README is packaged into the crate and is the crates.io
and docs.rs page of the published version, so it already describes the published state.

## Re-running a stuck release

Dispatch the workflow on `main` (both jobs run; dispatching another ref runs nothing):

```sh
gh workflow run release-plz.yml --ref main
gh run list --workflow release-plz.yml --branch main --limit 1   # then: gh run watch <run-id>
```

## Verifying the published crate

```sh
curl -s -A blindpay-release-check https://crates.io/api/v1/crates/blindpay | jq '.crate.max_version'
cargo info blindpay@0.2.0
```

Then check that:

- <https://docs.rs/blindpay> built (Linux, all features; `[package.metadata.docs.rs]` in
  `Cargo.toml`) and its front page is the README;
- the crates.io page shows the README with `cargo add blindpay` as the install step;
- `cargo add blindpay` in a scratch crate resolves 0.2.0 and the README quick start builds;
- the `v0.2.0` tag and GitHub release exist.

Then tell git-dependency users (the section above) to switch to `blindpay = "0.2"`, and consider
yanking the 0.1.0 placeholder (`cargo yank --version 0.1.0 blindpay`).

## Known traps

- Publishing credentials before the code and the release PR land publishes an incomplete 0.2.0.
- A published version's README and docs can't be changed. The README must never carry pre-publish
  wording ("not on crates.io yet"); interim instructions go in this file.
- `cargo publish --dry-run` (CI) only warns when the version already exists on crates.io, so it
  stays green after a release.
- `cargo-semver-checks` runs in `release-pr` and can force a minor bump on an API break.
