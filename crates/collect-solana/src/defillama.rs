//! DefiLlama context metrics (VAL-1, VAL-7): free, keyless, and never used in
//! a rating. Any failure becomes `Metric::Unavailable`; it never fails a run.

use celebtir_core::snapshot::{Activity, AssetGroups, DailyUsd, Metric, ProtocolMetrics};
use serde_json::Value;

use crate::verify::{HttpGet, get_with_retry};

pub const DEFILLAMA_API: &str = "https://api.llama.fi";

/// A non-negative USD amount, rounded to whole dollars.
fn usd(v: &Value) -> Option<u64> {
    v.as_f64()
        .filter(|x| x.is_finite() && *x >= 0.0)
        .map(|x| x.round() as u64)
}

fn unavailable(reason: impl Into<String>) -> Metric {
    Metric::Unavailable {
        reason: reason.into(),
    }
}

/// TVL, asset groups, fees and, when `has_volume` (DEXs), volume.
pub async fn protocol_metrics<H: HttpGet>(
    http: &H,
    slug: Option<&str>,
    has_volume: bool,
) -> ProtocolMetrics {
    let Some(slug) = slug else {
        let none = || Activity::Unavailable {
            reason: "no DefiLlama slug in the registry".into(),
        };
        return ProtocolMetrics {
            defillama_slug: None,
            tvl: unavailable("no DefiLlama slug in the registry"),
            asset_groups: AssetGroups::default(),
            fees: none(),
            volume: none(),
        };
    };
    let tvl = match get_with_retry(http, &format!("{DEFILLAMA_API}/tvl/{slug}")).await {
        Ok(v) => usd(&v).map_or_else(
            || unavailable(format!("DefiLlama returned a non-numeric TVL: {v}")),
            |value| Metric::Usd { value },
        ),
        Err(e) => unavailable(format!("DefiLlama TVL unavailable: {e}")),
    };
    let asset_groups = asset_groups(http, slug, &tvl).await;
    let fees = activity(http, slug, ActivityKind::Fees).await;
    let volume = if has_volume {
        activity(http, slug, ActivityKind::Volume).await
    } else {
        Activity::Unavailable {
            reason: "no volume metric for this category; fees only".into(),
        }
    };
    ProtocolMetrics {
        defillama_slug: Some(slug.to_string()),
        tvl,
        asset_groups,
        fees,
        volume,
    }
}

/// The four VAL-3 asset groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetGroup {
    Stablecoins,
    Sol,
    Btc,
    Other,
}

/// Groups a DefiLlama token symbol. BTC first (`*BTC*`), then dollar and euro
/// stablecoins (`*USD*`, a short list of others), then SOL and liquid staking
/// tokens (`SOL`, `*SOL`, `INF`). Everything else is "other".
pub fn classify(symbol: &str) -> AssetGroup {
    const STABLES: &[&str] = &[
        "USDT", "DAI", "UXD", "CASH", "EURC", "EURCV", "AUSD", "FRAX",
    ];
    const SOL_EXTRA: &[&str] = &["INF"];
    let s = symbol.to_ascii_uppercase();
    if s.contains("BTC") {
        AssetGroup::Btc
    } else if s.contains("USD") || STABLES.contains(&s.as_str()) {
        AssetGroup::Stablecoins
    } else if s.ends_with("SOL") || SOL_EXTRA.contains(&s.as_str()) {
        AssetGroup::Sol
    } else {
        AssetGroup::Other
    }
}

/// How far the token breakdown may stray from the TVL before the split is
/// considered unreliable.
const MAX_BREAKDOWN_GAP: f64 = 0.05;

/// Splits the protocol's Solana TVL by asset group from the newest entry of
/// DefiLlama's per-token breakdown. "Other" takes the remainder, so the four
/// groups always sum to the TVL.
pub async fn asset_groups<H: HttpGet>(http: &H, slug: &str, tvl: &Metric) -> AssetGroups {
    let unavailable = |reason: String| AssetGroups::Unavailable { reason };
    let Some(tvl) = tvl.usd() else {
        return unavailable("TVL unavailable, so it cannot be split".into());
    };
    let v = match get_with_retry(http, &format!("{DEFILLAMA_API}/protocol/{slug}")).await {
        Ok(v) => v,
        Err(e) => return unavailable(format!("DefiLlama breakdown unavailable: {e}")),
    };
    let Some(tokens) = v["chainTvls"]["Solana"]["tokensInUsd"]
        .as_array()
        .and_then(|a| a.last())
        .and_then(|last| last["tokens"].as_object())
    else {
        return unavailable("DefiLlama has no per-token breakdown for this protocol".into());
    };

    let mut sums = [0f64; 3];
    let mut total = 0f64;
    for (symbol, usd) in tokens {
        let Some(x) = usd.as_f64().filter(|x| x.is_finite()) else {
            continue;
        };
        total += x;
        match classify(symbol) {
            AssetGroup::Stablecoins => sums[0] += x,
            AssetGroup::Sol => sums[1] += x,
            AssetGroup::Btc => sums[2] += x,
            AssetGroup::Other => {}
        }
    }
    let gap = (total - tvl as f64).abs() / (tvl as f64).max(1.0);
    if gap > MAX_BREAKDOWN_GAP {
        return unavailable(format!(
            "token breakdown ({total:.0} USD) differs from TVL ({tvl} USD) by {:.1}%",
            gap * 100.0
        ));
    }
    let names = ["stablecoins", "SOL", "BTC"];
    let mut rounded = [0u64; 3];
    for (i, s) in sums.iter().enumerate() {
        if *s < 0.0 {
            return unavailable(format!(
                "{} net below zero after borrowed amounts",
                names[i]
            ));
        }
        rounded[i] = s.round() as u64;
    }
    let known: u64 = rounded.iter().sum();
    let Some(other) = tvl.checked_sub(known) else {
        return unavailable("asset groups exceed the TVL".into());
    };
    AssetGroups::Usd {
        stablecoins: rounded[0],
        sol: rounded[1],
        btc: rounded[2],
        other,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    /// `/summary/fees/<slug>?dataType=dailyFees`: every category.
    Fees,
    /// `/summary/dexs/<slug>?dataType=dailyVolume`: DEXs. Perps volume sits
    /// behind DefiLlama's paid plan, so perps show fees only.
    Volume,
}

/// Up to 30 complete days of fees or volume, with week-over-week change.
pub async fn activity<H: HttpGet>(http: &H, slug: &str, kind: ActivityKind) -> Activity {
    let (path, data_type, what) = match kind {
        ActivityKind::Fees => ("fees", "dailyFees", "fees"),
        ActivityKind::Volume => ("dexs", "dailyVolume", "volume"),
    };
    let unavailable = |reason: String| Activity::Unavailable { reason };
    let url = format!("{DEFILLAMA_API}/summary/{path}/{slug}?dataType={data_type}");
    let v = match get_with_retry(http, &url).await {
        Ok(v) => v,
        Err(e) => return unavailable(format!("DefiLlama {what} unavailable: {e}")),
    };
    let points: Vec<(i64, u64)> = v["totalDataChart"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| Some((p[0].as_i64()?, usd(&p[1])?)))
        .collect();
    // The newest point is usually today's partial day: end at the point that
    // matches `total24h`, or else drop the newest point.
    let end = match usd(&v["total24h"]) {
        Some(t) => points.iter().rposition(|(_, x)| *x == t),
        None => None,
    }
    .map(|i| i + 1)
    .unwrap_or(points.len().saturating_sub(1));
    let complete = &points[..end];
    if complete.is_empty() {
        return unavailable(format!("DefiLlama has no daily {what} for {slug}"));
    }
    let start = complete.len().saturating_sub(30);
    let days: Vec<DailyUsd> = complete[start..]
        .iter()
        .map(|(ts, x)| DailyUsd {
            date: celebtir_core::date::ymd(*ts),
            usd: *x,
        })
        .collect();
    // D5: calendar windows over the published days; a missing or near-zero
    // day makes the change unavailable rather than distorted.
    let (change_7d_bp, gap_days) = celebtir_core::activity::change_7d(&days);
    Activity::Usd {
        latest: complete.last().map(|(_, x)| *x).unwrap_or(0),
        days,
        change_7d_bp,
        gap_days,
    }
}

pub const DEFILLAMA_COINS: &str = "https://coins.llama.fi";

/// Current prices for many mints in one call (TOK-9). Context only.
pub async fn token_prices<H: HttpGet>(
    http: &H,
    mints: &[celebtir_core::Address],
) -> std::collections::BTreeMap<celebtir_core::Address, celebtir_core::snapshot::TokenMetrics> {
    use celebtir_core::snapshot::TokenMetrics;
    if mints.is_empty() {
        return Default::default();
    }
    let ids: Vec<String> = mints.iter().map(|m| format!("solana:{m}")).collect();
    let url = format!("{DEFILLAMA_COINS}/prices/current/{}", ids.join(","));
    let body = get_with_retry(http, &url).await;
    mints
        .iter()
        .map(|m| {
            let metric = match &body {
                Err(e) => TokenMetrics::Unavailable {
                    reason: format!("DefiLlama prices unavailable: {e}"),
                },
                Ok(v) => match v["coins"][format!("solana:{m}")]["price"]
                    .as_f64()
                    .filter(|p| p.is_finite() && *p >= 0.0)
                {
                    Some(p) => TokenMetrics::Price {
                        usd_micros: (p * 1_000_000.0).round() as u64,
                    },
                    None => TokenMetrics::Unavailable {
                        reason: "DefiLlama has no price for this mint".into(),
                    },
                },
            };
            (m.clone(), metric)
        })
        .collect()
}
