//! `celebtir build-site-data`: data/latest/ is the newest snapshot, and
//! data/history/<protocol>.json is a compact daily series (VAL-1, VAL-5).

mod common;

use std::path::Path;
use std::process::Command;

use celebtir_core::snapshot::{Metric, ProtocolMetrics};
use celebtir_core::to_canonical_json;
use common::*;

fn celebtir(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        .output()
        .unwrap()
}

fn day(root: &Path, date: &str, tvl: Option<u64>) {
    let dir = root.join("snapshots").join(date);
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = sample();
    for id in f.protocols.keys().cloned().collect::<Vec<_>>() {
        f.metrics.insert(
            id,
            ProtocolMetrics {
                defillama_slug: Some("x".into()),
                tvl: tvl.map_or(
                    Metric::Unavailable {
                        reason: "down".into(),
                    },
                    |value| Metric::Usd { value },
                ),
                asset_groups: Default::default(),
                fees: Default::default(),
                volume: Default::default(),
            },
        );
    }
    std::fs::write(dir.join("facts.json"), to_canonical_json(&f).unwrap()).unwrap();
    assert_eq!(
        celebtir(&["score", "--facts", dir.join("facts.json").to_str().unwrap()])
            .status
            .code(),
        Some(0)
    );
    std::fs::write(dir.join("run.json"), "{}\n").unwrap();
}

fn build(root: &Path) -> std::process::Output {
    celebtir(&[
        "build-site-data",
        "--data",
        root.to_str().unwrap(),
        "--as-of",
        "2026-10-17",
    ])
}

fn history(root: &Path, id: &str) -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(root.join("history").join(format!("{id}.json"))).unwrap(),
    )
    .unwrap()
}

#[test]
fn aud_6_upgrade_metrics_keep_prior_transaction_evidence_after_a_cold_scan() {
    let root = tempfile::tempdir().unwrap();
    for date in ["2026-10-15", "2026-10-16", "2026-10-17"] {
        day(root.path(), date, Some(1));
        let path = root.path().join("snapshots").join(date).join("facts.json");
        let mut facts: celebtir_core::snapshot::FactsFile =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let program = facts.protocols["sample-lend"]
            .programs
            .iter()
            .find(|(_, p)| p.role == celebtir_core::snapshot::ProgramRole::Core)
            .unwrap()
            .0
            .clone();
        facts.program_history.insert(
            program,
            celebtir_core::snapshot::ProgramHistory {
                events: if date == "2026-10-17" {
                    vec![]
                } else {
                    vec![celebtir_core::snapshot::UpgradeEvent {
                        signature: "same-transaction".into(),
                        slot: 42,
                        date: Some("2026-10-14".into()),
                        kind: celebtir_core::snapshot::UpgradeKind::Upgrade,
                        new_authority: None,
                    }]
                },
                scanned: 0,
                total: 0,
                complete: false,
            },
        );
        std::fs::write(path, to_canonical_json(&facts).unwrap()).unwrap();
    }
    assert!(build(root.path()).status.success());
    let metrics = history(root.path(), "sample-lend")["upgrades"].clone();
    assert_eq!(metrics["upgrades"], 1);
    assert_eq!(metrics["last_upgrade_date"], "2026-10-14");
    assert_eq!(metrics["complete"], false);
    assert!(build(root.path()).status.success());
    assert_eq!(history(root.path(), "sample-lend")["upgrades"], metrics);
}

#[test]
fn val_1_seven_runs_produce_a_seven_point_tvl_series() {
    let root = tempfile::tempdir().unwrap();
    for i in 0..7u64 {
        day(
            root.path(),
            &format!("2026-10-{:02}", 11 + i),
            Some(1_000 + i),
        );
    }
    let o = build(root.path());
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let h = history(root.path(), "sample-lend");
    let days = h["days"].as_array().unwrap();
    assert_eq!(days.len(), 7);
    assert_eq!(days[0]["date"], "2026-10-11");
    assert_eq!(days[6]["tvl_usd"], 1_006);
    assert_eq!(days[6]["stage"], "stage_0");
    assert_eq!(days[6]["ratings"]["upgrade_control"], "medium");
    assert_eq!(days[6]["methodology_version"], "0.3");
    assert_eq!(days[6]["assessment"], "assessed");
}

#[test]
fn site_data_latest_is_a_copy_of_the_newest_snapshot() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-15", Some(5));
    day(root.path(), "2026-10-16", Some(6));
    assert_eq!(build(root.path()).status.code(), Some(0));
    let latest = root.path().join("latest");
    let newest = root.path().join("snapshots/2026-10-16");
    for f in ["facts.json", "scores.json", "run.json"] {
        assert_eq!(
            std::fs::read(latest.join(f)).unwrap(),
            std::fs::read(newest.join(f)).unwrap(),
            "{f}"
        );
    }
}

#[test]
fn val_7_missing_tvl_days_stay_in_the_series_as_null() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-15", Some(5));
    day(root.path(), "2026-10-16", None);
    build(root.path());
    let days = history(root.path(), "sample-dex")["days"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(days.len(), 2);
    assert!(days[1]["tvl_usd"].is_null());
}

#[test]
fn val_5_history_files_carry_upgrade_metrics_as_of_the_build_date() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-16", Some(1));
    build(root.path());
    let h = history(root.path(), "sample-lend");
    assert_eq!(h["upgrades"]["upgrades"], 0);
    assert_eq!(
        h["upgrades"]["complete"], false,
        "the sample has no history read"
    );
}

#[test]
fn site_data_without_snapshots_fails_clearly() {
    let root = tempfile::tempdir().unwrap();
    let o = build(root.path());
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("no snapshots"));
}

#[test]
fn val_8_summary_series_has_total_tvl_value_by_control_and_stage_distribution() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-15", Some(100));
    day(root.path(), "2026-10-16", Some(200));
    assert_eq!(build(root.path()).status.code(), Some(0));
    let s: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.path().join("history/summary.json")).unwrap(),
    )
    .unwrap();
    let days = s["days"].as_array().unwrap();
    assert_eq!(days.len(), 2);
    let d = &days[1];
    assert_eq!(d["date"], "2026-10-16");
    assert_eq!(d["total_tvl_usd"], 600, "three protocols at 200 each");
    let classes = d["value_by_control"]["classes"].as_object().unwrap();
    assert_eq!(
        classes.values().map(|v| v.as_u64().unwrap()).sum::<u64>(),
        600
    );
    // v0.3: the sample DEX lists no admin accounts and no review, so its
    // admin powers are Unknown and its Stage is capped at 0 (F3).
    assert_eq!(d["stages"]["stage_0"], 2);
    assert_eq!(d["stages"]["stage_1"], 0);
    assert_eq!(d["stages"]["unrated"], 1);
    assert_eq!(d["protocols"], 3);
    assert_eq!(d["unassessed"], 0);
    assert_eq!(d["methodology_version"], "0.3");
}

#[test]
fn tok_9_thirty_snapshots_give_a_thirty_day_supply_line() {
    let root = tempfile::tempdir().unwrap();
    for i in 0..30 {
        let date = celebtir_core::date::ymd(
            (celebtir_core::date::days("2026-09-18").unwrap() + i) * 86_400,
        );
        day(root.path(), &date, Some(1));
    }
    assert_eq!(build(root.path()).status.code(), Some(0));
    let path = root
        .path()
        .join("history/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v.json");
    let t: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(t["symbol"], "USDC");
    let days = t["days"].as_array().unwrap();
    assert_eq!(days.len(), 30);
    assert_eq!(days[29]["date"], "2026-10-17");
    assert_eq!(days[0]["supply"], 1_000_000);
    assert_eq!(days[0]["decimals"], 6);
    assert!(
        days[0]["price_usd_micros"].is_null(),
        "no price collected in the sample"
    );
}

#[test]
fn reg_7_an_old_review_date_surfaces_the_note_after_the_next_build() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-16", Some(1));
    // Rewrite the day's facts with review dates, then rebuild.
    let path = root.path().join("snapshots/2026-10-16/facts.json");
    let mut facts: celebtir_core::snapshot::FactsFile =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    facts
        .protocols
        .get_mut("sample-lend")
        .unwrap()
        .last_reviewed = Some("2026-06-01".into());
    facts.protocols.get_mut("sample-dex").unwrap().last_reviewed = Some("2026-09-30".into());
    std::fs::write(&path, to_canonical_json(&facts).unwrap()).unwrap();
    build(root.path());
    let lend = history(root.path(), "sample-lend");
    assert_eq!(lend["review"]["due"], true, "138 days > 90");
    assert_eq!(lend["review"]["last_reviewed"], "2026-06-01");
    assert_eq!(history(root.path(), "sample-dex")["review"]["due"], false);
    assert_eq!(
        history(root.path(), "sample-perps")["review"]["due"],
        true,
        "no review date counts as due"
    );
}

/// ADR 017: a protocol that left coverage keeps its last known ratings.
#[test]
fn archived_protocols_keep_their_last_known_ratings_and_reason() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-15", Some(100));
    // The next day sample-perps is archived: gone from the facts, listed instead.
    let dir = root.path().join("snapshots").join("2026-10-16");
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = sample();
    f.protocols.remove("sample-perps");
    f.archived.insert(
        "sample-perps".into(),
        celebtir_core::snapshot::ArchivedProtocol {
            name: "Sample Perps".into(),
            archived_on: "2026-10-16".into(),
            reason: "Shut down by its team.".into(),
            source: "https://example.com/sunset".into(),
        },
    );
    std::fs::write(dir.join("facts.json"), to_canonical_json(&f).unwrap()).unwrap();
    assert_eq!(
        celebtir(&["score", "--facts", dir.join("facts.json").to_str().unwrap()])
            .status
            .code(),
        Some(0)
    );
    std::fs::write(dir.join("run.json"), "{}\n").unwrap();
    assert_eq!(build(root.path()).status.code(), Some(0));

    let archived: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.path().join("history/archived.json")).unwrap(),
    )
    .unwrap();
    let list = archived["protocols"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    let p = &list[0];
    assert_eq!(p["id"], "sample-perps");
    assert_eq!(p["reason"], "Shut down by its team.");
    assert_eq!(p["last_known"]["date"], "2026-10-15");
    assert!(p["last_known"]["ratings"].as_object().unwrap().len() == 6);
}

#[test]
fn archived_file_is_written_even_when_nothing_is_archived() {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-15", Some(100));
    assert_eq!(build(root.path()).status.code(), Some(0));
    let archived: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.path().join("history/archived.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(archived["protocols"], serde_json::json!([]));
}
