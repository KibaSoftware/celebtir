//! SNAP-3 for events.json: the committed schema matches the code.

#[test]
fn snap_3_committed_events_schema_matches_the_code() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/events.schema.json");
    let generated = celebtir_core::to_canonical_json(&celebtir_diff::events_schema()).unwrap();
    if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap_or_default(),
        generated,
        "stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
    );
}
