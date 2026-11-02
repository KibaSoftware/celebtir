//! OPS-6: each run reports projected monthly usage per external service and
//! flags it above 70 % of the free tier.

use std::collections::BTreeMap;
use std::process::Command;

use celebtir_core::snapshot::{RUN_SCHEMA_VERSION, RunFile};

fn summarize(calls: u64) -> serde_json::Value {
    let dir = tempfile::tempdir().unwrap();
    let day = dir.path().join("2026-10-17");
    std::fs::create_dir_all(&day).unwrap();
    let run = RunFile {
        schema_version: RUN_SCHEMA_VERSION.into(),
        tool_version: "0.1.0".into(),
        started_at: "2026-10-17T02:17:00Z".into(),
        finished_at: "2026-10-17T02:20:00Z".into(),
        duration_ms: 1,
        rpc_host: "example".into(),
        calls: BTreeMap::from([("getMultipleAccounts".to_string(), calls)]),
        retries: 0,
        budget: 100_000,
        budget_exceeded: false,
        protocols: vec!["a".into()],
        failures: vec![],
        review_needed: vec![],
        hyperliquid: None,
    };
    std::fs::write(
        day.join("run.json"),
        celebtir_core::to_canonical_json(&run).unwrap(),
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(["--format", "json", "summarize", day.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

#[test]
fn ops_6_typical_usage_is_well_under_the_free_tier() {
    // The PRD's estimate: about 2,000 calls a day, about 6 % of 1M a month.
    let s = summarize(2_000);
    assert_eq!(s["rpc_projected_monthly_pct"], 6);
    assert_eq!(s["usage_warning"], false);
}

#[test]
fn ops_6_raising_call_volume_triggers_the_warning() {
    let s = summarize(25_000);
    assert_eq!(s["rpc_projected_monthly_pct"], 75);
    assert_eq!(s["usage_warning"], true);
}
