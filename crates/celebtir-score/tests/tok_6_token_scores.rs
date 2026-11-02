//! TOK-6 and SCO-10: tokens get Low / Medium / High on four factors with the
//! same engine, no Stage, keyed by mint in scores.json.

mod common;

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{FactsFile, TokenExtensions, TokenProgram, TokenProvenance};
use celebtir_score::{Methodology, Rating, TokenFactor, score, score_token};
use common::*;

fn m() -> Methodology {
    Methodology::bundled("0.2").unwrap()
}

fn rating(t: &celebtir_core::snapshot::TokenFacts, f: TokenFactor) -> Rating {
    score_token(t, &m()).factors[&f].rating
}

fn with_ext(ext: TokenExtensions) -> celebtir_core::snapshot::TokenFacts {
    let mut t = token();
    t.program = TokenProgram::Token2022;
    t.extensions = fact(ext);
    t
}

#[test]
fn tok_6_every_token_gets_four_ratings_with_reason_next_step_and_evidence() {
    let s = score_token(&token(), &m());
    assert_eq!(s.factors.len(), 4);
    for (f, fs) in &s.factors {
        assert!(
            !fs.reason.is_empty() && !fs.next_step.is_empty() && !fs.evidence.is_empty(),
            "{f:?}"
        );
    }
}

#[test]
fn tok_6_supply_control_levels() {
    let mut t = token();
    assert_eq!(rating(&t, TokenFactor::SupplyControl), Rating::Low);
    t.mint_authority = fact(multisig(5, 8, 0));
    assert_eq!(rating(&t, TokenFactor::SupplyControl), Rating::Medium);
    t.mint_authority = fact(single_key());
    assert_eq!(rating(&t, TokenFactor::SupplyControl), Rating::High);
}

#[test]
fn tok_6_renounced_mint_with_freeze_kept_rates_supply_low_and_freeze_separately() {
    let mut t = token();
    t.freeze_authority = fact(single_key());
    assert_eq!(rating(&t, TokenFactor::SupplyControl), Rating::Low);
    assert_eq!(rating(&t, TokenFactor::FreezeAndSeize), Rating::High);
}

#[test]
fn tok_6_freeze_and_seize_levels() {
    let mut t = token();
    assert_eq!(rating(&t, TokenFactor::FreezeAndSeize), Rating::Low);
    t.freeze_authority = fact(multisig(3, 5, 0));
    assert_eq!(rating(&t, TokenFactor::FreezeAndSeize), Rating::Medium);
    let t = with_ext(TokenExtensions {
        permanent_delegate: Some(multisig(5, 8, 0)),
        ..Default::default()
    });
    assert_eq!(
        rating(&t, TokenFactor::FreezeAndSeize),
        Rating::High,
        "a permanent delegate can seize from any wallet"
    );
}

#[test]
fn tok_6_transfer_restriction_levels() {
    assert_eq!(
        rating(&token(), TokenFactor::TransferRestrictions),
        Rating::Low
    );
    let fee = with_ext(TokenExtensions {
        transfer_fee_authority: Some(multisig(3, 5, 0)),
        ..Default::default()
    });
    assert_eq!(
        rating(&fee, TokenFactor::TransferRestrictions),
        Rating::Medium
    );
    let safe_hook = with_ext(TokenExtensions {
        transfer_hook: Some(hook(Controller::Immutable, Controller::Immutable)),
        ..Default::default()
    });
    assert_eq!(
        rating(&safe_hook, TokenFactor::TransferRestrictions),
        Rating::Medium
    );
    let risky_hook = with_ext(TokenExtensions {
        transfer_hook: Some(hook(Controller::Immutable, single_key())),
        ..Default::default()
    });
    assert_eq!(
        rating(&risky_hook, TokenFactor::TransferRestrictions),
        Rating::High,
        "hook program upgradeable by one key"
    );
}

#[test]
fn tok_6_unknown_extension_is_unknown_not_ignored() {
    let t = with_ext(TokenExtensions {
        unknown: vec![99],
        ..Default::default()
    });
    let s = score_token(&t, &m());
    let fs = &s.factors[&TokenFactor::TransferRestrictions];
    assert_eq!(fs.rating, Rating::Unknown);
    assert!(fs.reason.contains("99"), "{}", fs.reason);
}

#[test]
fn tok_6_provenance_levels() {
    assert_eq!(rating(&token(), TokenFactor::Provenance), Rating::Low);
    let mut t = token();
    t.provenance = fact(TokenProvenance::Bridged {
        bridge: "wormhole".into(),
        backing_chain: "ethereum".into(),
        backing_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".into(),
    });
    let s = score_token(&t, &m());
    assert_eq!(s.factors[&TokenFactor::Provenance].rating, Rating::Medium);
    assert!(
        s.factors[&TokenFactor::Provenance]
            .reason
            .contains("wormhole")
    );
}

#[test]
fn sco_10_token_scores_appear_under_their_mint_without_a_stage() {
    let mint = addr("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let mut facts = FactsFile::new(BTreeMap::new());
    facts.tokens.insert(mint.clone(), token());
    let scores = score(&facts, &m());
    let t = &scores.tokens[&mint];
    assert_eq!(t.symbol, "TKN");
    let json = serde_json::to_value(t).unwrap();
    assert!(json.get("stage").is_none());
    assert_eq!(json["factors"].as_object().unwrap().len(), 4);
}

#[test]
fn tok_6_hook_extension_without_a_program_is_medium_when_someone_can_set_one() {
    let mut h = hook(multisig(3, 5, 0), Controller::Immutable);
    h.program = None;
    h.program_controller = None;
    let t = with_ext(TokenExtensions {
        transfer_hook: Some(h.clone()),
        ..Default::default()
    });
    assert_eq!(
        rating(&t, TokenFactor::TransferRestrictions),
        Rating::Medium
    );
    h.authority = Controller::Immutable;
    let t = with_ext(TokenExtensions {
        transfer_hook: Some(h),
        ..Default::default()
    });
    assert_eq!(rating(&t, TokenFactor::TransferRestrictions), Rating::Low);
}
