//! Published JSON Schemas and the compatibility rule between versions (SNAP-3).

use std::collections::BTreeSet;

use serde_json::Value;

use crate::snapshot::{FactsFile, RunFile};

pub fn facts_schema() -> Value {
    serde_json::to_value(schemars::schema_for!(FactsFile)).expect("schemas serialize")
}

/// The published schema for `run.json`.
pub fn run_schema() -> Value {
    serde_json::to_value(schemars::schema_for!(RunFile)).expect("schemas serialize")
}

/// Every published schema by file stem: `schemas/<name>.schema.json`.
pub fn all() -> Vec<(&'static str, Value)> {
    vec![("facts", facts_schema()), ("run", run_schema())]
}

/// The major component of a `MAJOR.MINOR.PATCH` version.
pub fn major(version: &str) -> u64 {
    version
        .split('.')
        .next()
        .and_then(|m| m.parse().ok())
        .unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CompatError {
    #[error("fields removed without a major version bump: {0:?}")]
    Removed(Vec<String>),
}

/// Fails when a property present in `baseline` is missing from `current`.
/// Additions are allowed. Paths join property names (and `$defs` entries)
/// with `/`; combinators like `oneOf` are merged, so reordering variants is
/// not a removal.
pub fn check_compatible(baseline: &Value, current: &Value) -> Result<(), CompatError> {
    let have = property_paths(current);
    let removed: Vec<String> = property_paths(baseline)
        .difference(&have)
        .cloned()
        .collect();
    if removed.is_empty() {
        Ok(())
    } else {
        Err(CompatError::Removed(removed))
    }
}

fn property_paths(schema: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    walk(schema, "", &mut out);
    out
}

fn walk(node: &Value, path: &str, out: &mut BTreeSet<String>) {
    match node {
        Value::Object(map) => {
            for (key, child) in map {
                match key.as_str() {
                    "properties" | "$defs" | "definitions" => {
                        if let Value::Object(entries) = child {
                            for (name, sub) in entries {
                                let name = if key == "properties" {
                                    name.clone()
                                } else {
                                    format!("$defs:{name}")
                                };
                                let p = format!("{path}/{name}");
                                out.insert(p.clone());
                                walk(sub, &p, out);
                            }
                        }
                    }
                    _ => walk(child, path, out),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|i| walk(i, path, out)),
        _ => {}
    }
}
