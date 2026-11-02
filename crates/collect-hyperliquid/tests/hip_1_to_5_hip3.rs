//! HIP-1 to HIP-5 (and HREG-4) on the captured `perpDexs` and
//! `delegatorSummary` responses (fixtures/hyperliquid/info/).

use std::collections::BTreeMap;
use std::path::Path;

use celebtir_core::snapshot::{Hip3DexFacts, Hip3Snapshot, Hip3Stake};
use celebtir_core::{Address, Chain, to_canonical_json};
use celebtir_registry::Hip3File;
use collect_hyperliquid::fixtures::{InfoFixture, InfoReplay};
use collect_hyperliquid::hip3;
use collect_solana::rpc::{Transport, TransportError};
use serde_json::{Value, json};

const XYZ_DEPLOYER: &str = "0x88806a71D74ad0a510b350545C9aE490912F0888";

fn info(name: &str) -> InfoFixture {
    InfoFixture::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/hyperliquid/info/{name}.json")),
    )
    .unwrap()
}

fn registry() -> Hip3File {
    serde_norway::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registry/hyperliquid/hip3.yaml"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn dexs(out: &hip3::Hip3Output) -> &BTreeMap<String, Hip3DexFacts> {
    match &out.snapshot {
        Hip3Snapshot::Listed { dexs } => dexs,
        other => panic!("{other:?}"),
    }
}

async fn run() -> hip3::Hip3Output {
    let replay = InfoReplay(vec![info("perp-dexs"), info("delegator-summary-xyz")]);
    hip3::collect(&replay, info("perp-dexs").anchor_block, &registry()).await
}

#[tokio::test]
async fn hip_1_every_non_null_entry_becomes_a_dex() {
    let out = run().await;
    let raw = info("perp-dexs").response;
    let n = raw
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| !d.is_null())
        .count();
    let d = dexs(&out);
    assert_eq!(d.len(), n);
    let xyz = &d["xyz"];
    assert_eq!(xyz.full_name, "XYZ");
    assert_eq!(xyz.deployer.value().as_str(), XYZ_DEPLOYER);
    assert_eq!(xyz.oracle_updater.value(), &None);
    assert!(xyz.sub_deployers.value().contains_key("setOracle"));
    assert!(xyz.oi_caps.value().contains_key("xyz:AAPL"));
    // A delegated oracle updater is kept.
    assert!(d["flx"].oracle_updater.value().is_some());
}

#[tokio::test]
async fn hip_2_raw_response_is_kept_and_deterministic() {
    let a = run().await;
    let b = run().await;
    let raw = a.raw.clone().unwrap();
    assert_eq!(raw, info("perp-dexs").response);
    assert_eq!(
        to_canonical_json(&a.snapshot).unwrap(),
        to_canonical_json(&b.snapshot).unwrap()
    );
    assert_eq!(
        to_canonical_json(&raw).unwrap(),
        to_canonical_json(&b.raw.unwrap()).unwrap()
    );
}

#[tokio::test]
async fn hip_3_every_fact_has_hypercore_evidence_at_the_anchor() {
    let out = run().await;
    let anchor = info("perp-dexs").anchor_block;
    for (name, d) in dexs(&out) {
        for e in [
            d.deployer.evidence(),
            d.oracle_updater.evidence(),
            d.sub_deployers.evidence(),
            d.fee_recipient.evidence(),
            d.oi_caps.evidence(),
            d.funding_multipliers.evidence(),
            d.stake.evidence(),
        ] {
            assert!(!e.is_empty(), "{name}");
            assert!(
                e.iter()
                    .all(|e| e.chain == Chain::HyperCore && e.slot == anchor)
            );
            assert!(
                e[0].explorer_url
                    .as_str()
                    .starts_with("https://app.hyperliquid.xyz/explorer/address/")
            );
        }
    }
}

#[tokio::test]
async fn hip_4_stake_is_compared_with_the_requirement() {
    let out = run().await;
    let d = dexs(&out);
    match d["xyz"].stake.value() {
        Hip3Stake::Read {
            stake_below_required,
            unstaking,
            required,
            ..
        } => {
            assert!(!stake_below_required, "501,610 is above 500,000");
            assert!(!unstaking);
            assert_eq!(required, "500000");
        }
        other => panic!("{other:?}"),
    }
    // Deployers without a captured summary stay Unavailable, never zero.
    assert!(matches!(
        d["flx"].stake.value(),
        Hip3Stake::Unavailable { .. }
    ));

    let mut low = info("delegator-summary-xyz").response;
    low["delegated"] = Value::String("1000.0".into());
    let parsed = hip3::parse(
        &info("perp-dexs").response,
        1,
        &BTreeMap::from([(Address::new(XYZ_DEPLOYER), Ok(low))]),
        &registry(),
    )
    .unwrap();
    assert!(matches!(
        parsed["xyz"].stake.value(),
        Hip3Stake::Read {
            stake_below_required: true,
            ..
        }
    ));
}

struct Down;

impl Transport for Down {
    async fn send(&self, _: Value) -> Result<Value, TransportError> {
        Err(TransportError::Status(500))
    }
}

#[tokio::test]
async fn hip_5_outage_marks_hip3_unavailable() {
    let out = hip3::collect(&Down, 1, &registry()).await;
    assert!(matches!(out.snapshot, Hip3Snapshot::Unavailable { .. }));
    assert!(out.raw.is_none());
    let garbage = InfoReplay(vec![InfoFixture {
        host: "synthetic".into(),
        anchor_block: 1,
        request: json!({"type": "perpDexs"}),
        response: json!({"oops": true}),
        note: "synthetic".into(),
    }]);
    let out = hip3::collect(&garbage, 1, &registry()).await;
    assert!(matches!(out.snapshot, Hip3Snapshot::Unavailable { .. }));
}

#[tokio::test]
async fn hreg_4_unknown_hip3_yaml_name_is_a_warning() {
    let mut r = registry();
    r.dexs.insert("gone".into(), Default::default());
    let replay = InfoReplay(vec![info("perp-dexs")]);
    let out = hip3::collect(&replay, 1, &r).await;
    assert!(matches!(out.snapshot, Hip3Snapshot::Listed { .. }));
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
}
