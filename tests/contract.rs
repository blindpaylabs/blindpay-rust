//! Contract check: the SDK's serde types against the committed spec snapshot.
//!
//! `.api-sync/spec-snapshot.json` is a copy of the public OpenAPI spec the
//! SDK targets. This test parses every `#[derive(Serialize|Deserialize)]`
//! struct and every `open_enum!` in `src/` with `syn`, then checks:
//!
//! - **Direction A (hard):** every wire field of a struct mapped in
//!   `.api-sync/contract-map.json` is a property of its spec schema. Every
//!   other serde struct's wire fields must be a property name somewhere in the
//!   spec. Exceptions live in `.api-sync/contract-allowlist.json`, each with a
//!   reason.
//! - **Direction B (hard):** `WebhookEvent` covers every event name the spec
//!   accepts on `WebhookEndpointIn.events`.
//! - **Direction C (hard):** a mapped *response* struct's non-`Option` field
//!   (without `#[serde(default)]`) must be required and non-nullable in the
//!   spec, or a real response can fail to decode. This is the check that would
//!   have caught `receiver_id`.
//!
//! - **Unmodelled response properties (hard):** every property of a mapped
//!   *response* schema (including the `*WebhookOut` webhook payloads) is a
//!   field of its Rust type, so a new response field in the spec fails CI.
//!   Properties deliberately left out (provider-named fields) are listed with a
//!   reason under `unmodelled` in the allowlist. `webhook_event` on a
//!   `*WebhookOut` schema is exempt: `WebhookEventPayload` consumes it.
//!
//! Request-side required-ness and unmodelled request properties are reported
//! as warnings (run with `--nocapture` to see them).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

// ---------------------------------------------------------------------------
// Rust side
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct RustField {
    wire: String,
    optional: bool,
    has_default: bool,
}

#[derive(Debug)]
struct RustStruct {
    name: String,
    file: String,
    fields: Vec<RustField>,
}

#[derive(Default)]
struct RustTypes {
    structs: Vec<RustStruct>,
    enums: BTreeMap<String, Vec<String>>,
}

impl RustTypes {
    fn find(&self, name: &str) -> Vec<&RustStruct> {
        self.structs.iter().filter(|s| s.name == name).collect()
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.meta
                .require_list()
                .is_ok_and(|l| l.tokens.to_string().contains("test"))
    })
}

fn derives(attrs: &[syn::Attribute], what: &str) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("derive")
            && a.meta.require_list().is_ok_and(|l| {
                l.tokens
                    .to_string()
                    .split(',')
                    .any(|t| t.rsplit("::").next().map(str::trim) == Some(what))
            })
    })
}

/// Reads `#[serde(rename = "..", default, flatten, skip..)]` on a field.
struct SerdeAttrs {
    rename: Option<String>,
    default: bool,
    flatten: bool,
    skip: bool,
}

fn serde_attrs(attrs: &[syn::Attribute]) -> SerdeAttrs {
    let mut out = SerdeAttrs {
        rename: None,
        default: false,
        flatten: false,
        skip: false,
    };
    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                out.rename = Some(lit.value());
            } else if meta.path.is_ident("default") {
                out.default = true;
                if meta.input.peek(syn::Token![=]) {
                    let _: syn::LitStr = meta.value()?.parse()?;
                }
            } else if meta.path.is_ident("flatten") {
                out.flatten = true;
            } else if meta.path.is_ident("skip") || meta.path.is_ident("skip_deserializing") {
                out.skip = true;
            } else if meta.input.peek(syn::Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }
            Ok(())
        });
    }
    out
}

/// Reads a struct-level `#[serde(rename_all = "..")]`.
fn rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    let mut out = None;
    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let _ = attr.parse_nested_meta(|meta| {
            if meta.input.peek(syn::Token![=]) {
                let lit: syn::LitStr = meta.value()?.parse()?;
                if meta.path.is_ident("rename_all") {
                    out = Some(lit.value());
                }
            }
            Ok(())
        });
    }
    out
}

/// Applies a serde `rename_all` rule to a snake_case field name.
fn apply_rename_all(rule: Option<&str>, ident: &str) -> String {
    match rule {
        Some("camelCase") => {
            let mut out = String::new();
            let mut upper = false;
            for c in ident.chars() {
                if c == '_' {
                    upper = true;
                } else if upper {
                    out.extend(c.to_uppercase());
                    upper = false;
                } else {
                    out.push(c);
                }
            }
            out
        }
        Some("snake_case") | None => ident.to_string(),
        Some("lowercase") => ident.to_lowercase(),
        Some("UPPERCASE") | Some("SCREAMING_SNAKE_CASE") => ident.to_uppercase(),
        Some(other) => panic!("contract check: unsupported rename_all = {other:?}"),
    }
}

fn is_option(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "Option"))
}

fn collect_items(items: &[syn::Item], file: &str, out: &mut RustTypes) {
    for item in items {
        match item {
            syn::Item::Struct(s) if !is_cfg_test(&s.attrs) => {
                let serde_type = derives(&s.attrs, "Deserialize") || derives(&s.attrs, "Serialize");
                let syn::Fields::Named(named) = &s.fields else {
                    continue;
                };
                if !serde_type {
                    continue;
                }
                let rule = rename_all(&s.attrs);
                let fields = named
                    .named
                    .iter()
                    .filter_map(|f| {
                        let attrs = serde_attrs(&f.attrs);
                        if attrs.flatten || attrs.skip {
                            return None;
                        }
                        let ident = f.ident.as_ref()?.to_string();
                        let ident = ident.trim_start_matches("r#").to_string();
                        Some(RustField {
                            wire: attrs
                                .rename
                                .unwrap_or_else(|| apply_rename_all(rule.as_deref(), &ident)),
                            optional: is_option(&f.ty),
                            has_default: attrs.default,
                        })
                    })
                    .collect();
                out.structs.push(RustStruct {
                    name: s.ident.to_string(),
                    file: file.to_string(),
                    fields,
                });
            }
            syn::Item::Macro(m) if m.mac.path.is_ident("open_enum") => {
                let tokens = m.mac.tokens.to_string();
                let name = tokens
                    .split_whitespace()
                    .skip_while(|t| *t != "enum")
                    .nth(1)
                    .unwrap_or_default()
                    .to_string();
                let values = tokens
                    .split("=>")
                    .skip(1)
                    .filter_map(|chunk| {
                        let chunk = chunk.trim_start();
                        let rest = chunk.strip_prefix('"')?;
                        Some(rest[..rest.find('"')?].to_string())
                    })
                    .collect();
                out.enums.insert(name, values);
            }
            syn::Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, items)) = &m.content {
                    collect_items(items, file, out);
                }
            }
            _ => {}
        }
    }
}

fn parse_rust_types() -> RustTypes {
    let mut files = Vec::new();
    rust_files(&manifest_dir().join("src"), &mut files);
    files.sort();
    let mut out = RustTypes::default();
    for path in files {
        let src = std::fs::read_to_string(&path).expect("read source");
        let file =
            syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        let rel = path
            .strip_prefix(manifest_dir())
            .unwrap()
            .display()
            .to_string();
        collect_items(&file.items, &rel, &mut out);
    }
    out
}

// ---------------------------------------------------------------------------
// Spec side
// ---------------------------------------------------------------------------

struct Spec {
    root: Value,
}

#[derive(Debug, Clone, Copy)]
struct SpecProp {
    required: bool,
    nullable: bool,
}

impl Spec {
    fn load() -> Self {
        let path = manifest_dir().join(".api-sync/spec-snapshot.json");
        let bytes = std::fs::read(&path).expect("read .api-sync/spec-snapshot.json");
        Spec {
            root: serde_json::from_slice(&bytes).expect("parse spec snapshot"),
        }
    }

    fn deref<'a>(&'a self, mut schema: &'a Value) -> &'a Value {
        for _ in 0..20 {
            let Some(r) = schema.get("$ref").and_then(Value::as_str) else {
                break;
            };
            let name = r.rsplit('/').next().unwrap();
            schema = &self.root["components"]["schemas"][name];
        }
        schema
    }

    fn nullable(&self, schema: &Value) -> bool {
        let schema = self.deref(schema);
        let type_null = match schema.get("type") {
            Some(Value::Array(ts)) => ts.iter().any(|t| t == "null"),
            Some(Value::String(t)) => t == "null",
            _ => false,
        };
        let union_null = ["anyOf", "oneOf"].iter().any(|k| {
            schema
                .get(*k)
                .and_then(Value::as_array)
                .is_some_and(|xs| xs.iter().any(|x| self.nullable(x)))
        });
        type_null
            || union_null
            || schema.get("nullable") == Some(&Value::Bool(true))
            || schema
                .get("enum")
                .and_then(Value::as_array)
                .is_some_and(|e| e.contains(&Value::Null))
    }

    /// Properties of a schema, merged across `allOf`/`anyOf`/`oneOf`. A property
    /// counts as required only if every branch that declares it requires it.
    fn props(&self, schema: &Value) -> BTreeMap<String, SpecProp> {
        let schema = self.deref(schema);
        let mut out: BTreeMap<String, SpecProp> = BTreeMap::new();
        if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
            for branch in all {
                out.extend(self.props(branch));
            }
            return out;
        }
        if let Some(any) = schema
            .get("anyOf")
            .or_else(|| schema.get("oneOf"))
            .and_then(Value::as_array)
        {
            let branches: Vec<_> = any
                .iter()
                .filter(|b| self.deref(b).get("type") != Some(&Value::from("null")))
                .map(|b| self.props(b))
                .collect();
            for branch in &branches {
                for (k, p) in branch {
                    let in_all = branches
                        .iter()
                        .all(|b| b.get(k).is_some_and(|q| q.required));
                    out.entry(k.clone())
                        .and_modify(|e| e.nullable |= p.nullable)
                        .or_insert(SpecProp {
                            required: in_all,
                            nullable: p.nullable,
                        });
                }
            }
            return out;
        }
        if schema.get("type") == Some(&Value::from("array")) {
            return self.props(&schema["items"]);
        }
        let required: BTreeSet<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        if let Some(props) = schema.get("properties").and_then(Value::as_object) {
            for (k, v) in props {
                out.insert(
                    k.clone(),
                    SpecProp {
                        required: required.contains(k.as_str()),
                        nullable: self.nullable(v),
                    },
                );
            }
        }
        out
    }

    fn op_body(&self, op: &str, part: &str) -> &Value {
        let (method, path) = op.split_once(' ').expect("op is `METHOD /path`");
        let o = &self.root["paths"][path][method.to_lowercase()];
        assert!(!o.is_null(), "op `{op}` not in spec snapshot");
        let content = if part == "request" {
            &o["requestBody"]["content"]
        } else {
            let responses = &o["responses"];
            if responses.get("200").is_some() {
                &responses["200"]["content"]
            } else {
                &responses["201"]["content"]
            }
        };
        let (_, media) = content
            .as_object()
            .and_then(|m| m.iter().next())
            .unwrap_or_else(|| panic!("op `{op}` has no {part} body"));
        &media["schema"]
    }

    /// Every property name declared anywhere in the spec.
    fn all_property_names(&self) -> BTreeSet<String> {
        fn walk(v: &Value, out: &mut BTreeSet<String>) {
            match v {
                Value::Object(map) => {
                    if let Some(Value::Object(props)) = map.get("properties") {
                        out.extend(props.keys().cloned());
                    }
                    for (k, child) in map {
                        if k == "parameters" {
                            for p in child.as_array().into_iter().flatten() {
                                if let Some(n) = p.get("name").and_then(Value::as_str) {
                                    out.insert(n.to_string());
                                }
                            }
                        }
                        walk(child, out);
                    }
                }
                Value::Array(xs) => xs.iter().for_each(|x| walk(x, out)),
                _ => {}
            }
        }
        let mut out = BTreeSet::new();
        walk(&self.root, &mut out);
        out
    }
}

// ---------------------------------------------------------------------------
// Map and allowlist
// ---------------------------------------------------------------------------

struct Mapping {
    rust: String,
    response: bool,
    props: BTreeMap<String, SpecProp>,
    source: String,
    /// A `*WebhookOut` schema, whose `webhook_event` the event enum consumes.
    webhook: bool,
}

fn load_json(rel: &str) -> Value {
    let path = manifest_dir().join(rel);
    serde_json::from_slice(&std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}")))
        .unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

fn load_mappings(spec: &Spec) -> Vec<Mapping> {
    load_json(".api-sync/contract-map.json")["types"]
        .as_array()
        .expect("contract-map.json has `types`")
        .iter()
        .map(|e| {
            let rust = e["rust"].as_str().unwrap().to_string();
            let response = e["direction"] == "response";
            let (schema, source) = if let Some(name) = e["schema"].as_str() {
                let s = &spec.root["components"]["schemas"][name];
                assert!(!s.is_null(), "schema {name} (for {rust}) not in spec");
                (s, format!("schema {name}"))
            } else {
                let op = e["op"].as_str().unwrap();
                let part = e["part"].as_str().unwrap();
                (spec.op_body(op, part), format!("{op} {part}"))
            };
            let webhook = e["schema"]
                .as_str()
                .is_some_and(|n| n.ends_with("WebhookOut"));
            Mapping {
                rust,
                response,
                props: spec.props(schema),
                source,
                webhook,
            }
        })
        .collect()
}

/// `(rust type, wire field)` pairs exempt from Direction A, with reasons.
fn load_allowlist() -> BTreeMap<(String, String), String> {
    load_allow_section("direction_a")
}

/// `(rust type, field)` pairs: spec properties deliberately left unmodelled.
fn load_unmodelled() -> BTreeMap<(String, String), String> {
    load_allow_section("unmodelled")
}

fn load_allow_section(section: &str) -> BTreeMap<(String, String), String> {
    load_json(".api-sync/contract-allowlist.json")[section]
        .as_array()
        .unwrap_or_else(|| panic!("contract-allowlist.json has `{section}`"))
        .iter()
        .map(|e| {
            let reason = e["reason"].as_str().unwrap_or_default();
            assert!(!reason.is_empty(), "allowlist entry without a reason: {e}");
            (
                (
                    e["rust"].as_str().unwrap().to_string(),
                    e["field"].as_str().unwrap().to_string(),
                ),
                reason.to_string(),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

#[test]
fn contract_check() {
    let rust = parse_rust_types();
    let spec = Spec::load();
    let mappings = load_mappings(&spec);
    let allowlist = load_allowlist();
    let unmodelled = load_unmodelled();
    let mut used_unmodelled: BTreeSet<(String, String)> = BTreeSet::new();
    let mut used_allow: BTreeSet<(String, String)> = BTreeSet::new();
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Direction A + C, mapped types.
    let mut mapped_names = BTreeSet::new();
    for m in &mappings {
        let structs = rust.find(&m.rust);
        assert!(
            !structs.is_empty(),
            "contract-map.json names `{}`, which isn't a serde struct in src/",
            m.rust
        );
        mapped_names.insert(m.rust.clone());
        for s in structs {
            for f in &s.fields {
                let key = (s.name.clone(), f.wire.clone());
                match m.props.get(&f.wire) {
                    None if allowlist.contains_key(&key) => {
                        used_allow.insert(key);
                    }
                    None => errors.push(format!(
                        "A: {}.{} ({}) is not a property of {}",
                        s.name, f.wire, s.file, m.source
                    )),
                    Some(p) if m.response && !f.optional && !f.has_default => {
                        if !p.required || p.nullable {
                            errors.push(format!(
                                "C: {}.{} ({}) is non-Option but {} in {}",
                                s.name,
                                f.wire,
                                s.file,
                                if p.nullable { "nullable" } else { "optional" },
                                m.source
                            ));
                        }
                    }
                    Some(p) if !m.response => {
                        if p.required && f.optional {
                            warnings.push(format!(
                                "request {}.{}: required in {} but Option",
                                s.name, f.wire, m.source
                            ));
                        } else if !p.required && !f.optional {
                            warnings.push(format!(
                                "request {}.{}: optional in {} but required",
                                s.name, f.wire, m.source
                            ));
                        }
                    }
                    Some(_) => {}
                }
            }
            let modelled: BTreeSet<&str> = s.fields.iter().map(|f| f.wire.as_str()).collect();
            let missing: Vec<&str> = m
                .props
                .keys()
                .map(String::as_str)
                .filter(|k| !modelled.contains(k))
                .filter(|k| !(m.webhook && *k == "webhook_event"))
                .filter(|k| {
                    let key = (s.name.clone(), (*k).to_string());
                    let known = unmodelled.contains_key(&key);
                    if known {
                        used_unmodelled.insert(key);
                    }
                    !known
                })
                .collect();
            if !missing.is_empty() {
                let line = format!(
                    "unmodelled in {} ({}): {}",
                    s.name,
                    m.source,
                    missing.join(", ")
                );
                if m.response {
                    // Model them, or list them under `unmodelled` with a reason.
                    errors.push(line);
                } else {
                    warnings.push(line);
                }
            }
        }
    }

    // Direction A, every other serde struct: field names must exist in the spec.
    let names = spec.all_property_names();
    for s in rust
        .structs
        .iter()
        .filter(|s| !mapped_names.contains(&s.name))
    {
        for f in &s.fields {
            let key = (s.name.clone(), f.wire.clone());
            if names.contains(&f.wire) {
                continue;
            }
            if allowlist.contains_key(&key) {
                used_allow.insert(key);
            } else {
                errors.push(format!(
                    "A: {}.{} ({}) is not a property name anywhere in the spec",
                    s.name, f.wire, s.file
                ));
            }
        }
    }

    // Direction B: WebhookEvent covers the spec's event names.
    let events = &spec.root["components"]["schemas"]["WebhookEndpointIn"]["properties"]["events"]["items"]
        ["enum"];
    let spec_events: BTreeSet<String> = events
        .as_array()
        .expect("WebhookEndpointIn.events enum")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let sdk_events: BTreeSet<String> = rust
        .enums
        .get("WebhookEvent")
        .expect("open_enum WebhookEvent")
        .iter()
        .cloned()
        .collect();
    for missing in spec_events.difference(&sdk_events) {
        errors.push(format!("B: WebhookEvent lacks spec event `{missing}`"));
    }
    for extra in sdk_events.difference(&spec_events) {
        warnings.push(format!(
            "WebhookEvent has `{extra}`, which the spec no longer lists"
        ));
    }

    for stale in unmodelled.keys().filter(|k| !used_unmodelled.contains(*k)) {
        errors.push(format!(
            "unmodelled entry {}.{} is modelled now (or unmapped); remove it",
            stale.0, stale.1
        ));
    }
    for stale in allowlist.keys().filter(|k| !used_allow.contains(*k)) {
        errors.push(format!(
            "allowlist entry {}.{} no longer matches anything; remove it",
            stale.0, stale.1
        ));
    }

    println!(
        "contract check: {} mapped types, {} serde structs, {} open enums, {} warnings",
        mappings.len(),
        rust.structs.len(),
        rust.enums.len(),
        warnings.len()
    );
    for w in &warnings {
        println!("  warn: {w}");
    }
    assert!(
        errors.is_empty(),
        "{} contract violation(s):\n{}",
        errors.len(),
        errors.join("\n")
    );
}
