//! SCO-4: the weakest core program decides each protocol factor; periphery
//! programs are shown but not counted.

mod common;

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{CodeTransparency, FactsFile, ProgramRole};
use celebtir_score::{Factor, Methodology, Rating, score, score_protocol};
use common::*;

const CORE_A: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const CORE_B: &str = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";
const PERIPHERY: &str = "FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr";

#[test]
fn sco_4_single_key_periphery_program_keeps_protocol_low() {
    let p = protocol(vec![
        (CORE_A, ProgramRole::Core, multisig(5, 8, 0)),
        (PERIPHERY, ProgramRole::Periphery, single_key()),
    ]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.factors[&Factor::UpgradeControl].rating, Rating::Low);
    // The periphery program is still shown, with its own High rating.
    assert_eq!(
        s.programs[&addr(PERIPHERY)].upgrade_control.rating,
        Rating::High
    );
    assert_eq!(s.programs[&addr(PERIPHERY)].role, ProgramRole::Periphery);
}

#[test]
fn sco_4_one_single_key_core_program_sinks_the_protocol() {
    let p = protocol(vec![
        (CORE_A, ProgramRole::Core, Controller::Immutable),
        (CORE_B, ProgramRole::Core, single_key()),
    ]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    let uc = &s.factors[&Factor::UpgradeControl];
    assert_eq!(uc.rating, Rating::High);
    assert!(
        uc.reason.contains("dRif"),
        "reason names the weakest program: {}",
        uc.reason
    );
}

#[test]
fn sco_4_weakest_link_applies_to_delay_and_transparency() {
    let mut p = protocol(vec![
        (CORE_A, ProgramRole::Core, multisig(5, 8, 5 * DAY)),
        (CORE_B, ProgramRole::Core, multisig(5, 8, DAY)),
    ]);
    p.programs.get_mut(&addr(CORE_B)).unwrap().code_transparency =
        fact(CodeTransparency::PublicSourceUnverified {
            repo: "https://github.com/x/y".into(),
            commit: None,
            on_chain_hash: None,
        });
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.factors[&Factor::UpgradeDelay].rating, Rating::Medium);
    assert_eq!(s.factors[&Factor::CodeTransparency].rating, Rating::Medium);
}

#[test]
fn sco_4_periphery_transparency_is_not_counted() {
    let mut p = protocol(vec![
        (CORE_A, ProgramRole::Core, Controller::Immutable),
        (PERIPHERY, ProgramRole::Periphery, Controller::Immutable),
    ]);
    p.programs
        .get_mut(&addr(PERIPHERY))
        .unwrap()
        .code_transparency = fact(CodeTransparency::NoPublicSource);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.factors[&Factor::CodeTransparency].rating, Rating::Low);
}

#[test]
fn sco_4_all_six_factors_are_scored() {
    let p = protocol(vec![(CORE_A, ProgramRole::Core, Controller::Immutable)]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.factors.len(), 6);
    assert_eq!(
        s.factors[&Factor::OracleDependency].rating,
        Rating::NotApplicable
    );
}

#[test]
fn sco_4_score_covers_every_protocol_in_the_facts_file() {
    let mut protocols = BTreeMap::new();
    protocols.insert(
        "a".to_string(),
        protocol(vec![(CORE_A, ProgramRole::Core, single_key())]),
    );
    protocols.insert(
        "b".to_string(),
        protocol(vec![(CORE_B, ProgramRole::Core, Controller::Immutable)]),
    );
    let scores = score(
        &FactsFile::new(protocols),
        &Methodology::bundled("0.2").unwrap(),
    );
    assert_eq!(scores.methodology_version, "0.2");
    assert_eq!(
        scores.protocols["a"].factors[&Factor::UpgradeControl].rating,
        Rating::High
    );
    assert_eq!(
        scores.protocols["b"].factors[&Factor::UpgradeControl].rating,
        Rating::Low
    );
}
