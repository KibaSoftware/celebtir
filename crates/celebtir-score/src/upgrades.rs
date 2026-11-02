//! Upgrade metrics (VAL-5) from ProgramData histories, as of a snapshot date.
//! Pure: the date comes from the caller, not a clock.

use celebtir_core::date::days;
use celebtir_core::snapshot::{ProgramHistory, UpgradeKind};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UpgradeMetrics {
    /// Upgrades across the protocol's core programs (deploys excluded).
    pub upgrades: u64,
    pub last_upgrade_date: Option<String>,
    pub last_upgrade_slot: Option<u64>,
    /// Mean days between consecutive dated upgrades, one decimal.
    pub average_interval_days: Option<f64>,
    /// Dated upgrades in the 30 days up to and including `as_of`.
    pub upgrades_last_30_days: u64,
    /// True when every history was read in full; otherwise the counts are a
    /// lower bound.
    pub complete: bool,
}

pub fn upgrade_metrics(histories: &[&ProgramHistory], as_of: &str) -> UpgradeMetrics {
    let mut upgrades: Vec<(u64, Option<&str>)> = histories
        .iter()
        .flat_map(|h| h.events.iter())
        .filter(|e| e.kind == UpgradeKind::Upgrade)
        .map(|e| (e.slot, e.date.as_deref()))
        .collect();
    upgrades.sort();

    let dated: Vec<i64> = upgrades
        .iter()
        .filter_map(|(_, d)| d.and_then(days))
        .collect();
    let average_interval_days = (dated.len() >= 2).then(|| {
        let span = (dated[dated.len() - 1] - dated[0]) as f64;
        (span / (dated.len() - 1) as f64 * 10.0).round() / 10.0
    });
    let upgrades_last_30_days = match days(as_of) {
        Some(today) => dated
            .iter()
            .filter(|d| today - **d < 30 && **d <= today)
            .count() as u64,
        None => 0,
    };
    let last = upgrades.last();
    UpgradeMetrics {
        upgrades: upgrades.len() as u64,
        last_upgrade_date: last.and_then(|(_, d)| d.map(str::to_string)),
        last_upgrade_slot: last.map(|(s, _)| *s),
        average_interval_days,
        upgrades_last_30_days,
        complete: !histories.is_empty() && histories.iter().all(|h| h.complete),
    }
}
