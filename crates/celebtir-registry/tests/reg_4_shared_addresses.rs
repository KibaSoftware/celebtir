//! REG-4: an address may appear in two protocols only if both list it under
//! `shared_with`.

mod common;

use celebtir_registry::validate;
use common::{Registry, VALID};

const SHARED: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

/// A second protocol, `fork-lend`, that deploys nothing of Kamino's except
/// what each test adds.
fn fork() -> String {
    VALID
        .replace("id: kamino-lend", "id: fork-lend")
        .replace("name: Kamino Lend", "name: Fork Lend")
        .replace(SHARED, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
        .replace(
            "FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr",
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
        )
        .replace(
            "6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM",
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        )
        .replace(
            "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
            "SysvarRent111111111111111111111111111111111",
        )
}

fn with_shared_program(yaml: &str, shared_with: Option<&str>) -> String {
    let extra = match shared_with {
        Some(other) => format!("    shared_with: [{other}]\n"),
        None => String::new(),
    };
    yaml.replace(
        "programs:\n",
        &format!(
            "programs:\n  - id: {SHARED}\n    name: Shared lending\n    role: core\n    source: https://example.com/shared\n{extra}"
        ),
    )
}

fn errors(r: &Registry) -> Vec<String> {
    validate(r.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

#[test]
fn reg_4_duplicate_address_without_shared_with_fails() {
    let r = Registry::valid();
    r.protocol("fork-lend", &with_shared_program(&fork(), None));
    let errs = errors(&r);
    assert!(
        errs.iter().any(|e| e.contains("fork-lend.yaml")
            && e.contains(SHARED)
            && e.contains("kamino-lend")),
        "{errs:#?}"
    );
    assert!(
        errs.iter()
            .any(|e| e.contains("kamino-lend.yaml") && e.contains("fork-lend")),
        "{errs:#?}"
    );
}

#[test]
fn reg_4_duplicate_listed_on_one_side_only_fails() {
    let r = Registry::valid();
    r.protocol(
        "fork-lend",
        &with_shared_program(&fork(), Some("kamino-lend")),
    );
    let errs = errors(&r);
    assert_eq!(errs.len(), 1, "{errs:#?}");
    assert!(errs[0].contains("kamino-lend.yaml") && errs[0].contains("shared_with"));
}

#[test]
fn reg_4_duplicate_listed_on_both_sides_passes() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replace(
            "    source: https://github.com/Kamino-Finance/klend\n  - id: Farms",
            "    source: https://github.com/Kamino-Finance/klend\n    shared_with: [fork-lend]\n  - id: Farms",
        ),
    );
    r.protocol(
        "fork-lend",
        &with_shared_program(&fork(), Some("kamino-lend")),
    );
    let errs = errors(&r);
    assert!(errs.is_empty(), "{errs:#?}");
}

#[test]
fn reg_4_shared_with_a_protocol_that_does_not_list_the_address_fails() {
    let r = Registry::valid();
    r.protocol("fork-lend", &fork());
    r.protocol(
        "kamino-lend",
        &VALID.replace(
            "    role: periphery\n",
            "    role: periphery\n    shared_with: [fork-lend]\n",
        ),
    );
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("kamino-lend.yaml") && e.contains("/programs/1/shared_with")),
        "{errs:#?}"
    );
}

#[test]
fn reg_4_shared_multisig_needs_shared_with() {
    let r = Registry::valid();
    r.protocol(
        "fork-lend",
        &fork().replace(
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            "6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM",
        ),
    );
    let errs = errors(&r);
    assert!(
        errs.iter().any(|e| e.contains("/multisigs/0")
            && e.contains("6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM")),
        "{errs:#?}"
    );
}

/// ADR 010: oracle price feeds are shared infrastructure and exempt.
#[test]
fn reg_4_shared_oracle_feed_is_allowed() {
    let r = Registry::valid();
    let mut fork = fork();
    assert!(fork.contains("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"));
    fork = fork.replace("market: SOL reserve", "market: SOL market");
    r.protocol("fork-lend", &fork);
    let errs = errors(&r);
    assert!(errs.is_empty(), "{errs:#?}");
}
