//! HSCO-4: HIP-3 DEXs are listed in `scores.json` without ratings or Stage.
//! HSCO-5: a Solana-only facts file scores byte-identically to the committed
//! golden output, so wave 2 changed nothing for Solana.

use std::collections::BTreeMap;
use std::path::Path;

use celebtir_core::snapshot::{FactsFile, Hip3DexFacts, Hip3Snapshot, Hip3Stake};
use celebtir_core::{Address, Chain, Evidence, Fact, to_canonical_json};
use celebtir_score::{HIP3_UNRATED, Methodology, score};

fn golden(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/golden/expected/{name}")),
    )
    .unwrap()
}

fn dex() -> Hip3DexFacts {
    let a = Address::new("0x88806a71D74ad0a510b350545C9aE490912F0888");
    let ev = Evidence::new(
        Chain::HyperCore,
        a.clone(),
        49_145_767,
        "info.perpDexs[xyz]",
    );
    Hip3DexFacts {
        full_name: "XYZ".into(),
        deployer: Fact::single(a, ev.clone()),
        oracle_updater: Fact::single(None, ev.clone()),
        sub_deployers: Fact::single(BTreeMap::new(), ev.clone()),
        fee_recipient: Fact::single(None, ev.clone()),
        oi_caps: Fact::single(BTreeMap::new(), ev.clone()),
        funding_multipliers: Fact::single(BTreeMap::new(), ev.clone()),
        stake: Fact::single(
            Hip3Stake::Unavailable {
                reason: "test".into(),
            },
            ev,
        ),
    }
}

#[test]
fn hsco_5_solana_only_facts_score_byte_identically_to_the_golden() {
    let facts: FactsFile = serde_json::from_str(&golden("facts.json")).unwrap();
    assert!(facts.hip3.is_none());
    let legacy = Methodology::from_toml(include_str!("../../../methodology/v0.1.toml")).unwrap();
    let scores = score(&facts, &legacy);
    let ours = to_canonical_json(&scores).unwrap();
    let expected = golden("scores.json");
    assert_eq!(ours.trim_end(), expected.trim_end());
}

#[test]
fn hsco_4_hip3_dexs_are_listed_unrated_and_change_nothing_else() {
    let mut facts: FactsFile = serde_json::from_str(&golden("facts.json")).unwrap();
    let before = score(&facts, &Methodology::bundled("0.2").unwrap());
    facts.hip3 = Some(Hip3Snapshot::Listed {
        dexs: BTreeMap::from([("xyz".to_string(), dex())]),
    });
    let after = score(&facts, &Methodology::bundled("0.2").unwrap());
    assert_eq!(after.protocols, before.protocols);
    assert_eq!(after.tokens, before.tokens);
    assert_eq!(after.value_by_control, before.value_by_control);
    let x = &after.hip3_dexs["xyz"];
    assert!(!x.rated);
    assert_eq!(x.reason, HIP3_UNRATED);
}

#[test]
fn hsco_4_unavailable_hip3_lists_nothing() {
    let mut facts: FactsFile = serde_json::from_str(&golden("facts.json")).unwrap();
    facts.hip3 = Some(Hip3Snapshot::Unavailable {
        reason: "down".into(),
    });
    assert!(
        score(&facts, &Methodology::bundled("0.2").unwrap())
            .hip3_dexs
            .is_empty()
    );
}
