//! `celebtir summarize <snapshot-dir>`: one line for the daily commit message
//! (OPS-5) and the projected monthly RPC usage (OPS-6).

use std::path::Path;

use celebtir_core::snapshot::RunFile;
use serde::Serialize;

use crate::{CliError, Format};

/// Helius free plan: 1M credits a month (PRD, Data sources).
pub const RPC_MONTHLY_FREE_CREDITS: u64 = 1_000_000;
/// Open an issue above this share of a free tier (OPS-6).
pub const USAGE_WARN_PCT: u64 = 70;

#[derive(Debug, Serialize)]
pub struct Summary {
    pub date: String,
    pub protocols: usize,
    pub failures: usize,
    pub events: usize,
    pub critical_events: usize,
    pub rpc_calls: u64,
    /// Today's calls × 30, as a share of the free tier.
    pub rpc_projected_monthly_pct: u64,
    pub usage_warning: bool,
    pub line: String,
}

pub fn summary(dir: &Path) -> Result<Summary, CliError> {
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
    let run: RunFile = read("run.json")
        .and_then(|t| serde_json::from_str(&t).ok())
        .ok_or_else(|| {
            CliError::Validation(format!("{} has no readable run.json", dir.display()))
        })?;
    let events: Vec<serde_json::Value> = read("events.json")
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v["events"].as_array().cloned())
        .unwrap_or_default();
    let critical = events
        .iter()
        .filter(|e| e["severity"] == "critical")
        .count();
    let calls: u64 = run.calls.values().sum();
    let pct = calls * 30 * 100 / RPC_MONTHLY_FREE_CREDITS;
    let date = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    let line = format!(
        "data: {date}: {} protocols, {} events ({critical} critical), {} failures, {calls} RPC calls",
        run.protocols.len(),
        events.len(),
        run.failures.len()
    );
    Ok(Summary {
        date,
        protocols: run.protocols.len(),
        failures: run.failures.len(),
        events: events.len(),
        critical_events: critical,
        rpc_calls: calls,
        rpc_projected_monthly_pct: pct,
        usage_warning: pct > USAGE_WARN_PCT,
        line,
    })
}

pub fn run(dir: &Path, format: Format) -> Result<(), CliError> {
    let s = summary(dir)?;
    match format {
        Format::Json => println!(
            "{}",
            serde_json::to_string(&s).map_err(|e| CliError::Validation(e.to_string()))?
        ),
        Format::Text => println!("{}", s.line),
    }
    Ok(())
}
