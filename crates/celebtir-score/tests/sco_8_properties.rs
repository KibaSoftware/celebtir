//! SCO-8: adding a voter, raising a time lock, or removing an admin key never
//! makes a rating worse; scoring is deterministic. 10,000 cases each.
//!
//! "Adding a voter" adds a member who also counts toward the threshold (ADR
//! 014): under the PRD's percentage rule, a voter added at a fixed threshold
//! can lower the ratio (3 of 5 = 60 % → 3 of 6 = 50 %), which is a correct
//! downgrade, not a bug.

mod common;

use std::collections::BTreeMap;

use celebtir_core::snapshot::{AdminPower, FactsFile, ProgramRole, ProtocolFacts};
use celebtir_core::{Address, Controller, MultisigKind};
use celebtir_score::{Factor, Methodology, Rating, score, score_protocol};
use common::*;
use proptest::prelude::*;

const CASES: u32 = 10_000;

fn leaf() -> impl Strategy<Value = Controller> {
    prop_oneof![
        Just(Controller::Immutable),
        Just(single_key()),
        Just(Controller::Unrecognized {
            key: addr("u"),
            owner_program: None
        }),
        Just(Controller::Unavailable { reason: "x".into() }),
    ]
}

fn multisig_strategy(
    inner: impl Strategy<Value = Controller> + Clone,
) -> impl Strategy<Value = Controller> {
    (
        0u16..=12,
        0u16..=12,
        0u32..=40 * DAY,
        proptest::option::of(inner),
    )
        .prop_map(
            |(threshold, voters, time_lock_secs, ca)| Controller::Multisig {
                kind: MultisigKind::SquadsV4,
                address: addr("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"),
                threshold,
                voters,
                time_lock_secs,
                config_authority: ca.map(Box::new),
                signers: vec![],
                nested_signers: Default::default(),
            },
        )
}

fn controller() -> impl Strategy<Value = Controller> {
    leaf().prop_recursive(2, 4, 1, |inner| {
        prop_oneof![multisig_strategy(inner.clone()), inner]
    })
}

fn power() -> impl Strategy<Value = AdminPower> {
    prop_oneof![
        Just(AdminPower::Parameters),
        Just(AdminPower::Pause),
        Just(AdminPower::Funds)
    ]
}

fn protocol_strategy() -> impl Strategy<Value = ProtocolFacts> {
    protocol_with_admins(0..3)
}

fn protocol_with_admins(admins: std::ops::Range<usize>) -> impl Strategy<Value = ProtocolFacts> {
    (
        proptest::collection::vec(controller(), 1..3),
        proptest::collection::vec((power(), controller()), admins),
    )
        .prop_map(|(programs, admins)| {
            let names = [
                "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
                "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH",
            ];
            let mut p = protocol(
                programs
                    .into_iter()
                    .enumerate()
                    .map(|(i, c)| (names[i], ProgramRole::Core, c))
                    .collect(),
            );
            p.admin = admin(admins);
            p
        })
}

/// Order used by `weakest()` when combining: Low < Medium < Unknown < High. A
/// known High beats missing data, and missing data beats a known Low or Medium,
/// because it could be worse (SCO-7, ADR 011 #10). So High → Unknown is an
/// improvement, and Low → Unknown is not allowed (ADR 014).
fn rank(r: Rating) -> u8 {
    match r {
        Rating::NotApplicable | Rating::Low => 0,
        Rating::Medium => 1,
        Rating::Unknown => 2,
        Rating::High => 3,
    }
}

fn no_worse(before: Rating, after: Rating) -> bool {
    rank(after) <= rank(before)
}

/// Applies `f` to every multisig in the tree (the outermost first).
fn map_multisigs(c: &Controller, f: &impl Fn(Controller) -> Controller) -> Controller {
    let mapped = match c {
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            signers,
            nested_signers,
        } => Controller::Multisig {
            kind: *kind,
            address: address.clone(),
            threshold: *threshold,
            voters: *voters,
            time_lock_secs: *time_lock_secs,
            config_authority: config_authority
                .as_ref()
                .map(|ca| Box::new(map_multisigs(ca, f))),
            signers: signers.clone(),
            nested_signers: nested_signers.clone(),
        },
        other => return other.clone(),
    };
    f(mapped)
}

/// Only multisigs that can already act: giving a 0-of-0 multisig its first
/// voter turns "nobody can act" into "one person can", a real increase in risk.
fn add_voter(c: Controller) -> Controller {
    match c {
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            signers,
            nested_signers,
        } if threshold >= 1 && threshold <= voters => Controller::Multisig {
            kind,
            address,
            threshold: threshold + 1,
            voters: voters + 1,
            time_lock_secs,
            config_authority,
            signers,
            nested_signers,
        },
        other => other,
    }
}

fn raise_time_lock(by: u32) -> impl Fn(Controller) -> Controller {
    move |c| match c {
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            signers,
            nested_signers,
        } => Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs: time_lock_secs.saturating_add(by),
            config_authority,
            signers,
            nested_signers,
        },
        other => other,
    }
}

fn transform(p: &ProtocolFacts, f: &impl Fn(Controller) -> Controller) -> ProtocolFacts {
    let mut q = p.clone();
    for prog in q.programs.values_mut() {
        prog.upgrade_control = fact(map_multisigs(prog.upgrade_control.value(), f));
    }
    for a in q.admin.values_mut() {
        a.controller = fact(map_multisigs(a.controller.value(), f));
    }
    q
}

fn assert_no_factor_worse(
    before: &ProtocolFacts,
    after: &ProtocolFacts,
) -> Result<(), TestCaseError> {
    let m = Methodology::bundled("0.2").unwrap();
    let (b, a) = (score_protocol(before, &m), score_protocol(after, &m));
    for f in Factor::ALL {
        let (rb, ra) = (b.factors[&f].rating, a.factors[&f].rating);
        prop_assert!(
            no_worse(rb, ra),
            "{f:?}: {rb:?} → {ra:?}\nbefore: {before:#?}\nafter: {after:#?}"
        );
    }
    prop_assert!(
        a.stage.stage >= b.stage.stage
            || b.stage.stage == celebtir_score::Stage::Unrated
                && a.stage.stage == celebtir_score::Stage::Unrated,
        "stage {:?} → {:?}",
        b.stage.stage,
        a.stage.stage
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn sco_8_adding_a_voter_never_worsens_a_rating(p in protocol_strategy()) {
        assert_no_factor_worse(&p, &transform(&p, &add_voter))?;
    }

    #[test]
    fn sco_8_raising_a_time_lock_never_worsens_a_rating(p in protocol_strategy(), by in 1u32..=30 * DAY) {
        assert_no_factor_worse(&p, &transform(&p, &raise_time_lock(by)))?;
    }

    #[test]
    fn sco_8_removing_an_admin_key_never_worsens_a_rating(p in protocol_with_admins(1..4), pick in any::<prop::sample::Index>()) {
        let mut q = p.clone();
        let key = q.admin.keys().nth(pick.index(q.admin.len())).unwrap().clone();
        q.admin.remove(&key);
        assert_no_factor_worse(&p, &q)?;
    }

    #[test]
    fn sco_8_scoring_is_deterministic(p in protocol_strategy()) {
        let m = Methodology::bundled("0.2").unwrap();
        let facts = FactsFile::new(BTreeMap::from([("p".to_string(), p)]));
        let a = serde_json::to_string(&score(&facts, &m)).unwrap();
        let b = serde_json::to_string(&score(&facts.clone(), &m)).unwrap();
        prop_assert_eq!(a, b);
    }
}

#[test]
fn sco_8_fixed_threshold_voter_can_correctly_lower_the_ratio() {
    // Documents ADR 014: 3 of 5 (60 %) is Low; 3 of 6 (50 %) is Medium.
    let m = Methodology::bundled("0.2").unwrap();
    let before = celebtir_score::factors::upgrade_control(&fact(multisig(3, 5, 0)), &m).rating;
    let after = celebtir_score::factors::upgrade_control(&fact(multisig(3, 6, 0)), &m).rating;
    assert_eq!((before, after), (Rating::Low, Rating::Medium));
    let _ = Address::new("unused");
}
