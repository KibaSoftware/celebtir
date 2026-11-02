//! SNAP-5 (secrets never in output) and SNAP-6 (size budgets).

use std::path::{Path, PathBuf};

use celebtir_core::budget::{
    self, BudgetViolation, DAY_SNAPSHOT_MAX_BYTES, HISTORY_FILE_MAX_BYTES,
};
use celebtir_core::{RpcEndpoint, to_canonical_json};

const KEYED: &str = "https://mainnet.helius-rpc.com/?api-key=0123456789abcdef";

#[test]
fn snap_5_rpc_endpoint_serializes_as_host_only() {
    #[derive(serde::Serialize)]
    struct Run {
        rpc: RpcEndpoint,
    }
    let out = to_canonical_json(&Run {
        rpc: RpcEndpoint::new(KEYED),
    })
    .unwrap();
    assert!(out.contains("\"mainnet.helius-rpc.com\""), "{out}");
    assert!(!out.contains("0123456789abcdef"), "{out}");
}

#[test]
fn snap_5_rpc_endpoint_debug_and_display_hide_the_key() {
    let rpc = RpcEndpoint::new("https://example.quiknode.pro/SECRETTOKEN/");
    assert_eq!(rpc.to_string(), "example.quiknode.pro");
    assert!(!format!("{rpc:?}").contains("SECRETTOKEN"));
    assert_eq!(rpc.url(), "https://example.quiknode.pro/SECRETTOKEN/");
}

#[test]
fn snap_5_unparseable_rpc_url_never_echoes_input() {
    let rpc = RpcEndpoint::new("not a url with key=SECRET");
    assert!(!rpc.to_string().contains("SECRET"));
}

#[test]
fn snap_5_secret_scan_flags_keys_in_text() {
    assert!(budget::find_secrets("{\"rpc\":\"https://x/?api-key=abc\"}").is_some());
    assert!(budget::find_secrets("{\"rpc\":\"mainnet.helius-rpc.com\"}").is_none());
}

#[test]
fn snap_6_day_snapshot_over_2_mb_is_a_violation() {
    let files = vec![
        (
            PathBuf::from("data/snapshots/2026-11-01/facts.json"),
            DAY_SNAPSHOT_MAX_BYTES - 10,
        ),
        (PathBuf::from("data/snapshots/2026-11-01/scores.json"), 11),
        (PathBuf::from("data/snapshots/2026-11-02/facts.json"), 100),
    ];
    let v = budget::check(&files);
    assert_eq!(v.len(), 1);
    assert!(matches!(&v[0], BudgetViolation::DaySnapshot { day, .. } if day == "2026-11-01"));
}

#[test]
fn snap_6_history_file_over_500_kb_is_a_violation() {
    let files = vec![
        (
            PathBuf::from("data/history/kamino-lend.json"),
            HISTORY_FILE_MAX_BYTES + 1,
        ),
        (
            PathBuf::from("data/history/drift.json"),
            HISTORY_FILE_MAX_BYTES,
        ),
    ];
    let v = budget::check(&files);
    assert_eq!(v.len(), 1);
    assert!(matches!(&v[0], BudgetViolation::HistoryFile { .. }));
}

/// The real `data/` directory: budgets hold and no secrets leak. Runs in CI.
#[test]
fn snap_5_snap_6_repository_data_dir_passes() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let mut files = Vec::new();
    collect(&data, &data, &mut files);
    let sized: Vec<_> = files
        .iter()
        .map(|(rel, abs)| (rel.clone(), std::fs::metadata(abs).unwrap().len()))
        .collect();
    let violations = budget::check(&sized);
    assert!(violations.is_empty(), "{violations:#?}");
    for (rel, abs) in &files {
        let text = std::fs::read_to_string(abs).unwrap_or_default();
        assert!(
            budget::find_secrets(&text).is_none(),
            "possible secret in {}",
            rel.display()
        );
        if let Ok(url) = std::env::var("CELEBTIR_RPC_URL") {
            assert!(
                url.is_empty() || !text.contains(&url),
                "RPC URL in {}",
                rel.display()
            );
        }
    }
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            let rel = Path::new("data").join(path.strip_prefix(root).unwrap());
            out.push((rel, path));
        }
    }
}
