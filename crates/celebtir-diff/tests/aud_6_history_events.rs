//! Launch audit F6: every ProgramData upgrade and authority change becomes
//! an on-chain event dated by its block, linked to its transaction, written
//! exactly once, including history recorded before such events existed.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, FactsFile, ProgramFacts, ProgramHistory, ProgramRole,
    ProtocolFacts, UpgradeEvent, UpgradeKind,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact};
use celebtir_diff::{EventKind, Reported, Severity, Snapshot, Source, diff_with, initial_events};
use celebtir_score::{Methodology, score};

const P: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

fn ev() -> Evidence {
    Evidence::new(Chain::Solana, Address::new(P), 1, "upgrade_authority")
}

fn tx(sig: &str, slot: u64, date: &str, kind: UpgradeKind) -> UpgradeEvent {
    UpgradeEvent {
        signature: sig.into(),
        slot,
        date: Some(date.into()),
        kind,
        new_authority: None,
    }
}

fn facts(history: Option<Vec<UpgradeEvent>>) -> FactsFile {
    let p = ProtocolFacts {
        chain: Chain::Solana,
        category: Some("lending".into()),
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: BTreeMap::from([(
            Address::new(P),
            ProgramFacts {
                role: ProgramRole::Core,
                loader: None,
                upgrade_control: Fact::single(Controller::Immutable, ev()),
                code_transparency: Fact::single(CodeTransparency::NoPublicSource, ev()),
            },
        )]),
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
            ev(),
        ),
        tokens: BTreeMap::new(),
        min_slot: 1,
        max_slot: 1,
    };
    let mut f = FactsFile::new(BTreeMap::from([("kamino-lend".to_string(), p)]));
    if let Some(events) = history {
        f.program_history.insert(
            Address::new(P),
            ProgramHistory {
                scanned: events.len() as u64,
                total: events.len() as u64,
                complete: true,
                events,
            },
        );
    }
    f
}

fn run(old: &FactsFile, new: &FactsFile, reported: &Reported) -> Vec<celebtir_diff::Event> {
    let m = Methodology::shipped();
    let (so, sn) = (score(old, &m), score(new, &m));
    diff_with(
        &Snapshot {
            date: "2026-10-21",
            facts: old,
            scores: &so,
        },
        &Snapshot {
            date: "2026-10-25",
            facts: new,
            scores: &sn,
        },
        reported,
    )
}

fn history() -> Vec<UpgradeEvent> {
    vec![
        tx("deploy", 10, "2025-01-01", UpgradeKind::Deploy),
        tx("old-upgrade", 20, "2026-10-19", UpgradeKind::Upgrade),
        tx("auth", 30, "2026-10-20", UpgradeKind::SetAuthority),
        tx("new-upgrade", 40, "2026-10-24", UpgradeKind::Upgrade),
        tx("gone", 50, "2026-10-24", UpgradeKind::RemoveAuthority),
    ]
}

#[test]
fn aud_6_history_already_recorded_is_backfilled_once_with_transaction_evidence() {
    let old = facts(Some(history()[..3].to_vec()));
    let new = facts(Some(history()));
    let events = run(&old, &new, &Reported::new());
    let chain: Vec<&celebtir_diff::Event> =
        events.iter().filter(|e| e.signature.is_some()).collect();
    // Deploy and extend are not changes of control; the other four are.
    assert_eq!(chain.len(), 4, "{chain:#?}");
    let by_sig = |s: &str| {
        chain
            .iter()
            .find(|e| e.signature.as_deref() == Some(s))
            .unwrap()
    };
    let old_up = by_sig("old-upgrade");
    assert_eq!(old_up.kind, EventKind::ProgramUpgraded);
    assert_eq!(old_up.source, Source::OnChain);
    assert!(old_up.backfill);
    assert_eq!(old_up.occurred_on.as_deref(), Some("2026-10-19"));
    assert_eq!(old_up.slot, Some(20));
    assert_eq!(old_up.detected_within_days, 6);
    assert_eq!(old_up.date, "2026-10-25");
    assert!(
        old_up.evidence[0]
            .explorer_url
            .as_str()
            .ends_with("/tx/old-upgrade")
    );
    assert_eq!(by_sig("auth").kind, EventKind::UpgradeAuthoritySet);
    assert_eq!(by_sig("auth").severity, Severity::Critical);
    let fresh = by_sig("new-upgrade");
    assert!(!fresh.backfill);
    assert_eq!(fresh.detected_within_days, 1);
    assert_eq!(by_sig("gone").kind, EventKind::UpgradeAuthorityRemoved);
}

#[test]
fn aud_6_bounded_or_missing_new_history_keeps_unreported_old_transactions() {
    let old = facts(Some(history()[..3].to_vec()));
    for latest in [Some(history()[3..].to_vec()), None] {
        let new = facts(latest);
        let events = run(&old, &new, &Reported::new());
        let old_upgrade = events
            .iter()
            .find(|e| e.signature.as_deref() == Some("old-upgrade"))
            .unwrap();
        assert!(old_upgrade.backfill);
        assert_eq!(old_upgrade.occurred_on.as_deref(), Some("2026-10-19"));
        assert_eq!(old_upgrade.detected_within_days, 6);
        let reported = events
            .iter()
            .filter_map(|e| Some((e.protocol.clone()?, e.signature.clone()?)))
            .collect();
        assert!(
            run(&old, &new, &reported)
                .iter()
                .all(|e| e.signature.is_none())
        );
    }
}

#[test]
fn aud_6_reported_signatures_are_never_repeated() {
    let old = facts(Some(history()));
    let new = facts(Some(history()));
    let reported: Reported = ["old-upgrade", "auth", "new-upgrade", "gone"]
        .iter()
        .map(|s| ("kamino-lend".to_string(), s.to_string()))
        .collect();
    let events = run(&old, &new, &reported);
    assert!(events.iter().all(|e| e.signature.is_none()), "{events:#?}");
}

#[test]
fn aud_6_a_history_seen_for_the_first_time_is_backfill() {
    let old = facts(None);
    let new = facts(Some(history()));
    let events = run(&old, &new, &Reported::new());
    let chain: Vec<_> = events.iter().filter(|e| e.signature.is_some()).collect();
    assert_eq!(chain.len(), 4);
    assert!(chain.iter().all(|e| e.backfill));
}

#[test]
fn aud_6_first_snapshot_reports_its_history() {
    let new = facts(Some(history()));
    let sn = score(&new, &Methodology::shipped());
    let events = initial_events(&Snapshot {
        date: "2026-10-25",
        facts: &new,
        scores: &sn,
    });
    assert_eq!(events.len(), 4);
    assert!(
        events
            .iter()
            .all(|e| e.backfill && e.protocol.as_deref() == Some("kamino-lend"))
    );
}

#[test]
fn aud_6_backfill_does_not_attribute_rating_changes_to_the_chain() {
    // A registry edit adds a program with no public source on the day old
    // history is backfilled: the rating change is the registry's, not the
    // chain's.
    let old = facts(Some(vec![]));
    let mut new = facts(Some(history()[..3].to_vec()));
    for p in new.protocols.values_mut() {
        let mut f = p.programs.values().next().unwrap().clone();
        f.code_transparency = Fact::single(CodeTransparency::NoPublicSource, ev());
        p.programs.insert(
            Address::new("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr"),
            f,
        );
    }
    let mut old = old;
    for p in old.protocols.values_mut() {
        for f in p.programs.values_mut() {
            f.code_transparency = Fact::single(
                CodeTransparency::PublicSourceUnverified {
                    repo: "https://github.com/a/b".into(),
                    commit: None,
                    on_chain_hash: None,
                },
                ev(),
            );
        }
    }
    let events = run(&old, &new, &Reported::new());
    assert!(events.iter().any(|e| e.backfill));
    let rating = events
        .iter()
        .find(|e| e.kind == EventKind::RatingChanged)
        .expect("code transparency changed");
    assert_eq!(rating.source, Source::RegistryEdit, "{rating:#?}");
}
