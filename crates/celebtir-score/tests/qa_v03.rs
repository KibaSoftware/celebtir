//! Methodology v0.3 (launch audit, CELEBTIR_FIX_INSTRUCTIONS): one test per
//! new rule, and the guarantee that v0.1/v0.2 replay ignores every new
//! representation.

mod common;

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    AdminPower, FactsFile, OracleRole, OracleSource, ProgramRole, ReviewedNone, UnderlyingFeed,
    UnderlyingFeeds,
};
use celebtir_core::{Address, Chain, Controller, MultisigKind};
use celebtir_score::value::ControlClass;
use celebtir_score::{Factor, Methodology, Rating, Stage, score, score_protocol, score_token};
use common::*;

const CORE: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

fn v03() -> Methodology {
    Methodology::shipped()
}

fn v02() -> Methodology {
    Methodology::bundled("0.2").unwrap()
}

fn pda() -> Controller {
    Controller::Pda {
        key: addr("6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        owner_program: Some(addr("11111111111111111111111111111111")),
    }
}

fn review() -> ReviewedNone {
    ReviewedNone {
        source: "https://github.com/solana-program/stake-pool".into(),
        reviewed_on: "2026-10-25".into(),
        note: "no instruction takes an oracle account".into(),
    }
}

#[test]
fn aud_2_unresolved_pda_token_authority_is_unknown_not_high() {
    let mut t = token();
    t.mint_authority = fact(pda());
    let s = score_token(&t, &v03());
    let supply = &s.factors[&celebtir_score::TokenFactor::SupplyControl];
    assert_eq!(supply.rating, Rating::Unknown);
    assert!(
        supply.reason.contains("program-derived address"),
        "{}",
        supply.reason
    );
    assert!(!supply.reason.contains("unrecognized"), "{}", supply.reason);
    // v0.2 replays the same fact as it was written then: Unrecognized, High.
    let legacy = score_token(&t, &v02());
    assert_eq!(
        legacy.factors[&celebtir_score::TokenFactor::SupplyControl].rating,
        Rating::High
    );
}

#[test]
fn aud_2_verified_pda_inherits_its_program_upgrade_control() {
    let mut t = token();
    t.mint_authority = fact(Controller::ProgramControlled {
        key: addr("6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        program: addr("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"),
        upgrade: Box::new(multisig(6, 10, 0)),
    });
    let s = score_token(&t, &v03());
    assert_eq!(
        s.factors[&celebtir_score::TokenFactor::SupplyControl].rating,
        Rating::Low
    );
}

#[test]
fn aud_2_pda_upgrade_authority_makes_the_protocol_unrated() {
    let p = protocol(vec![(CORE, ProgramRole::Core, pda())]);
    let s = score_protocol(&p, &v03());
    assert_eq!(s.factors[&Factor::UpgradeControl].rating, Rating::Unknown);
    assert_eq!(s.factors[&Factor::UpgradeDelay].rating, Rating::Unknown);
    assert_eq!(s.stage.stage, Stage::Unrated);
    assert_eq!(s.control_class, ControlClass::Unrecognized);
}

#[test]
fn aud_3_empty_admin_list_is_unknown_without_borrowed_evidence() {
    let p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    let s = score_protocol(&p, &v03());
    let a = &s.factors[&Factor::AdminPowers];
    assert_eq!(a.rating, Rating::Unknown);
    assert!(a.evidence.is_empty(), "{:?}", a.evidence);
    assert!(!a.reason.contains("upgrade reads"), "{}", a.reason);
    let o = &s.factors[&Factor::OracleDependency];
    assert_eq!(o.rating, Rating::Unknown);
    assert!(o.evidence.is_empty());
    // Unknown caps the Stage at 0.
    assert_ne!(s.stage.stage, Stage::Stage1);
    // v0.2 keeps its published behaviour.
    let legacy = score_protocol(&p, &v02());
    assert_eq!(legacy.factors[&Factor::AdminPowers].rating, Rating::Low);
}

#[test]
fn aud_3_sourced_reviewed_none_cites_its_source() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    p.admin_review = Some(review());
    p.oracle_review = Some(review());
    let s = score_protocol(&p, &v03());
    let a = &s.factors[&Factor::AdminPowers];
    assert_eq!(a.rating, Rating::Low);
    assert_eq!(a.registry_sources, vec![review().source]);
    let o = &s.factors[&Factor::OracleDependency];
    assert_eq!(o.rating, Rating::NotApplicable);
    assert_eq!(o.registry_sources, vec![review().source]);
}

#[test]
fn aud_3_lending_without_feeds_stays_unknown_even_with_a_review() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    p.category = Some("lending".into());
    p.oracle_review = Some(review());
    let s = score_protocol(&p, &v03());
    assert_eq!(s.factors[&Factor::OracleDependency].rating, Rating::Unknown);
}

#[test]
fn aud_7_value_classes_split_no_delay_short_and_delayed() {
    let m = v03();
    let class = |lock: u32| {
        score_protocol(
            &protocol(vec![(CORE, ProgramRole::Core, multisig(4, 6, lock))]),
            &m,
        )
        .control_class
    };
    assert_eq!(class(0), ControlClass::MultisigWithoutDelay);
    assert_eq!(class(600), ControlClass::MultisigShortDelay);
    assert_eq!(class(43_200), ControlClass::MultisigShortDelay);
    assert_eq!(class(DAY), ControlClass::MultisigWithDelay);
    // A config key that bypasses the lock means no delay.
    let bypass = with_config_authority(multisig(4, 6, 2 * DAY), single_key());
    let s = score_protocol(&protocol(vec![(CORE, ProgramRole::Core, bypass)]), &m);
    assert_eq!(s.control_class, ControlClass::SingleKey);
    // v0.2 had no short-delay class.
    let s = score_protocol(
        &protocol(vec![(CORE, ProgramRole::Core, multisig(4, 6, 43_200))]),
        &v02(),
    );
    assert_eq!(s.control_class, ControlClass::MultisigWithoutDelay);
}

#[test]
fn aud_7_value_rows_list_six_classes_in_v03_and_five_before() {
    let mut facts = FactsFile::new(BTreeMap::from([(
        "p".to_string(),
        protocol(vec![(CORE, ProgramRole::Core, multisig(4, 6, 600))]),
    )]));
    facts.metrics.clear();
    assert_eq!(score(&facts, &v03()).value_by_control.all.classes.len(), 6);
    assert_eq!(score(&facts, &v02()).value_by_control.all.classes.len(), 5);
}

fn guarded(guards: &[(&str, bool)], declared: &[&str]) -> Rating {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    p.category = Some("perps".into());
    let mut o = oracles(
        std::iter::once((
            "SOL custody",
            OracleRole::Primary,
            OracleSource::Other {
                owner_program: addr("DoVEsk76QybCEHQGzkvYPWLQu9gzNoZZZt3TPiL597e"),
            },
        ))
        .chain(
            guards
                .iter()
                .map(|(n, _)| ("SOL custody", OracleRole::Fallback, network(n))),
        )
        .collect(),
    );
    for (i, f) in o.values_mut().enumerate() {
        if i == 0 {
            f.guarded_by = declared.iter().map(|d| d.to_string()).collect();
            f.provider = Some("Edge by Chaos Labs".into());
        } else {
            f.bounds_primary = guards[i - 1].1;
        }
    }
    p.oracles = o;
    score_protocol(&p, &v03()).factors[&Factor::OracleDependency].rating
}

#[test]
fn aud_4_guarded_primary_is_low_only_with_two_registered_bounding_networks() {
    assert_eq!(
        guarded(
            &[("chainlink", true), ("pyth", true)],
            &["chainlink", "pyth"]
        ),
        Rating::Low
    );
    assert_eq!(
        guarded(&[("chainlink", true), ("pyth", true)], &[]),
        Rating::Low
    );
    // A documented guard that is not registered is a gap, not High or Low.
    assert_eq!(
        guarded(&[("pyth", true)], &["chainlink", "pyth"]),
        Rating::Unknown
    );
    // Fallbacks that do not bound the primary are not guards.
    assert_eq!(
        guarded(&[("chainlink", false), ("pyth", false)], &[]),
        Rating::High
    );
}

#[test]
fn aud_4_protocol_run_reason_names_what_the_aggregator_combines() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    let mut o = oracles(vec![(
        "SOL reserve",
        OracleRole::Primary,
        OracleSource::ProtocolRun,
    )]);
    let f = o.values_mut().next().unwrap();
    f.underlying = Some(fact(UnderlyingFeeds::Listed {
        feeds: vec![
            UnderlyingFeed {
                source_type: "Chainlink".into(),
                address: addr("a"),
            },
            UnderlyingFeed {
                source_type: "PythLazer".into(),
                address: addr("b"),
            },
        ],
    }));
    p.oracles = o;
    let r = &score_protocol(&p, &v03()).factors[&Factor::OracleDependency];
    assert_eq!(r.rating, Rating::High);
    assert!(r.reason.contains("Chainlink, PythLazer"), "{}", r.reason);
}

#[test]
fn aud_5_unassessed_chains_get_no_ratings_stage_or_value_class() {
    let mut evm = protocol(vec![(CORE, ProgramRole::Core, multisig(4, 8, 0))]);
    evm.chain = Chain::HyperEvm;
    let facts = FactsFile::new(BTreeMap::from([
        (
            "sol".to_string(),
            protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]),
        ),
        ("evm".to_string(), evm),
    ]));
    let s = score(&facts, &v03());
    assert!(s.protocols.contains_key("sol"));
    assert!(!s.protocols.contains_key("evm"));
    assert!(!s.unassessed["evm"].rated);
    assert_eq!(s.schema_version, "0.2.0");
    // v0.2 rated every chain and wrote the 0.1.0 format.
    let legacy = score(&facts, &v02());
    assert!(legacy.protocols.contains_key("evm") && legacy.unassessed.is_empty());
    assert_eq!(legacy.schema_version, "0.1.0");
}

#[test]
fn aud_12_squads_v3_says_no_time_lock() {
    let v3 = match multisig(6, 10, 0) {
        Controller::Multisig {
            address,
            threshold,
            voters,
            ..
        } => Controller::Multisig {
            kind: MultisigKind::SquadsV3,
            address,
            threshold,
            voters,
            time_lock_secs: 0,
            config_authority: None,
            signers: vec![],
            nested_signers: Default::default(),
        },
        c => c,
    };
    let s = score_protocol(&protocol(vec![(CORE, ProgramRole::Core, v3)]), &v03());
    let d = &s.factors[&Factor::UpgradeDelay].reason;
    assert!(d.contains("no time lock (Squads v3"), "{d}");
    assert!(!s.factors[&Factor::UpgradeControl].reason.contains("0 s"));
}

#[test]
fn aud_15_admin_reasons_name_each_account() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    p.admin = admin(vec![
        (AdminPower::Parameters, multisig(4, 6, 0)),
        (AdminPower::Parameters, multisig(4, 6, 0)),
    ]);
    let r = &score_protocol(&p, &v03()).factors[&Factor::AdminPowers].reason;
    let parts: Vec<&str> = r.split("; ").collect();
    assert_eq!(parts.len(), 2, "{r}");
    assert_ne!(parts[0], parts[1], "{r}");
}

#[test]
fn aud_15_single_callee_reason_is_singular() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(6, 10, 0))]);
    p.dependencies = dependencies(vec![Controller::Immutable]);
    let r = &score_protocol(&p, &v03()).factors[&Factor::Dependencies].reason;
    assert_eq!(r, "the 1 callee is rated Low on upgrade control");
}

#[test]
fn aud_18_stage_2_names_pause_as_funds_frozen_in_place() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.admin = admin(vec![(AdminPower::Pause, multisig(6, 10, 0))]);
    let s = score_protocol(&p, &v03());
    let r = s
        .stage
        .requirements
        .iter()
        .find(|r| r.id == "s2.no_fund_control")
        .unwrap();
    assert!(!r.met);
    assert!(
        r.detail.contains("can pause (funds frozen in place)"),
        "{}",
        r.detail
    );
    assert!(
        r.description.contains("token-issuer freeze keys"),
        "{}",
        r.description
    );
}

#[test]
fn aud_2_v02_replay_lowers_pda_to_unrecognized_exactly() {
    let p = protocol(vec![(CORE, ProgramRole::Core, pda())]);
    let mut lowered = p.clone();
    for f in lowered.programs.values_mut() {
        f.upgrade_control = fact(Controller::Unrecognized {
            key: addr("6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
            owner_program: Some(addr("11111111111111111111111111111111")),
        });
    }
    assert_eq!(score_protocol(&p, &v02()), score_protocol(&lowered, &v02()));
    let _ = Address::new("x");
}
