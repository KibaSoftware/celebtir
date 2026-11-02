//! CHG-3 (noise control: dependency events need 3 consecutive runs,
//! verification changes need 2) and CHG-4 (events are never rewritten).

use std::collections::BTreeMap;
use std::process::Command;

use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, DependencyFact, FactsFile, ProgramFacts, ProgramRole,
    ProtocolFacts,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact};
use celebtir_diff::{EventKind, Snapshot, confirmed_events};
use celebtir_score::{Methodology, ScoresFile, score};

const P: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const CALLEE: &str = "Ca11ee1111111111111111111111111111111111111";

fn ev() -> Evidence {
    Evidence::new(Chain::Solana, Address::new(P), 1, "x")
}

fn facts(callee: bool, verified: bool) -> FactsFile {
    let callees = if callee {
        BTreeMap::from([(
            Address::new(CALLEE),
            DependencyFact {
                upgrade_control: Fact::single(Controller::Immutable, ev()),
            },
        )])
    } else {
        BTreeMap::new()
    };
    let transparency = if verified {
        CodeTransparency::Verified {
            repo: "r".into(),
            commit: "c".into(),
            on_chain_hash: "h".into(),
            executable_hash: "h".into(),
            warning: None,
        }
    } else {
        CodeTransparency::PublicSourceUnverified {
            repo: "r".into(),
            commit: None,
            on_chain_hash: None,
        }
    };
    let p = ProtocolFacts {
        chain: Chain::Solana,
        category: None,
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
                code_transparency: Fact::single(transparency, ev()),
            },
        )]),
        admin: BTreeMap::new(),
        oracles: BTreeMap::new(),
        admin_review: None,
        oracle_review: None,
        dependencies: Fact::single(
            Dependencies::Sampled {
                callees,
                stale: false,
                declared: false,
            },
            ev(),
        ),
        tokens: BTreeMap::new(),
        min_slot: 1,
        max_slot: 1,
    };
    FactsFile::new(BTreeMap::from([("p".to_string(), p)]))
}

/// Events on the last day of a window of (callee seen, verified) days.
fn last_day(days: &[(bool, bool)]) -> Vec<EventKind> {
    let facts: Vec<FactsFile> = days.iter().map(|(c, v)| facts(*c, *v)).collect();
    let scores: Vec<ScoresFile> = facts
        .iter()
        .map(|f| score(f, &Methodology::shipped()))
        .collect();
    let dates: Vec<String> = (0..days.len())
        .map(|i| format!("2026-10-{:02}", 10 + i))
        .collect();
    let window: Vec<Snapshot<'_>> = (0..days.len())
        .map(|i| Snapshot {
            date: &dates[i],
            facts: &facts[i],
            scores: &scores[i],
        })
        .collect();
    confirmed_events(&window)
        .into_iter()
        .map(|e| e.kind)
        .collect()
}

#[test]
fn chg_3_a_dependency_seen_in_one_sample_only_produces_no_event() {
    assert!(last_day(&[(false, true), (false, true), (true, true)]).is_empty());
    assert!(last_day(&[(false, true), (true, true), (false, true)]).is_empty());
}

#[test]
fn chg_3_a_dependency_seen_three_runs_in_a_row_is_added_once() {
    assert_eq!(
        last_day(&[(false, true), (true, true), (true, true), (true, true)]),
        vec![EventKind::DependencyAdded]
    );
    // The fourth consecutive day does not repeat it.
    assert!(last_day(&[(true, true), (true, true), (true, true), (true, true)]).is_empty());
}

#[test]
fn chg_3_a_dependency_gone_three_runs_in_a_row_is_removed() {
    assert_eq!(
        last_day(&[(true, true), (false, true), (false, true), (false, true)]),
        vec![EventKind::DependencyRemoved]
    );
}

#[test]
fn chg_3_verification_changes_need_two_runs() {
    assert!(
        last_day(&[(false, true), (false, false)]).is_empty(),
        "one run is noise"
    );
    assert_eq!(
        last_day(&[(false, true), (false, false), (false, false)]),
        vec![EventKind::VerificationChanged]
    );
    assert_eq!(
        last_day(&[(false, false), (false, true), (false, true)]),
        vec![EventKind::VerificationChanged]
    );
}

#[test]
fn chg_3_short_windows_never_confirm() {
    assert!(last_day(&[(true, false)]).is_empty());
    assert!(last_day(&[(true, false), (true, false)]).is_empty());
}

/// CHG-4: the CI script fails when a committed events.json is modified.
#[test]
fn chg_4_rewriting_history_fails_the_append_only_check() {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/check-events-append-only.sh");
    let repo = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "t"]);
    let day = repo.path().join("data/snapshots/2026-10-16");
    std::fs::create_dir_all(&day).unwrap();
    std::fs::write(day.join("events.json"), "{\"events\":[]}\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-qm", "day 1"]);
    git(&["tag", "base"]);

    let check = || {
        Command::new("bash")
            .arg(&script)
            .arg("base")
            .current_dir(repo.path())
            .output()
            .unwrap()
    };

    // Adding a new day is fine.
    let day2 = repo.path().join("data/snapshots/2026-10-17");
    std::fs::create_dir_all(&day2).unwrap();
    std::fs::write(day2.join("events.json"), "{\"events\":[]}\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-qm", "day 2"]);
    assert!(
        check().status.success(),
        "{}",
        String::from_utf8_lossy(&check().stdout)
    );

    // Editing an earlier day is not.
    std::fs::write(day.join("events.json"), "{\"events\":[{\"id\":\"x\"}]}\n").unwrap();
    git(&["commit", "-qam", "rewrite"]);
    let o = check();
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("2026-10-16/events.json"));
}
