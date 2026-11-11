//! Guard: every mutating resource method forwards request options.
//!
//! A method whose doc comment carries a `POST|PUT|PATCH|DELETE /path` line
//! must have a `<name>_with_options` sibling that takes a trailing
//! `options: &RequestOptions` and passes it on, and the plain method must
//! delegate to it with `RequestOptions::default()`. Without this, a new method
//! could silently drop idempotency keys.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quote::ToTokens;

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn doc_text(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_mutating(doc: &str) -> bool {
    ["POST /", "PUT /", "PATCH /", "DELETE /"]
        .iter()
        .any(|verb| doc.contains(&format!("`{verb}")))
}

/// Every identifier in the method body.
fn body_idents(f: &syn::ImplItemFn) -> Vec<String> {
    fn walk(tokens: proc_macro2::TokenStream, out: &mut Vec<String>) {
        for tt in tokens {
            match tt {
                proc_macro2::TokenTree::Ident(id) => out.push(id.to_string()),
                proc_macro2::TokenTree::Group(g) => walk(g.stream(), out),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(f.block.to_token_stream(), &mut out);
    out
}

#[test]
fn every_mutating_method_forwards_request_options() {
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/resources"),
        &mut files,
    );
    let mut checked = 0;
    let mut failures = Vec::new();

    for path in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let file = syn::parse_file(&src).unwrap();
        for item in &file.items {
            let syn::Item::Impl(imp) = item else { continue };
            let methods: BTreeMap<String, &syn::ImplItemFn> = imp
                .items
                .iter()
                .filter_map(|i| match i {
                    syn::ImplItem::Fn(f) if matches!(f.vis, syn::Visibility::Public(_)) => {
                        Some((f.sig.ident.to_string(), f))
                    }
                    _ => None,
                })
                .collect();
            for (name, f) in &methods {
                if name.ends_with("_with_options") || !is_mutating(&doc_text(&f.attrs)) {
                    continue;
                }
                checked += 1;
                let where_ = format!("{}::{name}", path.display());
                let variant_name = format!("{name}_with_options");
                let Some(variant) = methods.get(&variant_name) else {
                    failures.push(format!("{where_}: no `{variant_name}`"));
                    continue;
                };
                let last = variant.sig.inputs.last().and_then(|arg| match arg {
                    syn::FnArg::Typed(t) => match &*t.pat {
                        syn::Pat::Ident(id) => Some(id.ident.to_string()),
                        _ => None,
                    },
                    _ => None,
                });
                if last.as_deref() != Some("options") {
                    failures.push(format!(
                        "{where_}: `{variant_name}` lacks a trailing `options`"
                    ));
                }
                let variant_body = body_idents(variant);
                if !variant_body.iter().any(|i| i == "options") {
                    failures.push(format!(
                        "{where_}: `{variant_name}` never passes `options` on"
                    ));
                }
                let plain = body_idents(f);
                if !plain.contains(&variant_name) || !plain.iter().any(|i| i == "RequestOptions") {
                    failures.push(format!(
                        "{where_}: must delegate to `{variant_name}(.., &RequestOptions::default())`"
                    ));
                }
            }
        }
    }

    assert!(
        checked >= 50,
        "expected every mutating method, found {checked}"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
