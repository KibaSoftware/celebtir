//! SCO-5: every rating carries a one-line reason, evidence references, and
//! what would raise it one level.

mod common;

use celebtir_core::Controller;
use celebtir_core::snapshot::{
    AdminPower, CodeTransparency, Dependencies, OracleRole, OracleSource, ProgramRole,
};
use celebtir_score::{Factor, FactorScore, Methodology, Rating, factors, score_protocol};
use common::*;

const CORE: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

fn controllers() -> Vec<Controller> {
    vec![
        Controller::Immutable,
        single_key(),
        multisig(5, 8, 5 * DAY),
        multisig(2, 4, DAY),
        multisig(1, 2, 0),
        with_config_authority(multisig(5, 8, 5 * DAY), single_key()),
        Controller::CannotAct {
            address: addr("x"),
            threshold: 4,
            voters: 3,
        },
        Controller::Unrecognized {
            key: addr("y"),
            owner_program: None,
        },
        Controller::Unavailable {
            reason: "rpc timeout".into(),
        },
    ]
}

fn assert_complete(where_: &str, s: &FactorScore) {
    assert!(!s.reason.trim().is_empty(), "{where_}: empty reason");
    assert!(!s.reason.contains('\n'), "{where_}: reason is not one line");
    assert!(!s.next_step.trim().is_empty(), "{where_}: empty next step");
    assert!(!s.evidence.is_empty(), "{where_}: no evidence");
}

#[test]
fn sco_5_every_rating_has_reason_next_step_and_evidence() {
    let m = Methodology::bundled("0.2").unwrap();
    let oracle_sets = [
        oracles(vec![]),
        oracles(vec![("SOL", OracleRole::Primary, network("pyth"))]),
        oracles(vec![
            ("SOL", OracleRole::Primary, network("pyth")),
            ("SOL", OracleRole::Fallback, network("switchboard")),
        ]),
        oracles(vec![(
            "SOL",
            OracleRole::Primary,
            OracleSource::ProtocolRun,
        )]),
        oracles(vec![(
            "SOL",
            OracleRole::Primary,
            OracleSource::Unavailable { reason: "x".into() },
        )]),
    ];
    let transparencies = [
        verified(),
        fact(CodeTransparency::PublicSourceUnverified {
            repo: "r".into(),
            commit: None,
            on_chain_hash: None,
        }),
        fact(CodeTransparency::NoPublicSource),
        fact(CodeTransparency::Unavailable {
            reason: "api down".into(),
        }),
    ];
    let mut checked = 0;
    for (i, c) in controllers().into_iter().enumerate() {
        for (j, o) in oracle_sets.iter().enumerate() {
            let mut p = protocol(vec![(CORE, ProgramRole::Core, c.clone())]);
            p.programs.get_mut(&addr(CORE)).unwrap().code_transparency =
                transparencies[(i + j) % transparencies.len()].clone();
            p.oracles = o.clone();
            p.admin = match j % 3 {
                0 => admin(vec![]),
                1 => admin(vec![(AdminPower::Funds, c.clone())]),
                _ => admin(vec![(AdminPower::Parameters, c.clone())]),
            };
            p.dependencies = if j == 4 {
                fact(Dependencies::Unavailable {
                    reason: "no sample".into(),
                })
            } else {
                dependencies(vec![c.clone()])
            };
            let s = score_protocol(&p, &m);
            for f in Factor::ALL {
                assert_complete(&format!("{c:?} / oracles {j} / {f:?}"), &s.factors[&f]);
                checked += 1;
            }
            for (a, ps) in &s.programs {
                for (name, fs) in [
                    ("upgrade_control", &ps.upgrade_control),
                    ("upgrade_delay", &ps.upgrade_delay),
                    ("code_transparency", &ps.code_transparency),
                ] {
                    assert_complete(&format!("{a} {name}"), fs);
                }
            }
        }
    }
    assert!(checked >= 200);
}

#[test]
fn sco_5_next_step_quotes_methodology_thresholds() {
    let m = Methodology::bundled("0.2").unwrap();
    let a = factors::upgrade_control(&fact(multisig(2, 4, 0)), &m);
    assert_eq!(a.rating, Rating::Medium);
    assert!(
        a.next_step.contains("5+ voters")
            && a.next_step.contains("60%")
            && a.next_step.contains("LOW"),
        "{}",
        a.next_step
    );

    let d = factors::upgrade_delay(&fact(multisig(5, 8, 0)), &m);
    assert_eq!(d.rating, Rating::High);
    assert!(
        d.next_step.contains("24 h") && d.next_step.contains("MEDIUM"),
        "{}",
        d.next_step
    );
    assert!(
        d.next_step.contains("5 days") && d.next_step.contains("LOW"),
        "{}",
        d.next_step
    );
}

#[test]
fn sco_5_config_authority_next_step_names_it() {
    let m = Methodology::bundled("0.2").unwrap();
    let c = with_config_authority(multisig(5, 8, 5 * DAY), single_key());
    for a in [
        factors::upgrade_control(&fact(c.clone()), &m),
        factors::upgrade_delay(&fact(c), &m),
    ] {
        assert_eq!(a.rating, Rating::High);
        assert!(a.next_step.contains("config authority"), "{}", a.next_step);
    }
}

#[test]
fn sco_5_low_ratings_say_there_is_no_higher_level() {
    let m = Methodology::bundled("0.2").unwrap();
    let a = factors::upgrade_control(&fact(Controller::Immutable), &m);
    assert_eq!(a.rating, Rating::Low);
    assert!(a.next_step.contains("lowest"), "{}", a.next_step);
}

#[test]
fn sco_5_empty_sets_borrow_program_evidence() {
    let p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    for f in [Factor::AdminPowers, Factor::OracleDependency] {
        assert!(!s.factors[&f].evidence.is_empty(), "{f:?}");
    }
}

/// The reason says what the borrowed evidence shows: the registry lists no
/// admin accounts or feeds, and the core programs' reads are cited instead.
#[test]
fn sco_5_empty_sets_say_what_the_borrowed_evidence_shows() {
    let p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(
        s.factors[&Factor::AdminPowers].reason,
        "the registry lists no admin accounts; the evidence is the core programs' upgrade reads"
    );
    assert_eq!(
        s.factors[&Factor::OracleDependency].reason,
        "the registry lists no price feeds; the evidence is the core programs' upgrade reads"
    );
}
