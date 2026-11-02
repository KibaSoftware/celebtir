//! SNAP-4: `celebtir score --facts <file>` reproduces the published
//! scores.json for any past day, byte for byte, using the methodology version
//! that day was scored with.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Re-scores `facts` into a temp file with `methodology/v<version>.toml` and
/// returns the bytes.
fn rescore(facts: &Path, version: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("scores.json");
    let methodology = repo().join(format!("methodology/v{version}.toml"));
    assert!(
        methodology.exists(),
        "methodology {version} must stay in methodology/ forever (SNAP-4)"
    );
    let o = Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args([
            "score",
            "--facts",
            facts.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--methodology",
            methodology.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    std::fs::read_to_string(out).unwrap()
}

fn version_of(scores: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(scores).unwrap();
    v["methodology_version"].as_str().unwrap().to_string()
}

#[test]
fn snap_4_rescoring_the_golden_facts_matches_the_golden_scores_byte_for_byte() {
    let published =
        std::fs::read_to_string(repo().join("fixtures/golden/expected/scores.json")).unwrap();
    let again = rescore(
        &repo().join("fixtures/golden/expected/facts.json"),
        &version_of(&published),
    );
    assert!(
        again == published,
        "re-scored golden facts differ from the golden scores"
    );
}

/// Every day under data/snapshots that has both files must re-score exactly.
/// Vacuous until the daily pipeline starts committing snapshots.
#[test]
fn snap_4_every_published_day_rescores_identically() {
    let root = repo().join("data/snapshots");
    let mut checked = 0;
    for day in std::fs::read_dir(&root).into_iter().flatten().flatten() {
        let (facts, scores) = (
            day.path().join("facts.json"),
            day.path().join("scores.json"),
        );
        if !(facts.exists() && scores.exists()) {
            continue;
        }
        let published = std::fs::read_to_string(&scores).unwrap();
        let again = rescore(&facts, &version_of(&published));
        assert!(
            again == published,
            "{} does not re-score to its published scores.json",
            day.path().display()
        );
        checked += 1;
    }
    eprintln!("re-scored {checked} published days");
}

/// Every replay fixture (one per published methodology, `fixtures/replay/`)
/// re-scores with its own methodology to its committed scores, byte for byte,
/// under today's code: v0.1 and v0.2 stay replayable after v0.3 (launch audit).
#[test]
fn snap_4_replay_fixtures_rescore_byte_identically_under_their_methodology() {
    let root = repo().join("fixtures/replay");
    let mut checked = Vec::new();
    for version in std::fs::read_dir(&root).unwrap().flatten() {
        if !version.path().is_dir() {
            continue;
        }
        for day in std::fs::read_dir(version.path()).unwrap().flatten() {
            let published = std::fs::read_to_string(day.path().join("scores.json")).unwrap();
            let v = version_of(&published);
            assert_eq!(
                format!("v{v}"),
                version.file_name().to_string_lossy(),
                "{}",
                day.path().display()
            );
            let again = rescore(&day.path().join("facts.json"), &v);
            assert!(
                again == published,
                "{} no longer re-scores to its published scores.json",
                day.path().display()
            );
            checked.push(v);
        }
    }
    checked.sort();
    assert_eq!(
        checked,
        ["0.1", "0.2"],
        "a replay fixture per legacy methodology"
    );
}
