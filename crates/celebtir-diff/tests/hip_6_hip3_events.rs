//! HIP-6: a HIP-3 DEX that appears or disappears, or whose deployer, oracle
//! control or caps change, raises an event; an unavailable day does not.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{FactsFile, Hip3DexFacts, Hip3Snapshot, Hip3Stake};
use celebtir_core::{Address, Chain, Evidence, Fact};
use celebtir_diff::{EventKind, Severity, Snapshot, diff};
use celebtir_score::{Methodology, score};

const DEPLOYER: &str = "0x88806a71D74ad0a510b350545C9aE490912F0888";
const UPDATER: &str = "0x94757f8DCb4bf73b850195660e959d1105CfedD5";

fn dex(updater: Option<&str>, cap: &str) -> Hip3DexFacts {
    let d = Address::new(DEPLOYER);
    let ev = Evidence::new(Chain::HyperCore, d.clone(), 1, "info.perpDexs[xyz]");
    Hip3DexFacts {
        full_name: "XYZ".into(),
        deployer: Fact::single(d, ev.clone()),
        oracle_updater: Fact::single(updater.map(Address::new), ev.clone()),
        sub_deployers: Fact::single(BTreeMap::new(), ev.clone()),
        fee_recipient: Fact::single(None, ev.clone()),
        oi_caps: Fact::single(
            BTreeMap::from([("xyz:AAPL".to_string(), cap.to_string())]),
            ev.clone(),
        ),
        funding_multipliers: Fact::single(BTreeMap::new(), ev.clone()),
        stake: Fact::single(
            Hip3Stake::Unavailable {
                reason: "test".into(),
            },
            ev,
        ),
    }
}

fn facts(hip3: Option<Hip3Snapshot>) -> FactsFile {
    let mut f = FactsFile::new(BTreeMap::new());
    f.hip3 = hip3;
    f
}

fn listed(dexs: Vec<(&str, Hip3DexFacts)>) -> Option<Hip3Snapshot> {
    Some(Hip3Snapshot::Listed {
        dexs: dexs.into_iter().map(|(n, d)| (n.to_string(), d)).collect(),
    })
}

fn events(old: &FactsFile, new: &FactsFile) -> Vec<celebtir_diff::Event> {
    let m = Methodology::shipped();
    let (so, sn) = (score(old, &m), score(new, &m));
    diff(
        &Snapshot {
            date: "2026-10-20",
            facts: old,
            scores: &so,
        },
        &Snapshot {
            date: "2026-10-21",
            facts: new,
            scores: &sn,
        },
    )
}

#[test]
fn hip_6_changed_oracle_updater_emits_one_event_with_before_and_after() {
    let old = facts(listed(vec![("xyz", dex(None, "100"))]));
    let new = facts(listed(vec![("xyz", dex(Some(UPDATER), "100"))]));
    let e = events(&old, &new);
    assert_eq!(e.len(), 1, "{e:#?}");
    assert_eq!(e[0].kind, EventKind::Hip3Changed);
    assert_eq!(e[0].severity, Severity::Warning);
    assert_eq!(e[0].subject, "hip3:xyz.oracle_updater");
    assert!(e[0].before.is_null());
    assert_eq!(e[0].after, serde_json::json!(UPDATER));
    assert!(!e[0].evidence.is_empty());
}

#[test]
fn hip_6_new_and_removed_dexs_and_cap_changes_are_events() {
    let old = facts(listed(vec![
        ("xyz", dex(None, "100")),
        ("old", dex(None, "1")),
    ]));
    let new = facts(listed(vec![
        ("xyz", dex(None, "200")),
        ("new", dex(None, "1")),
    ]));
    let mut subjects: Vec<String> = events(&old, &new).into_iter().map(|e| e.subject).collect();
    subjects.sort();
    assert_eq!(subjects, ["hip3:new", "hip3:old", "hip3:xyz.oi_caps"]);
}

#[test]
fn hip_6_an_unavailable_day_is_not_a_change() {
    let old = facts(listed(vec![("xyz", dex(None, "100"))]));
    let new = facts(Some(Hip3Snapshot::Unavailable {
        reason: "down".into(),
    }));
    assert!(events(&old, &new).is_empty());
    assert!(events(&facts(None), &old).is_empty());
}
