//! VAL-2: TVL summed per controller class for the day, for all protocols and
//! per category; the five classes sum to the total TVL.

mod common;

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{AssetGroups, FactsFile, Metric, ProgramRole, ProtocolMetrics};
use celebtir_score::value::{ControlClass, control_class, value_by_control};
use common::*;

const A: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const B: &str = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";

fn metrics(tvl: Option<u64>) -> ProtocolMetrics {
    ProtocolMetrics {
        defillama_slug: Some("x".into()),
        tvl: match tvl {
            Some(value) => Metric::Usd { value },
            None => Metric::Unavailable {
                reason: "down".into(),
            },
        },
        asset_groups: AssetGroups::default(),
        fees: Default::default(),
        volume: Default::default(),
    }
}

#[test]
fn val_2_controllers_map_to_the_five_classes() {
    assert_eq!(
        control_class(&Controller::Immutable),
        ControlClass::Immutable
    );
    assert_eq!(control_class(&single_key()), ControlClass::SingleKey);
    assert_eq!(
        control_class(&multisig(5, 8, DAY)),
        ControlClass::MultisigWithDelay
    );
    assert_eq!(
        control_class(&multisig(5, 8, 0)),
        ControlClass::MultisigWithoutDelay
    );
    // ADR 005: a single-key config authority makes the multisig one key.
    assert_eq!(
        control_class(&with_config_authority(multisig(5, 8, DAY), single_key())),
        ControlClass::SingleKey
    );
    // A multisig config authority with no time lock removes the delay.
    assert_eq!(
        control_class(&with_config_authority(
            multisig(5, 8, DAY),
            multisig(3, 5, 0)
        )),
        ControlClass::MultisigWithoutDelay
    );
    for c in [
        Controller::Unrecognized {
            key: addr("u"),
            owner_program: None,
        },
        Controller::Unavailable { reason: "x".into() },
        Controller::CannotAct {
            address: addr("m"),
            threshold: 3,
            voters: 1,
        },
    ] {
        assert_eq!(control_class(&c), ControlClass::Unrecognized, "{c:?}");
    }
}

fn facts() -> FactsFile {
    let mut lend = protocol(vec![
        (A, ProgramRole::Core, multisig(5, 8, DAY)),
        (B, ProgramRole::Core, single_key()),
    ]);
    lend.category = Some("lending".into());
    let mut dex = protocol(vec![
        (A, ProgramRole::Core, Controller::Immutable),
        (B, ProgramRole::Periphery, single_key()),
    ]);
    dex.category = Some("dex".into());
    let mut perps = protocol(vec![(A, ProgramRole::Core, multisig(3, 5, 0))]);
    perps.category = Some("perps".into());
    let mut dark = protocol(vec![(A, ProgramRole::Core, Controller::Immutable)]);
    dark.category = Some("dex".into());
    let mut f = FactsFile::new(BTreeMap::from([
        ("lend".to_string(), lend),
        ("dex".to_string(), dex),
        ("perps".to_string(), perps),
        ("dark".to_string(), dark),
    ]));
    f.metrics.insert("lend".into(), metrics(Some(1_000)));
    f.metrics.insert("dex".into(), metrics(Some(300)));
    f.metrics.insert("perps".into(), metrics(Some(50)));
    f.metrics.insert("dark".into(), metrics(None));
    f
}

#[test]
fn val_2_weakest_core_program_decides_and_classes_sum_to_total_tvl() {
    let v = value_by_control(&facts());
    let all = &v.all;
    assert_eq!(
        all.classes[&ControlClass::SingleKey],
        1_000,
        "lend's weakest core program is a single key"
    );
    assert_eq!(
        all.classes[&ControlClass::Immutable],
        300,
        "dex's single-key periphery program is not counted"
    );
    assert_eq!(all.classes[&ControlClass::MultisigWithoutDelay], 50);
    assert_eq!(all.total, 1_350);
    assert_eq!(all.classes.values().sum::<u64>(), all.total);
    assert_eq!(
        all.classes.len(),
        5,
        "every class is present, zeros included"
    );
    assert_eq!(
        v.excluded,
        vec!["dark".to_string()],
        "protocols without TVL are listed, not guessed"
    );
}

#[test]
fn val_2_per_category_rows_also_sum() {
    let v = value_by_control(&facts());
    assert_eq!(v.by_category["lending"].total, 1_000);
    assert_eq!(v.by_category["dex"].total, 300);
    for row in v.by_category.values() {
        assert_eq!(row.classes.values().sum::<u64>(), row.total);
    }
}

#[test]
fn val_2_is_part_of_the_scores_file() {
    let s = celebtir_score::score(
        &facts(),
        &celebtir_score::Methodology::bundled("0.2").unwrap(),
    );
    assert_eq!(s.value_by_control.all.total, 1_350);
}

#[test]
fn val_2_each_protocol_score_carries_its_class_and_a_short_controller_summary() {
    let s = celebtir_score::score(
        &facts(),
        &celebtir_score::Methodology::bundled("0.2").unwrap(),
    );
    assert_eq!(s.protocols["lend"].control_class, ControlClass::SingleKey);
    assert_eq!(s.protocols["lend"].controller_summary, "Single key");
    assert_eq!(s.protocols["dex"].control_class, ControlClass::Immutable);
    assert_eq!(s.protocols["dex"].controller_summary, "Immutable");
    assert_eq!(
        s.protocols["perps"].controller_summary,
        "Squads 3 of 5 · no delay"
    );
}

#[test]
fn val_2_short_summaries_cover_every_controller() {
    use celebtir_score::factors::short_describe;
    assert_eq!(
        short_describe(&multisig(5, 10, DAY)),
        "Squads 5 of 10 · 24 h delay"
    );
    assert_eq!(
        short_describe(&with_config_authority(multisig(5, 8, DAY), single_key())),
        "Squads 5 of 8 · config key"
    );
    assert_eq!(
        short_describe(&Controller::Unrecognized {
            key: addr("u"),
            owner_program: None
        }),
        "Unrecognized"
    );
    assert_eq!(
        short_describe(&Controller::Unavailable { reason: "x".into() }),
        "Unreadable"
    );
    assert_eq!(
        short_describe(&Controller::CannotAct {
            address: addr("m"),
            threshold: 3,
            voters: 1
        }),
        "Multisig that cannot act"
    );
}
