//! SCO-3: the six factor rules follow the PRD table, one test per cell.
//! SCO-2: changing a threshold in the methodology file changes ratings.

mod common;

use celebtir_core::snapshot::{AdminPower, CodeTransparency, OracleRole, OracleSource};
use celebtir_score::{Methodology, Rating, SHIPPED_METHODOLOGY, factors};
use common::*;

fn m() -> Methodology {
    Methodology::bundled("0.2").unwrap()
}

// ── Upgrade control ─────────────────────────────────────────────────────────

#[test]
fn sco_3_upgrade_control_low_when_immutable() {
    let a = factors::upgrade_control(&fact(celebtir_core::Controller::Immutable), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_low_for_5_of_8_multisig() {
    let a = factors::upgrade_control(&fact(multisig(5, 8, 0)), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
    assert!(a.reason.contains("5 of 8"), "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_medium_for_2_of_4_multisig() {
    // 4 voters < 5, 50 % >= 50 %.
    let a = factors::upgrade_control(&fact(multisig(2, 4, 0)), &m());
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_medium_for_large_multisig_below_60_pct() {
    // 9 voters but only 5/9 = 55 %.
    let a = factors::upgrade_control(&fact(multisig(5, 9, 0)), &m());
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_high_for_single_key() {
    let a = factors::upgrade_control(&fact(single_key()), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_high_for_small_multisig() {
    let a = factors::upgrade_control(&fact(multisig(2, 2, 0)), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_high_when_config_authority_is_one_key() {
    let c = with_config_authority(multisig(6, 9, 0), single_key());
    let a = factors::upgrade_control(&fact(c), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
    assert!(a.reason.contains("config authority"), "{}", a.reason);
}

#[test]
fn sco_3_ratio_uses_voters_not_members() {
    // A 7-member multisig where 2 members cannot vote is recorded as 5 voters
    // by the collector (COL-9); 3 of 5 voters is exactly 60 %.
    let a = factors::upgrade_control(&fact(multisig(3, 5, 0)), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_control_high_for_unrecognized_controller() {
    // ADR 002.
    let c = celebtir_core::Controller::Unrecognized {
        key: addr("GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw"),
        owner_program: Some(addr("GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw")),
    };
    let a = factors::upgrade_control(&fact(c), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

// ── Upgrade delay ───────────────────────────────────────────────────────────

#[test]
fn sco_3_upgrade_delay_low_for_5_day_time_lock() {
    let a = factors::upgrade_delay(&fact(multisig(5, 8, 5 * DAY)), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_delay_low_when_immutable() {
    let a = factors::upgrade_delay(&fact(celebtir_core::Controller::Immutable), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_delay_medium_for_24_hours() {
    let a = factors::upgrade_delay(&fact(multisig(5, 8, DAY)), &m());
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_delay_high_without_time_lock() {
    let a = factors::upgrade_delay(&fact(multisig(5, 8, 0)), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_delay_high_for_single_key() {
    let a = factors::upgrade_delay(&fact(single_key()), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_upgrade_delay_config_authority_overrides_time_lock() {
    // ADR 005: a single-key config authority can remove the time lock at once.
    let c = with_config_authority(multisig(5, 8, 10 * DAY), single_key());
    let a = factors::upgrade_delay(&fact(c), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

// ── Admin powers ────────────────────────────────────────────────────────────

#[test]
fn sco_3_admin_powers_low_without_admin_keys() {
    let a = factors::admin_powers(&admin(vec![]), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_admin_powers_low_for_parameters_behind_time_lock() {
    let a = factors::admin_powers(
        &admin(vec![(AdminPower::Parameters, multisig(3, 5, 2 * DAY))]),
        &m(),
    );
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_admin_powers_medium_for_pause_held_by_multisig() {
    let a = factors::admin_powers(&admin(vec![(AdminPower::Pause, multisig(3, 5, 0))]), &m());
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_admin_powers_medium_for_parameters_without_time_lock() {
    let a = factors::admin_powers(
        &admin(vec![(AdminPower::Parameters, multisig(3, 5, 0))]),
        &m(),
    );
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_admin_powers_high_when_one_key_changes_parameters() {
    let a = factors::admin_powers(&admin(vec![(AdminPower::Parameters, single_key())]), &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_admin_powers_high_when_one_key_moves_funds() {
    let a = factors::admin_powers(
        &admin(vec![
            (AdminPower::Pause, multisig(3, 5, 0)),
            (AdminPower::Funds, single_key()),
        ]),
        &m(),
    );
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_admin_powers_low_when_admin_renounced() {
    let a = factors::admin_powers(
        &admin(vec![(
            AdminPower::Funds,
            celebtir_core::Controller::Immutable,
        )]),
        &m(),
    );
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

// ── Code transparency ───────────────────────────────────────────────────────

#[test]
fn sco_3_code_transparency_low_for_verified_build() {
    let a = factors::code_transparency(&verified());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_code_transparency_medium_for_public_unverified_source() {
    let a = factors::code_transparency(&fact(CodeTransparency::PublicSourceUnverified {
        repo: "https://github.com/x/y".into(),
        commit: None,
        on_chain_hash: None,
    }));
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_code_transparency_high_without_public_source() {
    let a = factors::code_transparency(&fact(CodeTransparency::NoPublicSource));
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

// ── Oracle dependency ───────────────────────────────────────────────────────

#[test]
fn sco_3_oracle_low_for_established_network_with_fallback() {
    let o = oracles(vec![
        ("SOL", OracleRole::Primary, network("pyth")),
        ("SOL", OracleRole::Fallback, network("switchboard")),
    ]);
    let a = factors::oracle_dependency(&o, &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_oracle_medium_for_single_established_network() {
    let o = oracles(vec![
        ("SOL", OracleRole::Primary, network("pyth")),
        ("USDC", OracleRole::Primary, network("pyth")),
        ("USDC", OracleRole::Fallback, network("switchboard")),
    ]);
    let a = factors::oracle_dependency(&o, &m());
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
    assert!(a.reason.contains("SOL"), "{}", a.reason);
}

#[test]
fn sco_3_oracle_high_for_protocol_run_feed() {
    let o = oracles(vec![(
        "SOL",
        OracleRole::Primary,
        OracleSource::ProtocolRun,
    )]);
    let a = factors::oracle_dependency(&o, &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_oracle_high_for_unclassified_owner() {
    let o = oracles(vec![(
        "SOL",
        OracleRole::Primary,
        OracleSource::Other {
            owner_program: addr("Unknown111111111111111111111111111111111111"),
        },
    )]);
    let a = factors::oracle_dependency(&o, &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

#[test]
fn sco_3_oracle_not_applicable_without_price_feeds() {
    let a = factors::oracle_dependency(&oracles(vec![]), &m());
    assert_eq!(a.rating, Rating::NotApplicable, "{}", a.reason);
}

// ── Dependencies ────────────────────────────────────────────────────────────

#[test]
fn sco_3_dependencies_low_when_all_callees_are_low() {
    let d = dependencies(vec![
        celebtir_core::Controller::Immutable,
        multisig(5, 8, 0),
    ]);
    let a = factors::dependencies(&d, &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_dependencies_low_without_callees() {
    let a = factors::dependencies(&dependencies(vec![]), &m());
    assert_eq!(a.rating, Rating::Low, "{}", a.reason);
}

#[test]
fn sco_3_dependencies_medium_when_any_callee_is_medium() {
    let d = dependencies(vec![
        celebtir_core::Controller::Immutable,
        multisig(2, 4, 0),
    ]);
    let a = factors::dependencies(&d, &m());
    assert_eq!(a.rating, Rating::Medium, "{}", a.reason);
}

#[test]
fn sco_3_dependencies_high_when_any_callee_is_high() {
    let d = dependencies(vec![multisig(2, 4, 0), single_key()]);
    let a = factors::dependencies(&d, &m());
    assert_eq!(a.rating, Rating::High, "{}", a.reason);
}

// ── SCO-2 ───────────────────────────────────────────────────────────────────

#[test]
fn sco_2_changing_the_5_day_delay_to_7_days_changes_ratings() {
    let six_days = fact(multisig(5, 8, 6 * DAY));
    assert_eq!(factors::upgrade_delay(&six_days, &m()).rating, Rating::Low);

    let seven = Methodology::from_toml(
        &SHIPPED_METHODOLOGY.replace("low_min_secs = 432000", "low_min_secs = 604800"),
    )
    .unwrap();
    assert_eq!(
        factors::upgrade_delay(&six_days, &seven).rating,
        Rating::Medium
    );
}
