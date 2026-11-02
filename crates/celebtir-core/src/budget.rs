//! Size budgets for published data (SNAP-6) and a scan for leaked secrets
//! (SNAP-5). Pure functions over paths and sizes; callers do the I/O.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// All files of one day's snapshot together.
pub const DAY_SNAPSHOT_MAX_BYTES: u64 = 2 * 1024 * 1024;
/// Each `data/history/<protocol>.json`.
pub const HISTORY_FILE_MAX_BYTES: u64 = 500 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetViolation {
    DaySnapshot { day: String, bytes: u64, max: u64 },
    HistoryFile { path: PathBuf, bytes: u64, max: u64 },
}

/// Checks `(path relative to the repo, size in bytes)` pairs against the budgets.
pub fn check(files: &[(PathBuf, u64)]) -> Vec<BudgetViolation> {
    let mut days: BTreeMap<String, u64> = BTreeMap::new();
    let mut out = Vec::new();
    for (path, bytes) in files {
        let parts = parts(path);
        match parts.as_slice() {
            ["data", "snapshots", day, ..] if parts.len() > 3 => {
                *days.entry((*day).to_string()).or_default() += bytes;
            }
            ["data", "history", _] if *bytes > HISTORY_FILE_MAX_BYTES => {
                out.push(BudgetViolation::HistoryFile {
                    path: path.clone(),
                    bytes: *bytes,
                    max: HISTORY_FILE_MAX_BYTES,
                });
            }
            _ => {}
        }
    }
    for (day, bytes) in days {
        if bytes > DAY_SNAPSHOT_MAX_BYTES {
            out.push(BudgetViolation::DaySnapshot {
                day,
                bytes,
                max: DAY_SNAPSHOT_MAX_BYTES,
            });
        }
    }
    out
}

fn parts(path: &Path) -> Vec<&str> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect()
}

const SECRET_MARKERS: &[&str] = &[
    "api-key=",
    "api_key=",
    "apikey=",
    "?key=",
    "&key=",
    "access_token=",
    "secret=",
    "authorization:",
];

/// Returns the first secret-looking marker found in `text`, if any.
pub fn find_secrets(text: &str) -> Option<&'static str> {
    let lower = text.to_ascii_lowercase();
    SECRET_MARKERS.iter().copied().find(|m| lower.contains(m))
}
