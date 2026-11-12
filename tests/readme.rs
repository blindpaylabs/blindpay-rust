//! The README is the crate docs, so its snippets compile as doctests (`cargo
//! test --doc`). Doctests see this crate's own dependencies, though, which a
//! user's project doesn't have. This checks that every crate a README snippet
//! names (other than `blindpay` and `std`) is listed in one of the README's
//! `toml` blocks, so copying a snippet never fails with "undeclared crate".

use std::collections::BTreeSet;

/// The bodies of the README's fenced blocks whose info string starts with `lang`.
fn blocks(readme: &str, lang: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Option<String> = None;
    for line in readme.lines() {
        match &mut current {
            None => {
                if let Some(info) = line.strip_prefix("```") {
                    if info.split(',').next() == Some(lang) {
                        current = Some(String::new());
                    }
                }
            }
            Some(body) => {
                if line.starts_with("```") {
                    out.push(current.take().unwrap());
                } else {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    out
}

/// Crate names (snake_case) used as a path root (`name::`) in Rust code,
/// including doctest-hidden `# ` lines and `use name::` imports.
fn crates_used(code: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in code.lines() {
        let line = line.trim_start().trim_start_matches("# ");
        if line.starts_with("//") {
            continue;
        }
        let bytes = line.as_bytes();
        for (i, _) in line.match_indices("::") {
            let start = line[..i]
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map_or(0, |p| p + 1);
            let name = &line[start..i];
            let preceded_by_path = start >= 2 && &bytes[start - 2..start] == b"::";
            let first_lower = name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
            if !name.is_empty() && first_lower && !preceded_by_path {
                out.insert(name.to_string());
            }
        }
    }
    out
}

#[test]
fn readme_snippets_only_use_crates_the_readme_lists() {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("read README.md");
    let listed: BTreeSet<String> = blocks(&readme, "toml")
        .iter()
        .flat_map(|b| b.lines().map(str::to_owned).collect::<Vec<_>>())
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().replace('-', "_")))
        .collect();
    // Path roots that aren't other crates: std, this crate, `self`/`super`/
    // `crate`, and primitive types (`str::to_owned`, `i64::MAX`).
    let builtin = [
        "std", "core", "alloc", "blindpay", "self", "super", "crate", "str", "bool", "char", "u8",
        "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
        "f64",
    ];
    let rust_blocks = blocks(&readme, "rust");
    assert!(
        rust_blocks.len() > 10,
        "README rust blocks: {}",
        rust_blocks.len()
    );

    let mut missing = BTreeSet::new();
    for block in &rust_blocks {
        for name in crates_used(block) {
            if !builtin.contains(&name.as_str()) && !listed.contains(&name) {
                missing.insert(name);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "README snippets use crates its toml blocks don't list: {missing:?}"
    );
}
