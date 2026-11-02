//! OPS-2: the daily pipeline script runs every step in order, offline here,
//! and a failing step leaves published data unchanged.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn daily(data: &Path, registry: &Path, date: &str) -> std::process::Output {
    let s = |p: &Path| p.to_string_lossy().replace('\\', "/");
    Command::new("bash")
        .arg(repo().join("scripts/daily.sh"))
        .args([
            "--date",
            date,
            "--registry",
            &s(registry),
            "--data",
            &s(data),
            "--bin",
            &s(Path::new(env!("CARGO_BIN_EXE_celebtir"))),
        ])
        .args([
            "--offline",
            &s(&repo().join("fixtures/golden/replay")),
            "--tx-sample",
            "5",
            "--history-limit",
            "5",
        ])
        .env_remove("CELEBTIR_RPC_URL")
        .output()
        .unwrap()
}

#[test]
fn ops_2_two_daily_runs_produce_snapshots_events_and_site_data() {
    let data = tempfile::tempdir().unwrap();
    let registry = repo().join("fixtures/golden/registry");
    let first = daily(data.path(), &registry, "2026-10-16");
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}\n{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let second = daily(data.path(), &registry, "2026-10-17");
    assert_eq!(
        second.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let day = data.path().join("snapshots/2026-10-17");
    for f in ["facts.json", "scores.json", "run.json", "events.json"] {
        assert!(day.join(f).is_file(), "{f}");
    }
    // F6: the first day reports only its recorded ProgramData history.
    let first: celebtir_diff::EventsFile = serde_json::from_str(
        &std::fs::read_to_string(data.path().join("snapshots/2026-10-16/events.json")).unwrap(),
    )
    .unwrap();
    assert!(first.previous_date.is_empty());
    assert!(
        first
            .events
            .iter()
            .all(|e| e.backfill && e.signature.is_some())
    );
    assert_eq!(
        std::fs::read(data.path().join("latest/facts.json")).unwrap(),
        std::fs::read(day.join("facts.json")).unwrap()
    );
    let h: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(data.path().join("history/kamino-lend.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(h["days"].as_array().unwrap().len(), 2);
    assert!(data.path().join("feeds/all.atom").is_file());
    let out = String::from_utf8_lossy(&second.stdout);
    assert!(
        out.contains("data: 2026-10-17: 2 protocols"),
        "summary line for the commit message: {out}"
    );
}

#[test]
fn ops_2_a_failing_step_leaves_published_data_unchanged() {
    let data = tempfile::tempdir().unwrap();
    let good = repo().join("fixtures/golden/registry");
    assert_eq!(
        daily(data.path(), &good, "2026-10-16").status.code(),
        Some(0)
    );
    let before = std::fs::read(data.path().join("latest/facts.json")).unwrap();

    let broken = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(broken.path().join("solana")).unwrap();
    std::fs::write(broken.path().join("solana/x.yaml"), "id: [broken").unwrap();
    let o = daily(data.path(), broken.path(), "2026-10-17");
    assert_ne!(o.status.code(), Some(0));
    assert!(
        !data.path().join("snapshots/2026-10-17").exists(),
        "no snapshot for a failed day"
    );
    assert_eq!(
        std::fs::read(data.path().join("latest/facts.json")).unwrap(),
        before
    );
}

#[test]
fn ops_2_qa_same_date_cannot_overwrite_published_snapshot() {
    let data = tempfile::tempdir().unwrap();
    let day = data.path().join("snapshots/2026-10-21");
    std::fs::create_dir_all(&day).unwrap();
    std::fs::write(day.join("facts.json"), "preserve me").unwrap();
    let out = daily(
        data.path(),
        &repo().join("fixtures/golden/registry"),
        "2026-10-21",
    );
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(
        std::fs::read_to_string(day.join("facts.json")).unwrap(),
        "preserve me"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
}
