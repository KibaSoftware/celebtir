//! HUI-1: summary days carry each chain's protocols, TVL by control and
//! Stages, so the site's charts follow the chain picker. F5: an unassessed
//! chain carries its protocols and TVL, but no Stages or value classes.

mod common;

use celebtir::site_data::by_chain;
use celebtir_core::Chain;
use celebtir_core::snapshot::{Metric, ProtocolMetrics};
use celebtir_score::{Methodology, score};
use common::sample;

fn tvl(value: u64) -> ProtocolMetrics {
    ProtocolMetrics {
        defillama_slug: Some("x".into()),
        tvl: Metric::Usd { value },
        asset_groups: Default::default(),
        fees: Default::default(),
        volume: Default::default(),
    }
}

#[test]
fn hui_1_summary_day_splits_protocols_and_tvl_by_chain() {
    let mut f = sample();
    let ids: Vec<String> = f.protocols.keys().cloned().collect();
    // One sample protocol moves to HyperEVM; each has its own TVL.
    f.protocols.get_mut(&ids[0]).unwrap().chain = Chain::HyperEvm;
    for (i, id) in ids.iter().enumerate() {
        f.metrics.insert(id.clone(), tvl(1_000 * (i as u64 + 1)));
    }
    let s = score(&f, &Methodology::shipped());
    let split = by_chain(&f, &s);

    let hl = &split["hyperliquid"];
    let sol = &split["solana"];
    assert_eq!(hl.protocols, 1);
    assert_eq!(sol.protocols, ids.len() as u64 - 1);
    assert_eq!(hl.total_tvl_usd, 1_000);
    // v0.3 rates Solana only: HyperEVM is listed with TVL, never staged.
    assert_eq!(hl.assessment, celebtir::coverage::Assessment::Unassessed);
    assert!(hl.stages.is_none() && hl.value_by_control.is_none());
    assert_eq!(sol.total_tvl_usd, s.value_by_control.all.total);
    assert_eq!(
        sol.stages.as_ref().unwrap().values().sum::<u64>(),
        s.protocols.len() as u64
    );
}

#[test]
fn hui_1_a_v02_day_withholds_hyperevm_stages_under_the_shipped_policy() {
    let mut f = sample();
    let ids: Vec<String> = f.protocols.keys().cloned().collect();
    f.protocols.get_mut(&ids[0]).unwrap().chain = Chain::HyperEvm;
    // A stored v0.2 day scored HyperEVM; publication withholds it (F5).
    let s = score(&f, &Methodology::bundled("0.2").unwrap());
    assert!(s.protocols.contains_key(&ids[0]));
    let split = by_chain(&f, &s);
    assert!(split["hyperliquid"].stages.is_none());
    assert_eq!(
        split["solana"]
            .stages
            .as_ref()
            .unwrap()
            .values()
            .sum::<u64>(),
        ids.len() as u64 - 1
    );
}

#[test]
fn hui_1_a_solana_only_day_has_an_empty_hyperliquid_share() {
    let f = sample();
    let s = score(&f, &Methodology::shipped());
    let split = by_chain(&f, &s);
    assert_eq!(split["hyperliquid"].protocols, 0);
    assert_eq!(split["hyperliquid"].total_tvl_usd, 0);
    assert_eq!(split["solana"].protocols, s.protocols.len() as u64);
}
