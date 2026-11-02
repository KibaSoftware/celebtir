//! `celebtir diff <old> <new>` writes events.json into the new snapshot folder.

mod common;

use std::path::Path;
use std::process::Command;

use celebtir_core::{Address, Controller, Fact, to_canonical_json};
use common::*;

fn celebtir(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        .output()
        .unwrap()
}

fn snapshot(dir: &Path, facts: &celebtir_core::snapshot::FactsFile) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("facts.json"), to_canonical_json(facts).unwrap()).unwrap();
    let o = celebtir(&["score", "--facts", dir.join("facts.json").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0));
}

#[test]
fn diff_writes_events_for_a_day_with_an_on_chain_change() {
    let root = tempfile::tempdir().unwrap();
    let (old, new) = (
        root.path().join("2026-10-16"),
        root.path().join("2026-10-17"),
    );
    snapshot(&old, &sample());
    let mut today = sample();
    let program = today
        .protocols
        .get_mut("sample-lend")
        .unwrap()
        .programs
        .get_mut(&Address::new("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"))
        .unwrap();
    let evidence = program.upgrade_control.evidence()[0].clone();
    program.upgrade_control = Fact::single(Controller::Immutable, evidence);
    snapshot(&new, &today);

    let o = celebtir(&["diff", old.to_str().unwrap(), new.to_str().unwrap()]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let events: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(new.join("events.json")).unwrap()).unwrap();
    assert_eq!(events["date"], "2026-10-17");
    assert_eq!(events["previous_date"], "2026-10-16");
    let kinds: Vec<&str> = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"made_immutable"), "{kinds:?}");

    let schema = celebtir_diff::events_schema();
    assert!(
        jsonschema::validator_for(&schema)
            .unwrap()
            .is_valid(&events)
    );
}

#[test]
fn diff_rejects_folders_not_named_by_date() {
    let root = tempfile::tempdir().unwrap();
    let o = celebtir(&[
        "diff",
        root.path().to_str().unwrap(),
        root.path().to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn chg_3_diff_reads_earlier_folders_to_confirm_changes() {
    use celebtir_core::snapshot::CodeTransparency;
    let root = tempfile::tempdir().unwrap();
    let klend = Address::new("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");
    for (day, verified) in [
        ("2026-10-14", true),
        ("2026-10-15", true),
        ("2026-10-16", false),
        ("2026-10-17", false),
    ] {
        let mut f = sample();
        let program = f
            .protocols
            .get_mut("sample-lend")
            .unwrap()
            .programs
            .get_mut(&klend)
            .unwrap();
        if !verified {
            let e = program.code_transparency.evidence()[0].clone();
            program.code_transparency = Fact::single(
                CodeTransparency::PublicSourceUnverified {
                    repo: "r".into(),
                    commit: None,
                    on_chain_hash: None,
                },
                e,
            );
        }
        snapshot(&root.path().join(day), &f);
    }
    let run = |old: &str, new: &str| {
        let o = celebtir(&[
            "diff",
            root.path().join(old).to_str().unwrap(),
            root.path().join(new).to_str().unwrap(),
        ]);
        assert_eq!(o.status.code(), Some(0));
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.path().join(new).join("events.json")).unwrap(),
        )
        .unwrap();
        v["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["kind"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert!(
        !run("2026-10-15", "2026-10-16").contains(&"verification_changed".to_string()),
        "first unverified day is noise"
    );
    assert!(
        run("2026-10-16", "2026-10-17").contains(&"verification_changed".to_string()),
        "second day confirms"
    );
}
