//! UI-21: every node in the control graph is colored by its own rating, so the
//! scorer publishes one per admin field and per sampled callee, from the same
//! rules as the factors (never thresholds in the site, SCO-2).

mod common;

use celebtir_core::snapshot::{AdminPower, ProgramRole};
use celebtir_score::{Methodology, Rating, score_protocol};
use common::*;

const DAY: u32 = 86_400;

#[test]
fn ui_21_each_admin_field_has_its_own_rating_and_reason() {
    let mut p = protocol(vec![("prog", ProgramRole::Core, multisig(5, 10, DAY))]);
    p.admin = admin(vec![
        (AdminPower::Parameters, multisig(3, 5, 7 * DAY)),
        (AdminPower::Funds, single_key()),
    ]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.admin.len(), 2);
    assert_eq!(s.admin["acct0.owner"].rating, Rating::Low);
    assert_eq!(s.admin["acct1.owner"].rating, Rating::High);
    assert!(s.admin["acct1.owner"].reason.contains("move funds"));
    // The factor is still the weakest link of the fields.
    assert_eq!(
        s.factors[&celebtir_score::Factor::AdminPowers].rating,
        Rating::High
    );
}

#[test]
fn ui_21_each_sampled_callee_has_its_own_rating() {
    let mut p = protocol(vec![("prog", ProgramRole::Core, multisig(5, 10, DAY))]);
    p.dependencies = dependencies(vec![single_key(), celebtir_core::Controller::Immutable]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.callees[&addr("callee0")].rating, Rating::High);
    assert_eq!(s.callees[&addr("callee1")].rating, Rating::Low);
}

#[test]
fn ui_21_no_dependencies_sampled_means_no_callee_ratings() {
    let p = protocol(vec![("prog", ProgramRole::Core, multisig(5, 10, DAY))]);
    assert!(
        score_protocol(&p, &Methodology::bundled("0.2").unwrap())
            .callees
            .is_empty()
    );
}
