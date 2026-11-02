//! The published `scores.json` schema matches the code (SNAP-3), and real
//! scorer output validates against it.

mod common;

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{AdminPower, FactsFile, ProgramRole};
use celebtir_score::{Methodology, score, scores_schema};
use common::*;

fn schema() -> serde_json::Value {
    serde_json::to_value(scores_schema()).unwrap()
}

#[test]
fn snap_3_committed_scores_schema_matches_the_code() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/scores.schema.json");
    let generated = celebtir_core::to_canonical_json(&schema()).unwrap();
    if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
    );
}

#[test]
fn snap_3_scores_output_validates_against_its_schema() {
    let mut p = protocol(vec![
        (
            "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
            ProgramRole::Core,
            multisig(3, 5, 0),
        ),
        (
            "FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr",
            ProgramRole::Periphery,
            single_key(),
        ),
    ]);
    p.admin = admin(vec![
        (AdminPower::Pause, single_key()),
        (AdminPower::Funds, Controller::Immutable),
    ]);
    let mut facts = FactsFile::new(BTreeMap::from([("demo".to_string(), p)]));
    facts.tokens.insert(
        addr("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        token(),
    );
    let out = serde_json::to_value(score(&facts, &Methodology::bundled("0.2").unwrap())).unwrap();
    let errors: Vec<String> = jsonschema_errors(&schema(), &out);
    assert!(errors.is_empty(), "{errors:#?}");
}

fn jsonschema_errors(schema: &serde_json::Value, instance: &serde_json::Value) -> Vec<String> {
    let v = jsonschema::validator_for(schema).unwrap();
    v.iter_errors(instance)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect()
}
