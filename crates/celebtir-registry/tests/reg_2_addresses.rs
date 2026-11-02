//! REG-2 (offline half): every address is valid base58 of 32 bytes. The
//! on-chain half (program IDs are executable accounts) is tested in
//! collect-solana's `reg_2_programs_on_chain`.

mod common;

use celebtir_registry::validate;
use common::{Registry, VALID};

fn errors(r: &Registry) -> Vec<String> {
    validate(r.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

#[test]
fn reg_2_typoed_program_id_fails_with_its_field() {
    let r = Registry::valid();
    // `0`, `O`, `I` and `l` are not base58.
    r.protocol(
        "kamino-lend",
        &VALID.replace(
            "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
            "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmj0",
        ),
    );
    let errs = errors(&r);
    assert!(
        errs.iter().any(|e| e.contains("kamino-lend.yaml")
            && e.contains("/programs/0/id")
            && e.contains("base58")),
        "{errs:#?}"
    );
}

#[test]
fn reg_2_wrong_length_address_fails() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replace(
            "6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM",
            "6hhBGCtmg7tPWUSgp3LG6X2rs",
        ),
    );
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("/multisigs/0/address") && e.contains("32 bytes")),
        "{errs:#?}"
    );
}

#[test]
fn reg_2_every_address_field_is_checked() {
    let r = Registry::valid();
    let yaml = VALID
        .replace("7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF", "bad-admin")
        .replace("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE", "bad-oracle");
    r.protocol("kamino-lend", &yaml);
    let errs = errors(&r);
    for field in ["/admin_accounts/0/account", "/oracles/0/account"] {
        assert!(errs.iter().any(|e| e.contains(field)), "{field}: {errs:#?}");
    }
}

#[test]
fn reg_2_valid_registry_still_passes() {
    assert!(errors(&Registry::valid()).is_empty());
}
