//! QA-1: every captured fixture is well formed, carries its slot and source,
//! and never a full RPC URL.

use collect_solana::fixtures::{AccountFixture, accounts_dir, load};

fn all() -> Vec<AccountFixture> {
    let mut names: Vec<String> = std::fs::read_dir(accounts_dir())
        .unwrap()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_str()?
                .strip_suffix(".json")
                .map(str::to_string)
        })
        .collect();
    names.sort();
    names.iter().map(|n| load(n)).collect()
}

#[test]
fn qa_1_every_fixture_carries_slot_source_and_matching_name() {
    let fixtures = all();
    assert!(
        fixtures.len() >= 7,
        "expected the captured program fixtures"
    );
    for f in fixtures {
        assert!(f.slot > 0, "{}: no slot", f.name);
        assert!(!f.note.is_empty(), "{}: say why it was captured", f.name);
        assert!(
            !f.source.contains("://"),
            "{}: source must be a host, not a URL",
            f.name
        );
        assert!(
            celebtir_core::budget::find_secrets(&f.source).is_none(),
            "{}",
            f.name
        );
        if let Some((_, len)) = f.data_slice {
            assert!(
                f.data().len() <= len,
                "{}: slice longer than requested",
                f.name
            );
        }
        let reread = f.read();
        assert_eq!(reread.slot, f.slot);
        assert_eq!(reread.account.is_some(), f.owner.is_some());
    }
}

#[test]
fn qa_1_fixture_names_match_files() {
    for entry in std::fs::read_dir(accounts_dir()).unwrap().flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        let stem = file
            .strip_suffix(".json")
            .expect("only .json files in fixtures/accounts");
        assert_eq!(load(stem).name, stem);
    }
}
