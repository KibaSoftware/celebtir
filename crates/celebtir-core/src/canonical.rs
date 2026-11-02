use serde::Serialize;
use serde_json::{Map, Value};

/// Serialize to pretty JSON with object keys sorted at every level and a
/// trailing newline, so identical inputs give byte-identical files (SNAP-2).
///
/// Keys are sorted here rather than relying on serde_json's default map, which
/// switches to insertion order if any crate enables `preserve_order`.
/// Arrays keep their order: producers keep collections in maps keyed by stable
/// IDs, or sort them before serializing.
pub fn to_canonical_json<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<String> {
    let mut out = serde_json::to_string_pretty(&sort_keys(serde_json::to_value(value)?))?;
    out.push('\n');
    Ok(out)
}

fn sort_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut sorted = Map::new();
            for (k, v) in entries {
                sorted.insert(k, sort_keys(v));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_keys).collect()),
        other => other,
    }
}
