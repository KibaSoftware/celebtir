//! TOK-8: token changes (new authority, extension enabled, authority
//! renounced) appear as change-log events.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    FactsFile, TokenExtensions, TokenFacts, TokenProgram, TokenProvenance,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact};
use celebtir_diff::{EventKind, Severity, Snapshot, diff};
use celebtir_score::{Methodology, score};

const MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

fn key(s: &str) -> Controller {
    Controller::SingleKey {
        key: Address::new(s),
    }
}

fn token(freeze: Controller, ext: TokenExtensions) -> FactsFile {
    let e = Evidence::new(Chain::Solana, Address::new(MINT), 1, "mint");
    let mut f = FactsFile::new(BTreeMap::new());
    f.tokens.insert(
        Address::new(MINT),
        TokenFacts {
            symbol: "USDC".into(),
            program: TokenProgram::Token2022,
            supply: 1_000_000,
            decimals: 6,
            mint_authority: Fact::single(
                key("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG"),
                e.clone(),
            ),
            freeze_authority: Fact::single(freeze, e.clone()),
            extensions: Fact::single(ext, e.clone()),
            provenance: Fact::single(TokenProvenance::Native, e),
            identity: None,
        },
    );
    f
}

fn events(old: &FactsFile, new: &FactsFile) -> Vec<celebtir_diff::Event> {
    let (so, sn) = (
        score(old, &Methodology::shipped()),
        score(new, &Methodology::shipped()),
    );
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
    .into_iter()
    .filter(|e| e.kind == EventKind::TokenControlChanged)
    .collect()
}

#[test]
fn tok_8_changing_the_freeze_authority_yields_an_event() {
    let old = token(
        key("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"),
        TokenExtensions::default(),
    );
    let new = token(
        key("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
        TokenExtensions::default(),
    );
    let e = events(&old, &new);
    assert_eq!(e.len(), 1, "{e:#?}");
    assert_eq!(e[0].severity, Severity::Warning);
    assert!(e[0].subject.ends_with(":freeze_authority"));
    assert!(e[0].summary.contains("USDC"));
}

#[test]
fn tok_8_renouncing_an_authority_is_positive() {
    let old = token(
        key("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"),
        TokenExtensions::default(),
    );
    let new = token(Controller::Immutable, TokenExtensions::default());
    assert_eq!(events(&old, &new)[0].severity, Severity::Positive);
}

#[test]
fn tok_8_a_new_permanent_delegate_is_critical() {
    let freeze = key("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar");
    let old = token(freeze.clone(), TokenExtensions::default());
    let new = token(
        freeze,
        TokenExtensions {
            permanent_delegate: Some(key("2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk")),
            ..Default::default()
        },
    );
    let e = events(&old, &new);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].severity, Severity::Critical);
}

#[test]
fn tok_8_an_enabled_extension_is_an_event() {
    let freeze = key("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar");
    let old = token(freeze.clone(), TokenExtensions::default());
    let new = token(
        freeze,
        TokenExtensions {
            default_frozen: true,
            other: vec!["cpi_guard".into()],
            ..Default::default()
        },
    );
    let subjects: Vec<String> = events(&old, &new).into_iter().map(|e| e.subject).collect();
    assert!(
        subjects.iter().any(|s| s.ends_with(":default_frozen")),
        "{subjects:?}"
    );
    assert!(
        subjects.iter().any(|s| s.ends_with(":extensions")),
        "{subjects:?}"
    );
}

#[test]
fn tok_8_unchanged_tokens_are_quiet() {
    let t = token(
        key("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"),
        TokenExtensions::default(),
    );
    assert!(events(&t, &t).is_empty());
}
