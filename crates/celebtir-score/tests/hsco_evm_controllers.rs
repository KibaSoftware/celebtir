//! HSCO-2, HSCO-3, HSCO-6: Safe and timelock controllers (wave 2, ADR 024)
//! score with the unchanged v0.1 rules.

mod common;

use celebtir_core::snapshot::ProgramRole;
use celebtir_core::{Controller, MultisigKind};
use celebtir_score::{Factor, Methodology, Rating, factors, score_protocol, sentences};
use common::*;

const CORE: &str = "0x1111111111111111111111111111111111111111";

fn m() -> Methodology {
    Methodology::bundled("0.2").unwrap()
}

fn safe(threshold: u16, voters: u16) -> Controller {
    Controller::Multisig {
        kind: MultisigKind::Safe,
        address: addr("0x2222222222222222222222222222222222222222"),
        threshold,
        voters,
        time_lock_secs: 0,
        config_authority: None,
        signers: vec![],
        nested_signers: Default::default(),
    }
}

fn timelock(min_delay_secs: u64, proposer: Controller, admin: Option<Controller>) -> Controller {
    Controller::Timelock {
        address: addr("0x3333333333333333333333333333333333333333"),
        min_delay_secs,
        proposer: Box::new(proposer),
        admin: admin.map(Box::new),
    }
}

#[test]
fn hsco_2_safe_5_of_8_behind_7_day_timelock_rates_low() {
    let c = fact(timelock(7 * u64::from(DAY), safe(5, 8), None));
    let uc = factors::upgrade_control(&c, &m());
    let ud = factors::upgrade_delay(&c, &m());
    assert_eq!(uc.rating, Rating::Low, "{}", uc.reason);
    assert_eq!(ud.rating, Rating::Low, "{}", ud.reason);
    assert!(uc.reason.contains("Safe"), "{}", uc.reason);
}

#[test]
fn hsco_2_safe_rates_on_threshold_like_squads() {
    for (t, v) in [(5, 8), (2, 4), (2, 2), (5, 9)] {
        let a = factors::upgrade_control(&fact(safe(t, v)), &m()).rating;
        let b = factors::upgrade_control(&fact(multisig(t, v, 0)), &m()).rating;
        assert_eq!(a, b, "{t} of {v}");
    }
}

#[test]
fn hsco_3_outside_admin_makes_delay_zero_and_rates_on_admin() {
    let c = fact(timelock(7 * u64::from(DAY), safe(5, 8), Some(single_key())));
    let ud = factors::upgrade_delay(&c, &m());
    let uc = factors::upgrade_control(&c, &m());
    assert_eq!(ud.rating, Rating::High, "{}", ud.reason);
    assert_eq!(uc.rating, Rating::High, "{}", uc.reason);
    assert!(uc.reason.contains("admin"), "{}", uc.reason);
}

#[test]
fn hsco_3_outside_admin_blocks_stage_1() {
    let p = protocol(vec![(
        CORE,
        ProgramRole::Core,
        timelock(7 * u64::from(DAY), safe(5, 8), Some(single_key())),
    )]);
    let s = score_protocol(&p, &m());
    assert_eq!(s.factors[&Factor::UpgradeDelay].rating, Rating::High);
}

#[test]
fn hsco_6_timelock_sentence_names_holder_and_passes_lint() {
    let p = protocol(vec![(
        CORE,
        ProgramRole::Core,
        timelock(2 * u64::from(DAY), single_key(), None),
    )]);
    let s = score_protocol(&p, &m());
    assert!(!s.risks.is_empty());
    let upgrade = s
        .risks
        .iter()
        .find(|r| r.factor == Factor::UpgradeControl)
        .expect("an upgrade sentence");
    assert!(
        upgrade.text.contains("the holder of key"),
        "{}",
        upgrade.text
    );
    for r in &s.risks {
        assert_eq!(sentences::banned_word(&r.text), None, "{}", r.text);
    }
}

#[test]
fn hsco_6_timelock_admin_named_in_sentence() {
    let p = protocol(vec![(
        CORE,
        ProgramRole::Core,
        timelock(7 * u64::from(DAY), safe(5, 8), Some(single_key())),
    )]);
    let s = score_protocol(&p, &m());
    assert!(
        s.risks
            .iter()
            .any(|r| r.text.contains("the timelock's admin")),
        "{:?}",
        s.risks
    );
}
