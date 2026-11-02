//! SCO-7: unavailable facts produce Unknown, shown separately from High.
//! The Stage consequences (Unrated, cap at 0) are tested with the Stage
//! decision in sco_6_stage.rs.

mod common;

use celebtir_core::Controller;
use celebtir_core::snapshot::{
    AdminPower, CodeTransparency, Dependencies, OracleRole, OracleSource, ProgramRole,
};
use celebtir_score::{Factor, Methodology, Rating, factors, score_protocol, weakest};
use common::*;

fn unavailable(reason: &str) -> Controller {
    Controller::Unavailable {
        reason: reason.into(),
    }
}

#[test]
fn sco_7_undecodable_admin_account_shows_unknown_not_high() {
    let mut p = protocol(vec![(
        "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
        ProgramRole::Core,
        multisig(5, 8, 0),
    )]);
    p.admin = admin(vec![(
        AdminPower::Parameters,
        unavailable("IDL decode failed for field lending_market_owner"),
    )]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    let a = &s.factors[&Factor::AdminPowers];
    assert_eq!(a.rating, Rating::Unknown);
    assert!(a.reason.contains("IDL decode failed"), "{}", a.reason);
}

#[test]
fn sco_7_unavailable_upgrade_authority_is_unknown() {
    let m = Methodology::bundled("0.2").unwrap();
    let c = fact(unavailable("ProgramData read failed"));
    assert_eq!(factors::upgrade_control(&c, &m).rating, Rating::Unknown);
    assert_eq!(factors::upgrade_delay(&c, &m).rating, Rating::Unknown);
}

#[test]
fn sco_7_every_unavailable_fact_kind_is_unknown() {
    let m = Methodology::bundled("0.2").unwrap();
    assert_eq!(
        factors::code_transparency(&fact(CodeTransparency::Unavailable {
            reason: "api".into()
        }))
        .rating,
        Rating::Unknown
    );
    assert_eq!(
        factors::oracle_dependency(
            &oracles(vec![(
                "SOL",
                OracleRole::Primary,
                OracleSource::Unavailable { reason: "x".into() }
            )]),
            &m
        )
        .rating,
        Rating::Unknown
    );
    assert_eq!(
        factors::dependencies(&fact(Dependencies::Unavailable { reason: "x".into() }), &m).rating,
        Rating::Unknown
    );
}

#[test]
fn sco_7_unknown_is_neutral_not_a_risk_level() {
    assert_eq!(Rating::Unknown.level(), None);
    assert_ne!(Rating::Unknown, Rating::High);
    // Unknown next to Low stays Unknown: missing data never helps…
    assert_eq!(
        weakest([Rating::Low, Rating::Unknown]),
        Some(Rating::Unknown)
    );
    // …and never hides a known High.
    assert_eq!(weakest([Rating::Unknown, Rating::High]), Some(Rating::High));
}

#[test]
fn sco_7_cannot_act_multisig_is_unknown_with_explanation() {
    let c = fact(Controller::CannotAct {
        address: addr("x"),
        threshold: 5,
        voters: 3,
    });
    let a = factors::upgrade_control(&c, &Methodology::bundled("0.2").unwrap());
    assert_eq!(a.rating, Rating::Unknown);
    assert!(a.reason.contains("cannot act"), "{}", a.reason);
}

#[test]
fn sco_7_multisig_with_threshold_above_voters_is_unknown() {
    let a = factors::upgrade_control(
        &fact(multisig(4, 3, 0)),
        &Methodology::bundled("0.2").unwrap(),
    );
    assert_eq!(a.rating, Rating::Unknown);
    assert!(a.reason.contains("cannot act"), "{}", a.reason);
}
