//! SCO-6: Stage plus the unmet requirements for the next Stage.
//! SCO-7 (Stage side): Unknown upgrade control → Unrated; other Unknown caps at 0.

mod common;

use celebtir_core::Controller;
use celebtir_core::snapshot::{
    AdminPower, CodeTransparency, OracleRole, OracleSource, ProgramRole, ProtocolFacts,
};
use celebtir_score::{Methodology, Stage, StageResult, score_protocol};
use common::*;

const CORE: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const CORE_B: &str = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";

fn stage(p: &ProtocolFacts) -> StageResult {
    score_protocol(p, &Methodology::bundled("0.2").unwrap()).stage
}

fn missing_ids(s: &StageResult) -> Vec<&str> {
    s.missing.iter().map(|r| r.id.as_str()).collect()
}

fn guarded() -> ProtocolFacts {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(5, 8, 5 * DAY))]);
    p.admin = admin(vec![(AdminPower::Parameters, multisig(3, 5, 0))]);
    p.oracles = oracles(vec![("SOL", OracleRole::Primary, network("pyth"))]);
    p
}

#[test]
fn sco_6_stage_0_lists_exactly_the_failing_stage_1_requirements() {
    let p = protocol(vec![(CORE, ProgramRole::Core, multisig(5, 8, 0))]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage0);
    assert_eq!(s.next, Some(Stage::Stage1));
    assert_eq!(missing_ids(&s), vec!["s1.upgrade_delay"]);
    assert!(
        s.missing[0].detail.contains("time lock"),
        "{}",
        s.missing[0].detail
    );
}

#[test]
fn sco_6_stage_0_with_several_gaps_lists_each() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, single_key())]);
    p.admin = admin(vec![(AdminPower::Funds, single_key())]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage0);
    assert_eq!(
        missing_ids(&s),
        vec!["s1.upgrade_control", "s1.upgrade_delay", "s1.admin_powers"]
    );
}

#[test]
fn sco_6_guarded_protocol_is_stage_1_and_lists_stage_2_gaps() {
    let s = stage(&guarded());
    assert_eq!(s.stage, Stage::Stage1);
    assert_eq!(s.next, Some(Stage::Stage2));
    assert_eq!(missing_ids(&s), vec!["s2.immutable_or_governance"]);
}

#[test]
fn sco_6_emergency_multisig_is_exempt_from_the_delay() {
    // 8 of 10 voters = 80 % >= 75 %, 10 >= 8 voters.
    let p = protocol(vec![(CORE, ProgramRole::Core, multisig(8, 10, 0))]);
    assert_eq!(stage(&p).stage, Stage::Stage1);
}

#[test]
fn sco_6_immutable_protocol_without_fund_keys_is_stage_2() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.oracles = oracles(vec![
        ("SOL", OracleRole::Primary, network("pyth")),
        ("SOL", OracleRole::Fallback, network("switchboard")),
    ]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage2);
    assert_eq!(s.next, None);
    assert!(s.missing.is_empty());
}

#[test]
fn sco_6_pause_key_blocks_stage_2() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.admin = admin(vec![(AdminPower::Pause, multisig(5, 8, 0))]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage1);
    assert_eq!(missing_ids(&s), vec!["s2.no_fund_control"]);
}

#[test]
fn sco_6_single_key_config_authority_blocks_stage_1() {
    let c = with_config_authority(multisig(8, 10, 10 * DAY), single_key());
    let p = protocol(vec![(CORE, ProgramRole::Core, c)]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage0);
    assert!(
        missing_ids(&s).contains(&"s1.config_authority"),
        "{:?}",
        missing_ids(&s)
    );
}

#[test]
fn sco_6_unverified_build_is_stage_0_with_verified_builds_missing() {
    // ADR 001.
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.programs.get_mut(&addr(CORE)).unwrap().code_transparency =
        fact(CodeTransparency::PublicSourceUnverified {
            repo: "https://github.com/x/y".into(),
            commit: None,
            on_chain_hash: None,
        });
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage0);
    assert_eq!(missing_ids(&s), vec!["s1.verified_builds"]);
}

#[test]
fn sco_6_no_public_source_is_unrated() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.programs.get_mut(&addr(CORE)).unwrap().code_transparency =
        fact(CodeTransparency::NoPublicSource);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Unrated);
    assert_eq!(s.next, Some(Stage::Stage0));
    assert_eq!(missing_ids(&s), vec!["s0.public_source"]);
}

#[test]
fn sco_6_only_core_programs_decide_the_stage() {
    let p = protocol(vec![
        (CORE, ProgramRole::Core, Controller::Immutable),
        (CORE_B, ProgramRole::Periphery, single_key()),
    ]);
    assert_eq!(stage(&p).stage, Stage::Stage2);
}

#[test]
fn sco_6_every_requirement_is_reported_met_or_unmet() {
    let s = stage(&guarded());
    let ids: Vec<&str> = s.requirements.iter().map(|r| r.id.as_str()).collect();
    for id in [
        "s0.core_programs",
        "s0.public_source",
        "s0.upgrade_control_known",
        "s1.upgrade_control",
        "s1.admin_powers",
        "s1.upgrade_delay",
        "s1.config_authority",
        "s1.verified_builds",
        "s1.no_unknown",
        "s2.immutable_or_governance",
        "s2.no_fund_control",
    ] {
        assert!(ids.contains(&id), "missing requirement {id}");
    }
}

#[test]
fn sco_7_unknown_upgrade_control_makes_the_protocol_unrated() {
    let p = protocol(vec![(
        CORE,
        ProgramRole::Core,
        Controller::Unavailable {
            reason: "rpc".into(),
        },
    )]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Unrated);
    assert_eq!(missing_ids(&s), vec!["s0.upgrade_control_known"]);
}

#[test]
fn sco_7_any_other_unknown_caps_the_stage_at_0() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.oracles = oracles(vec![(
        "SOL",
        OracleRole::Primary,
        OracleSource::Unavailable { reason: "x".into() },
    )]);
    let s = stage(&p);
    assert_eq!(s.stage, Stage::Stage0);
    assert_eq!(missing_ids(&s), vec!["s1.no_unknown"]);
    assert!(
        s.missing[0].detail.contains("Oracle dependency"),
        "{}",
        s.missing[0].detail
    );
}

#[test]
fn sco_7_unreadable_config_authority_makes_the_protocol_unrated_even_when_high() {
    // Found by the SCO-8 property tests: a weak multisig rates High whatever
    // its config authority is, but the protocol still cannot be assessed.
    let c = with_config_authority(
        multisig(1, 2, 0),
        Controller::Unavailable {
            reason: "rpc".into(),
        },
    );
    let s = stage(&protocol(vec![(CORE, ProgramRole::Core, c)]));
    assert_eq!(s.stage, Stage::Unrated);
    assert_eq!(missing_ids(&s), vec!["s0.upgrade_control_known"]);
}
