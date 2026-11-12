//! Generator and check for `src/error_code.rs` (the typed `ErrorCode` set).
//!
//! The source is the error table in the `Errors` tag of the committed spec
//! snapshot (`| `CODE` | status | retry | description |`). This test renders
//! `src/error_code.rs` from it and fails when the committed file differs, so a
//! new code in the spec fails CI. To regenerate after refreshing the snapshot:
//!
//! ```sh
//! BLINDPAY_UPDATE_ERROR_CODES=1 cargo test --test error_codes
//! ```

use std::collections::BTreeSet;
use std::path::PathBuf;

use blindpay::{ErrorCode, ErrorCodeInfo, RetryClass};
use serde_json::Value;

const GENERATED: &str = "src/error_code.rs";
const UPDATE_ENV: &str = "BLINDPAY_UPDATE_ERROR_CODES";

struct Row {
    code: String,
    status: u16,
    retry: RetryClass,
    description: String,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Parses one table row: `` | `CODE` | 400 | actionable | Description. | ``.
fn parse_row(line: &str) -> Result<Row, String> {
    let inner = line
        .strip_prefix("| ")
        .and_then(|l| l.strip_suffix(" |"))
        .ok_or("row doesn't start with `| ` and end with ` |`")?;
    let cells: Vec<&str> = inner.splitn(4, " | ").map(str::trim).collect();
    let [code, status, retry, description] = cells[..] else {
        return Err(format!("expected 4 cells, got {}", cells.len()));
    };
    let code = code
        .strip_prefix('`')
        .and_then(|c| c.strip_suffix('`'))
        .filter(|c| {
            !c.is_empty()
                && c.chars()
                    .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        })
        .ok_or_else(|| format!("bad code cell {code:?}"))?;
    let status: u16 = status
        .parse()
        .ok()
        .filter(|s| (100..600).contains(s))
        .ok_or_else(|| format!("bad status cell {status:?}"))?;
    let retry = match retry {
        "retryable" => RetryClass::Retryable,
        "terminal" => RetryClass::Terminal,
        "actionable" => RetryClass::Actionable,
        other => return Err(format!("bad retry cell {other:?}")),
    };
    if description.is_empty() {
        return Err("empty description".into());
    }
    Ok(Row {
        code: code.to_string(),
        status,
        retry,
        description: description.to_string(),
    })
}

/// Parses the `Errors` tag's table. Every line that starts with `` | ` `` must
/// parse, and there must be at least one.
fn spec_rows() -> Vec<Row> {
    let path = manifest_dir().join(".api-sync/spec-snapshot.json");
    let spec: Value = serde_json::from_slice(&std::fs::read(path).expect("read spec snapshot"))
        .expect("parse spec snapshot");
    let description = spec["tags"]
        .as_array()
        .expect("spec has tags")
        .iter()
        .find(|t| t["name"] == "Errors")
        .and_then(|t| t["description"].as_str())
        .expect("spec has an `Errors` tag with a description");
    let candidates: Vec<&str> = description
        .lines()
        .filter(|l| l.starts_with("| `"))
        .collect();
    assert!(
        !candidates.is_empty(),
        "no rows in the spec's error-code table"
    );
    let mut rows = Vec::new();
    let mut bad = Vec::new();
    for line in &candidates {
        match parse_row(line) {
            Ok(row) => rows.push(row),
            Err(why) => bad.push(format!("{why}: {line}")),
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {} error-table rows don't parse:\n{}",
        bad.len(),
        candidates.len(),
        bad.join("\n")
    );
    assert_eq!(rows.len(), candidates.len());
    rows
}

/// `TRANSFERS_SWAP_AMOUNT_BELOW_MINIMUM` -> `TransfersSwapAmountBelowMinimum`.
fn variant(code: &str) -> String {
    code.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let lower = p.to_ascii_lowercase();
            let mut chars = lower.chars();
            let first = chars.next().unwrap().to_ascii_uppercase();
            std::iter::once(first).chain(chars).collect::<String>()
        })
        .collect()
}

fn retry_variant(retry: RetryClass) -> &'static str {
    match retry {
        RetryClass::Retryable => "Retryable",
        RetryClass::Terminal => "Terminal",
        RetryClass::Actionable => "Actionable",
        other => panic!("no table value for {other}"),
    }
}

/// Makes spec text safe for rustdoc: escapes link brackets and HTML, and wraps
/// bare URLs so `cargo doc -D warnings` passes.
fn doc_text(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            if word.starts_with("http://") || word.starts_with("https://") {
                let url = word.trim_end_matches(['.', ',', ';', ':', ')']);
                format!("<{url}>{}", &word[url.len()..])
            } else {
                word.replace('[', "\\[")
                    .replace(']', "\\]")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render(rows: &[Row]) -> String {
    let mut out = String::new();
    out.push_str(
        "//! Error codes from the spec's `Errors` table. Generated by `tests/error_codes.rs`\n\
         //! from `.api-sync/spec-snapshot.json`; don't edit by hand. Regenerate with\n\
         //! `BLINDPAY_UPDATE_ERROR_CODES=1 cargo test --test error_codes`.\n\
         \n\
         use crate::common::open_enum;\n\
         use crate::error::{ErrorCodeInfo, RetryClass};\n\
         \n\
         open_enum! {\n\
         \x20   /// An error code the API documents (the `code` of an error body).\n\
         \x20   ///\n\
         \x20   /// Get one from [`ApiError::error_code`](crate::ApiError::error_code). Codes\n\
         \x20   /// this SDK version doesn't know decode to [`ErrorCode::Unknown`] with the\n\
         \x20   /// raw string, so match with a wildcard arm. [`ErrorCode::info`] gives the\n\
         \x20   /// documented status and [`RetryClass`].\n\
         \x20   pub enum ErrorCode {\n",
    );
    for row in rows {
        out.push_str(&format!(
            "        /// `{}` ({}, {}): {}\n        {} => \"{}\",\n",
            row.code,
            row.status,
            row.retry.as_str(),
            doc_text(&row.description),
            variant(&row.code),
            row.code
        ));
    }
    out.push_str(
        "    }\n\
         }\n\
         \n\
         impl ErrorCode {\n\
         \x20   /// The status and retry class the API documents for this code; `None`\n\
         \x20   /// for [`ErrorCode::Unknown`].\n\
         \x20   pub fn info(&self) -> Option<ErrorCodeInfo> {\n\
         \x20       use RetryClass::*;\n\
         \x20       let (status, retry) = match self {\n",
    );
    for row in rows {
        out.push_str(&format!(
            "            Self::{} => ({}, {}),\n",
            variant(&row.code),
            row.status,
            retry_variant(row.retry)
        ));
    }
    out.push_str(
        "            Self::Unknown(_) => return None,\n\
         \x20       };\n\
         \x20       Some(ErrorCodeInfo { status, retry })\n\
         \x20   }\n\
         }\n",
    );
    out
}

#[test]
fn generated_error_codes_match_the_spec_snapshot() {
    let rows = spec_rows();
    let codes: BTreeSet<&str> = rows.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(codes.len(), rows.len(), "duplicate code in the spec table");
    let variants: BTreeSet<String> = rows.iter().map(|r| variant(&r.code)).collect();
    assert_eq!(variants.len(), rows.len(), "two codes map to one variant");

    let expected = render(&rows);
    let path = manifest_dir().join(GENERATED);
    if std::env::var_os(UPDATE_ENV).is_some() {
        std::fs::write(&path, &expected).expect("write src/error_code.rs");
        println!("wrote {GENERATED} ({} codes)", rows.len());
        return;
    }
    let actual = std::fs::read_to_string(&path).unwrap_or_default();
    if actual != expected {
        let line = actual
            .lines()
            .zip(expected.lines())
            .position(|(a, e)| a != e)
            .unwrap_or_else(|| actual.lines().count().min(expected.lines().count()));
        panic!(
            "{GENERATED} is out of date with the spec's error table (first difference at \
             line {}). Run `{UPDATE_ENV}=1 cargo test --test error_codes`.",
            line + 1
        );
    }
}

#[test]
fn every_spec_code_round_trips_with_its_info() {
    if std::env::var_os(UPDATE_ENV).is_some() {
        // This binary was compiled from the file being regenerated; the next
        // plain run checks the new one.
        return;
    }
    for row in spec_rows() {
        let code = ErrorCode::from(row.code.as_str());
        assert!(
            !matches!(code, ErrorCode::Unknown(_)),
            "{} is Unknown",
            row.code
        );
        assert_eq!(code.as_str(), row.code);
        let info = code.info().expect("known code has info");
        assert_eq!(info.status, row.status, "{}", row.code);
        assert_eq!(info.retry, row.retry, "{}", row.code);
    }
}

#[test]
fn swap_minimum_code_and_unknown_codes() {
    let code = ErrorCode::from("TRANSFERS_SWAP_AMOUNT_BELOW_MINIMUM");
    assert_eq!(code, ErrorCode::TransfersSwapAmountBelowMinimum);
    let info: ErrorCodeInfo = code.info().unwrap();
    assert_eq!(info.status, 400);
    assert_eq!(info.retry, RetryClass::Actionable);

    let unknown = ErrorCode::from("SOMETHING_NEW_LATER");
    assert_eq!(unknown, ErrorCode::Unknown("SOMETHING_NEW_LATER".into()));
    assert_eq!(unknown.as_str(), "SOMETHING_NEW_LATER");
    assert_eq!(unknown.info(), None);
}
