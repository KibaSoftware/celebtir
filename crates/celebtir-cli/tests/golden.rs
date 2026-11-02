//! QA-3: the whole pipeline (collect → score) replays recorded mainnet
//! responses offline and must reproduce the committed golden outputs.
//! CLI-4: `collect --offline` needs no network.
//!
//! Regenerate the goldens after an intended change with
//! `CELEBTIR_UPDATE_GOLDEN=1 cargo test -p celebtir --test golden`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn celebtir(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        // Point any accidental network use at nothing.
        .env("CELEBTIR_RPC_URL", "http://127.0.0.1:9")
        .output()
        .unwrap()
}

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// collect --offline then score, into `dir`.
fn pipeline(dir: &Path, replay: &Path) -> std::process::Output {
    let registry = repo().join("fixtures/golden/registry");
    let collected = celebtir(&[
        "collect",
        "--registry",
        &s(&registry),
        "--offline",
        &s(replay),
        "--tx-sample",
        "5",
        "--history-limit",
        "5",
        "--date",
        "2026-10-17",
        "--out",
        &s(dir),
        "--cache",
        &s(&dir.join("cache.json")),
    ]);
    if collected.status.code() == Some(0) {
        let scored = celebtir(&[
            "score",
            "--facts",
            &s(&dir.join("facts.json")),
            "--methodology",
            &s(&repo().join("methodology/v0.1.toml")),
        ]);
        assert_eq!(
            scored.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&scored.stderr)
        );
    }
    collected
}

/// The first differing lines, readable in a CI log.
fn diff(expected: &str, actual: &str) -> String {
    let (e, a): (Vec<&str>, Vec<&str>) = (expected.lines().collect(), actual.lines().collect());
    let mut out = Vec::new();
    for i in 0..e.len().max(a.len()) {
        let (x, y) = (
            e.get(i).copied().unwrap_or("<end>"),
            a.get(i).copied().unwrap_or("<end>"),
        );
        if x != y {
            out.push(format!("line {}:\n  - {x}\n  + {y}", i + 1));
            if out.len() == 15 {
                out.push("…".into());
                break;
            }
        }
    }
    out.join("\n")
}

#[test]
fn qa_3_offline_pipeline_reproduces_the_golden_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let o = pipeline(dir.path(), &repo().join("fixtures/golden/replay"));
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );

    let expected_dir = repo().join("fixtures/golden/expected");
    let mut problems = Vec::new();
    for file in ["facts.json", "scores.json"] {
        let actual = std::fs::read_to_string(dir.path().join(file)).unwrap();
        let golden = expected_dir.join(file);
        if std::env::var_os("CELEBTIR_UPDATE_GOLDEN").is_some() {
            std::fs::create_dir_all(&expected_dir).unwrap();
            std::fs::write(&golden, &actual).unwrap();
        }
        let expected = std::fs::read_to_string(&golden).unwrap_or_default();
        if expected != actual {
            problems.push(format!(
                "{file} differs from its golden:\n{}",
                diff(&expected, &actual)
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "{}\n(regenerate with CELEBTIR_UPDATE_GOLDEN=1 if the change is intended)",
        problems.join("\n\n")
    );
}

#[test]
fn cli_4_offline_collect_uses_no_network_and_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let o = pipeline(dir.path(), &repo().join("fixtures/golden/replay"));
    assert_eq!(o.status.code(), Some(0));
    let run: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("run.json")).unwrap())
            .unwrap();
    assert_eq!(run["rpc_host"], "offline");
    assert!(!String::from_utf8_lossy(&o.stderr).contains("CELEBTIR_RPC_URL is not set"));
}

#[test]
fn cli_4_offline_with_missing_recordings_fails_per_protocol() {
    let dir = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let o = pipeline(dir.path(), empty.path());
    assert_eq!(o.status.code(), Some(2), "partial failure, not a crash");
    let run: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("run.json")).unwrap())
            .unwrap();
    let failures = run["failures"].as_array().unwrap();
    assert!(
        failures.iter().any(|f| f["error"]
            .as_str()
            .unwrap()
            .contains("no recorded response")),
        "{failures:#?}"
    );
}

#[test]
fn ui_12_facts_carry_registry_sources_and_reviewer_for_the_protocol_page() {
    let dir = tempfile::tempdir().unwrap();
    let o = pipeline(dir.path(), &repo().join("fixtures/golden/replay"));
    assert_eq!(o.status.code(), Some(0));
    let facts: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("facts.json")).unwrap())
            .unwrap();
    let kamino = &facts["protocols"]["kamino-lend"];
    assert_eq!(kamino["reviewed_by"], "golden-fixture");
    let sources = kamino["sources"].as_array().unwrap();
    let kinds: Vec<&str> = sources
        .iter()
        .map(|s| s["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "admin", "multisig", "oracle", "oracle", "program", "program"
        ],
        "sorted by kind, then address"
    );
    let lending = sources
        .iter()
        .find(|s| s["address"] == "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD")
        .unwrap();
    assert_eq!(lending["label"], "Lending");
    assert_eq!(lending["url"], "https://github.com/Kamino-Finance/klend");
}

#[test]
fn tok_1_token_facts_carry_issuer_type_and_source_for_the_token_page() {
    let dir = tempfile::tempdir().unwrap();
    let o = pipeline(dir.path(), &repo().join("fixtures/golden/replay"));
    assert_eq!(o.status.code(), Some(0));
    let facts: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("facts.json")).unwrap())
            .unwrap();
    let usdc = &facts["tokens"]["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"]["identity"];
    assert_eq!(usdc["kind"], "stablecoin");
    assert_eq!(usdc["issuer"], "Circle");
    assert_eq!(usdc["issuer_url"], "https://www.circle.com/usdc");
    assert_eq!(
        usdc["source"],
        "https://developers.circle.com/stablecoins/usdc-contract-addresses"
    );
    let jito = &facts["tokens"]["J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"]["identity"];
    assert_eq!(jito["kind"], "liquid-staking");
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}

/// ADR 017: archived registry entries ride along in facts.json.
#[test]
fn archived_registry_entries_are_published_in_the_facts() {
    let reg = tempfile::tempdir().unwrap();
    copy_dir(&repo().join("fixtures/golden/registry"), reg.path());
    std::fs::write(
        reg.path().join("archived.yaml"),
        "archived:\n  - id: old-dex\n    name: Old DEX\n    archived_on: 2026-09-01\n    reason: Shut down by its team.\n    source: https://example.com/sunset\n",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let o = celebtir(&[
        "collect",
        "--registry",
        &s(reg.path()),
        "--offline",
        &s(&repo().join("fixtures/golden/replay")),
        "--tx-sample",
        "5",
        "--history-limit",
        "5",
        "--date",
        "2026-10-17",
        "--out",
        &s(dir.path()),
        "--cache",
        &s(&dir.path().join("cache.json")),
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let facts: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("facts.json")).unwrap())
            .unwrap();
    assert_eq!(
        facts["archived"]["old-dex"]["reason"],
        "Shut down by its team."
    );
    assert_eq!(facts["archived"]["old-dex"]["archived_on"], "2026-09-01");
}
