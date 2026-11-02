//! TOK-7: a protocol lists the tokens it depends on with their worst rating,
//! and is flagged when it accepts a High-rated token as collateral.

mod common;

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{FactsFile, ProgramRole, TokenExtensions, TokenProgram, TokenRole};
use celebtir_score::{Methodology, Rating, score};
use common::*;

const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const PYUSD: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

fn facts(roles: Vec<(&str, Vec<TokenRole>)>) -> FactsFile {
    let mut p = protocol(vec![(
        "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
        ProgramRole::Core,
        Controller::Immutable,
    )]);
    p.tokens = roles.into_iter().map(|(m, r)| (addr(m), r)).collect();
    let mut f = FactsFile::new(BTreeMap::from([("lend".to_string(), p)]));
    f.tokens.insert(addr(USDC), token());
    let mut delegated = token();
    delegated.symbol = "PYUSD".into();
    delegated.program = TokenProgram::Token2022;
    delegated.extensions = fact(TokenExtensions {
        permanent_delegate: Some(multisig(3, 5, 0)),
        ..Default::default()
    });
    f.tokens.insert(addr(PYUSD), delegated);
    f
}

#[test]
fn tok_7_protocol_lists_tokens_with_their_worst_rating() {
    let s = score(
        &facts(vec![
            (USDC, vec![TokenRole::Deposit]),
            (PYUSD, vec![TokenRole::Borrowable]),
        ]),
        &Methodology::bundled("0.2").unwrap(),
    );
    let exposure = &s.protocols["lend"].tokens;
    assert_eq!(exposure.len(), 2);
    assert_eq!(exposure[&addr(USDC)].worst, Rating::Low);
    assert_eq!(exposure[&addr(PYUSD)].worst, Rating::High);
    assert_eq!(exposure[&addr(PYUSD)].symbol, "PYUSD");
    assert!(
        exposure[&addr(PYUSD)].warning.is_none(),
        "borrowable only, not collateral"
    );
}

#[test]
fn tok_7_accepting_a_token_with_a_permanent_delegate_as_collateral_warns() {
    let s = score(
        &facts(vec![(
            PYUSD,
            vec![TokenRole::Collateral, TokenRole::Borrowable],
        )]),
        &Methodology::bundled("0.2").unwrap(),
    );
    let w = s.protocols["lend"].tokens[&addr(PYUSD)]
        .warning
        .as_deref()
        .expect("a warning");
    assert!(
        w.contains("PYUSD") && w.contains("collateral") && w.contains("permanent delegate"),
        "{w}"
    );
}

#[test]
fn tok_7_unscored_token_shows_unknown() {
    let s = score(
        &facts(vec![(
            "So11111111111111111111111111111111111111112",
            vec![TokenRole::Deposit],
        )]),
        &Methodology::bundled("0.2").unwrap(),
    );
    let e = &s.protocols["lend"].tokens[&addr("So11111111111111111111111111111111111111112")];
    assert_eq!(e.worst, Rating::Unknown);
}
