//! CHG-1 (events carry before/after values, evidence and the transaction
//! when ProgramData history provides one) and CHG-2 (events are tagged by
//! source; a registry fix never appears as an on-chain change).

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, FactsFile, ProgramFacts, ProgramHistory, ProgramRole,
    ProtocolFacts, UpgradeEvent, UpgradeKind,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact, MultisigKind};
use celebtir_diff::{EventKind, Severity, Snapshot, Source, diff};
use celebtir_score::{Methodology, SHIPPED_METHODOLOGY, score};

const P: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const Q: &str = "FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr";

fn ev(slot: u64) -> Evidence {
    Evidence::new(
        Chain::Solana,
        Address::new("9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA"),
        slot,
        "upgrade_authority",
    )
}

fn multisig(
    threshold: u16,
    voters: u16,
    time_lock_secs: u32,
    ca: Option<Controller>,
) -> Controller {
    Controller::Multisig {
        kind: MultisigKind::SquadsV4,
        address: Address::new("6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM"),
        threshold,
        voters,
        time_lock_secs,
        config_authority: ca.map(Box::new),
        signers: vec![],
        nested_signers: Default::default(),
    }
}

fn facts(programs: Vec<(&str, Controller)>, history: Vec<UpgradeEvent>) -> FactsFile {
    let p = ProtocolFacts {
        chain: Chain::Solana,
        category: Some("lending".into()),
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: programs
            .into_iter()
            .map(|(a, c)| {
                (
                    Address::new(a),
                    ProgramFacts {
                        role: ProgramRole::Core,
                        loader: None,
                        upgrade_control: Fact::single(c, ev(1)),
                        code_transparency: Fact::single(CodeTransparency::NoPublicSource, ev(1)),
                    },
                )
            })
            .collect(),
        admin: BTreeMap::new(),
        oracles: BTreeMap::new(),
        admin_review: None,
        oracle_review: None,
        dependencies: Fact::single(
            Dependencies::Sampled {
                callees: BTreeMap::new(),
                stale: false,
                declared: false,
            },
            ev(1),
        ),
        tokens: BTreeMap::new(),
        min_slot: 1,
        max_slot: 1,
    };
    let mut f = FactsFile::new(BTreeMap::from([("kamino-lend".to_string(), p)]));
    f.program_history.insert(
        Address::new(P),
        ProgramHistory {
            scanned: history.len() as u64,
            total: history.len() as u64,
            complete: true,
            events: history,
        },
    );
    f
}

fn upgrade(sig: &str, slot: u64) -> UpgradeEvent {
    UpgradeEvent {
        signature: sig.into(),
        slot,
        date: Some("2026-10-17".into()),
        kind: UpgradeKind::Upgrade,
        new_authority: None,
    }
}

fn run(old: &FactsFile, new: &FactsFile) -> Vec<celebtir_diff::Event> {
    run_with(old, new, &Methodology::shipped(), &Methodology::shipped())
}

fn run_with(
    old: &FactsFile,
    new: &FactsFile,
    mo: &Methodology,
    mn: &Methodology,
) -> Vec<celebtir_diff::Event> {
    let (so, sn) = (score(old, mo), score(new, mn));
    diff(
        &Snapshot {
            date: "2026-10-16",
            facts: old,
            scores: &so,
        },
        &Snapshot {
            date: "2026-10-17",
            facts: new,
            scores: &sn,
        },
    )
}

fn of_kind(events: &[celebtir_diff::Event], k: EventKind) -> Vec<&celebtir_diff::Event> {
    events.iter().filter(|e| e.kind == k).collect()
}

#[test]
fn chg_1_upgrade_event_links_to_its_transaction() {
    let old = facts(
        vec![(P, multisig(5, 10, 86_400, None))],
        vec![upgrade("old", 100)],
    );
    let new = facts(
        vec![(P, multisig(5, 10, 86_400, None))],
        vec![upgrade("old", 100), upgrade("3tTU214kgv", 200)],
    );
    let events = run(&old, &new);
    let up = of_kind(&events, EventKind::ProgramUpgraded);
    assert_eq!(up.len(), 1, "{events:#?}");
    assert_eq!(up[0].signature.as_deref(), Some("3tTU214kgv"));
    assert_eq!(up[0].source, Source::OnChain);
    assert_eq!(up[0].severity, Severity::Warning);
}

#[test]
fn chg_1_controller_change_carries_before_after_and_evidence() {
    let key = Controller::SingleKey {
        key: Address::new("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
    };
    let events = run(
        &facts(vec![(P, multisig(5, 10, 0, None))], vec![]),
        &facts(vec![(P, key)], vec![]),
    );
    let c = of_kind(&events, EventKind::ControllerChanged);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].severity, Severity::Critical);
    assert_eq!(c[0].before["kind"], "multisig");
    assert_eq!(c[0].after["kind"], "single_key");
    assert!(!c[0].evidence.is_empty());
    assert!(
        !of_kind(&events, EventKind::RatingChanged).is_empty(),
        "the rating follows"
    );
}

#[test]
fn chg_1_removing_the_authority_is_a_positive_made_immutable_event() {
    let events = run(
        &facts(vec![(P, multisig(5, 10, 0, None))], vec![]),
        &facts(vec![(P, Controller::Immutable)], vec![]),
    );
    let m = of_kind(&events, EventKind::MadeImmutable);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].severity, Severity::Positive);
}

#[test]
fn chg_1_multisig_parameter_changes_are_warnings_or_critical_with_a_config_authority() {
    let events = run(
        &facts(vec![(P, multisig(5, 10, 86_400, None))], vec![]),
        &facts(vec![(P, multisig(4, 10, 0, None))], vec![]),
    );
    let m = of_kind(&events, EventKind::MultisigChanged);
    assert_eq!((m.len(), m[0].severity), (1, Severity::Warning));

    let ca = Controller::SingleKey {
        key: Address::new("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
    };
    let events = run(
        &facts(vec![(P, multisig(5, 10, 0, None))], vec![]),
        &facts(vec![(P, multisig(5, 10, 0, Some(ca)))], vec![]),
    );
    assert_eq!(
        of_kind(&events, EventKind::MultisigChanged)[0].severity,
        Severity::Critical
    );
}

#[test]
fn chg_2_a_registry_fix_never_appears_as_an_on_chain_change() {
    // The registry gains a second program; nothing changed on chain.
    let old = facts(vec![(P, multisig(5, 10, 0, None))], vec![]);
    let new = facts(
        vec![(P, multisig(5, 10, 0, None)), (Q, Controller::Immutable)],
        vec![],
    );
    let events = run(&old, &new);
    assert!(!events.is_empty());
    for e in &events {
        assert_ne!(e.source, Source::OnChain, "{e:#?}");
    }
    assert_eq!(of_kind(&events, EventKind::RegistryEdit).len(), 1);
}

#[test]
fn chg_2_unreadable_controllers_are_not_reported_as_changes() {
    let unavailable = Controller::Unavailable {
        reason: "rpc".into(),
    };
    let events = run(
        &facts(vec![(P, multisig(5, 10, 0, None))], vec![]),
        &facts(vec![(P, unavailable)], vec![]),
    );
    assert!(of_kind(&events, EventKind::ControllerChanged).is_empty());
}

#[test]
fn chg_2_methodology_changes_tag_the_rating_changes() {
    let f = facts(vec![(P, multisig(5, 10, 6 * 86_400, None))], vec![]);
    let strict = Methodology::from_toml(
        &SHIPPED_METHODOLOGY.replace("low_min_secs = 432000", "low_min_secs = 604800"),
    )
    .unwrap();
    let mut strict = strict;
    strict.version = "test-stricter".into();
    let events = run_with(&f, &f, &Methodology::shipped(), &strict);
    assert_eq!(of_kind(&events, EventKind::MethodologyChange).len(), 1);
    let r = of_kind(&events, EventKind::RatingChanged);
    assert!(!r.is_empty());
    assert!(r.iter().all(|e| e.source == Source::Methodology));
}

#[test]
fn chg_4_event_ids_are_stable_and_sorted() {
    let old = facts(vec![(P, multisig(5, 10, 0, None))], vec![]);
    let new = facts(vec![(P, Controller::Immutable)], vec![]);
    let a = run(&old, &new);
    let b = run(&old, &new);
    assert_eq!(a, b);
    assert!(a.windows(2).all(|w| w[0].id < w[1].id));
    assert!(
        a.iter()
            .any(|e| e.id == format!("2026-10-17:kamino-lend:made_immutable:{P}"))
    );
}

#[test]
fn chg_1_first_history_is_backfill_not_news() {
    let mut old = facts(vec![(P, multisig(5, 10, 0, None))], vec![]);
    old.program_history.clear();
    let new = facts(
        vec![(P, multisig(5, 10, 0, None))],
        vec![upgrade("a", 1), upgrade("b", 2)],
    );
    assert!(of_kind(&run(&old, &new), EventKind::ProgramUpgraded).is_empty());
}

/// CHG-5: skipping two days yields events with a 3-day detection window.
#[test]
fn chg_5_events_after_missed_runs_carry_the_detection_window() {
    let old = facts(vec![(P, multisig(5, 10, 0, None))], vec![]);
    let new = facts(vec![(P, Controller::Immutable)], vec![]);
    let (so, sn) = (
        score(&old, &Methodology::shipped()),
        score(&new, &Methodology::shipped()),
    );
    let events = diff(
        &Snapshot {
            date: "2026-10-14",
            facts: &old,
            scores: &so,
        },
        &Snapshot {
            date: "2026-10-17",
            facts: &new,
            scores: &sn,
        },
    );
    assert!(!events.is_empty());
    assert!(
        events.iter().all(|e| e.detected_within_days == 3),
        "{events:#?}"
    );
    // A normal day-to-day diff has a window of 1.
    assert!(run(&old, &new).iter().all(|e| e.detected_within_days == 1));
}

#[test]
fn chg_3_a_signer_swap_alone_raises_no_event() {
    let with = |signer: &str| {
        let Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            ..
        } = multisig(5, 10, 86_400, None)
        else {
            unreachable!()
        };
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            signers: vec![Address::new(signer)],
            nested_signers: Default::default(),
        }
    };
    let old = facts(
        vec![(P, with("SignerA1111111111111111111111111111111111111"))],
        vec![],
    );
    let new = facts(
        vec![(P, with("SignerB1111111111111111111111111111111111111"))],
        vec![],
    );
    assert!(run(&old, &new).is_empty(), "{:?}", run(&old, &new));
}
