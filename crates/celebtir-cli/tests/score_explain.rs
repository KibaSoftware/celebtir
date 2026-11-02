//! `celebtir score` and `celebtir explain` over the hand-built sample facts
//! (slice 2.12), including CLI-3: `explain --format json` validates against
//! the scores schema.

mod common;

use std::process::{Command, Output};

use common::*;

fn celebtir(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        .env_remove("CELEBTIR_RPC_URL")
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn sample_facts_fixture_is_current() {
    let generated = celebtir_core::to_canonical_json(&sample()).unwrap();
    let path = sample_path();
    if std::env::var_os("CELEBTIR_UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "stale; run with CELEBTIR_UPDATE_FIXTURES=1"
    );
}

#[test]
fn score_writes_canonical_scores_json() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("scores.json");
    let o = celebtir(&[
        "score",
        "--facts",
        sample_path().to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let text = std::fs::read_to_string(&out).unwrap();
    let scores: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(scores["methodology_version"], "0.3");
    assert_eq!(
        scores["protocols"]["sample-lend"]["stage"]["stage"],
        "stage_0"
    );
    // v0.3 (F3): no admin accounts read and no review that found none is
    // Unknown, which caps the Stage at 0.
    assert_eq!(
        scores["protocols"]["sample-dex"]["stage"]["stage"],
        "stage_0"
    );
    assert_eq!(
        scores["protocols"]["sample-perps"]["stage"]["stage"],
        "unrated"
    );
    assert_eq!(
        text,
        celebtir_core::to_canonical_json(&scores).unwrap(),
        "output is canonical"
    );
    assert!(stdout(&o).contains("scored 3 protocols and 1 tokens"));
}

#[test]
fn score_with_a_methodology_file_changes_ratings() {
    let dir = tempfile::tempdir().unwrap();
    let methodology = dir.path().join("strict.toml");
    std::fs::write(
        &methodology,
        celebtir_score::SHIPPED_METHODOLOGY
            .replace("low_min_secs = 432000", "low_min_secs = 864000"),
    )
    .unwrap();
    let out = dir.path().join("scores.json");
    let o = celebtir(&[
        "score",
        "--facts",
        sample_path().to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--methodology",
        methodology.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0));
    let scores: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(
        scores["protocols"]["sample-dex"]["factors"]["upgrade_delay"]["rating"],
        "medium"
    );
}

#[test]
fn explain_prints_factors_evidence_next_steps_and_missing_items() {
    let o = celebtir(&[
        "explain",
        "sample-lend",
        "--facts",
        sample_path().to_str().unwrap(),
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let text = stdout(&o);
    for needle in [
        "sample-lend  ·  Stage 0  ·  slot 382,114,900",
        "Upgrade control   MEDIUM  Squads v4 multisig 6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM, 2 of 4 voters, no time lock",
        "evidence: upgrade_authority 9FtmuBFcCqRHGaqKSAYHwLjcHTvn7WZ1ZnkWgytXu3EA @ 382,114,880",
        "next: a multisig of 5+ voters at a 60%+ threshold → LOW",
        "Upgrade delay     HIGH",
        "Periphery programs (shown, not counted)",
        "Missing for Stage 1:",
    ] {
        assert!(text.contains(needle), "missing `{needle}` in:\n{text}");
    }
}

#[test]
fn explain_unrated_protocol_lists_what_stage_0_needs() {
    let o = celebtir(&[
        "explain",
        "sample-perps",
        "--facts",
        sample_path().to_str().unwrap(),
    ]);
    let text = stdout(&o);
    assert!(
        text.contains("Unrated") && text.contains("Missing for Stage 0:"),
        "{text}"
    );
    assert!(text.contains("RPC timeout"), "{text}");
}

#[test]
fn explain_unknown_protocol_exits_1_and_lists_known_ids() {
    let o = celebtir(&[
        "explain",
        "nope",
        "--facts",
        sample_path().to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("sample-lend"));
}

#[test]
fn cli_3_explain_json_validates_against_the_scores_schema() {
    let o = celebtir(&[
        "explain",
        "sample-lend",
        "--facts",
        sample_path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let instance: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(instance["protocols"].as_object().unwrap().len(), 1);
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo().join("schemas/scores.schema.json")).unwrap(),
    )
    .unwrap();
    let v = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = v.iter_errors(&instance).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:#?}");
}
