//! `registry/archived.yaml` (ADR 017): protocols that shut down or left
//! coverage, with a date, a reason and a source, for the /archived page.

mod common;

use celebtir_registry::validate;
use common::Registry;

const ARCHIVED: &str = r#"
archived:
  - id: old-dex
    name: Old DEX
    archived_on: 2026-09-01
    reason: Shut down by its team; withdrawals only.
    source: https://example.com/old-dex-sunset
"#;

fn errors(r: &Registry) -> Vec<String> {
    validate(r.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

#[test]
fn archived_entries_load_with_date_reason_and_source() {
    let r = Registry::valid();
    r.write("archived.yaml", ARCHIVED);
    let report = validate(r.path());
    assert!(report.is_ok(), "{:?}", report.diagnostics);
    assert_eq!(report.archived.len(), 1);
    assert_eq!(report.archived[0].id, "old-dex");
    assert_eq!(report.archived[0].archived_on, "2026-09-01");
}

#[test]
fn archived_file_is_optional() {
    let r = Registry::valid();
    let report = validate(r.path());
    assert!(report.is_ok(), "{:?}", report.diagnostics);
    assert!(report.archived.is_empty());
}

#[test]
fn an_archived_id_cannot_also_be_an_active_protocol() {
    let r = Registry::valid();
    r.write("archived.yaml", &ARCHIVED.replace("old-dex", "kamino-lend"));
    assert!(
        errors(&r)
            .iter()
            .any(|e| e.contains("archived.yaml") && e.contains("kamino-lend")),
        "{:?}",
        errors(&r)
    );
}

#[test]
fn archived_entries_need_a_real_date_a_reason_and_unique_ids() {
    let r = Registry::valid();
    r.write(
        "archived.yaml",
        &ARCHIVED.replace("2026-09-01", "2026-13-45"),
    );
    assert!(
        errors(&r).iter().any(|e| e.contains("archived_on")),
        "{:?}",
        errors(&r)
    );

    r.write(
        "archived.yaml",
        &ARCHIVED.replace("    reason: Shut down by its team; withdrawals only.\n", ""),
    );
    assert!(
        errors(&r).iter().any(|e| e.contains("reason")),
        "{:?}",
        errors(&r)
    );

    let twice = format!("{ARCHIVED}{}", &ARCHIVED["\narchived:\n".len()..]);
    r.write("archived.yaml", &twice);
    assert!(
        errors(&r).iter().any(|e| e.contains("duplicate")),
        "{:?}",
        errors(&r)
    );
}

#[test]
fn archived_committed_schema_matches_the_code() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/registry-archived.schema.json");
    let generated =
        celebtir_core::to_canonical_json(&celebtir_registry::archived_schema()).unwrap();
    if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
    );
}
