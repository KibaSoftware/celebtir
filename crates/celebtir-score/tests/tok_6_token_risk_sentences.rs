//! The token page's risk summary in L2Beat's style ("Your balance can be
//! frozen if…"), one sentence per power held, its severity from that power's
//! own rating. Low powers and renounced authorities produce none.

mod common;

use celebtir_core::Controller;
use celebtir_core::snapshot::{TokenExtensions, TokenProgram, TokenProvenance};
use celebtir_score::sentences::{self, Severity};
use celebtir_score::{Methodology, TokenFactor, score_token};
use common::*;

fn m() -> Methodology {
    Methodology::bundled("0.2").unwrap()
}

#[test]
fn tok_6_a_single_key_freeze_and_mint_authority_give_two_high_sentences() {
    let mut t = token();
    t.mint_authority = fact(single_key());
    t.freeze_authority = fact(single_key());
    let s = score_token(&t, &m());
    let texts: Vec<&str> = s.risks.iter().map(|r| r.text.as_str()).collect();
    assert!(
        texts
            .iter()
            .any(|x| x.starts_with("New supply can be minted if")),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|x| x.starts_with("Your balance can be frozen if")),
        "{texts:?}"
    );
    assert!(s.risks.iter().all(|r| r.severity == Severity::High));
    assert_eq!(s.risks[0].factor, TokenFactor::SupplyControl);
}

#[test]
fn tok_6_renounced_authorities_give_no_sentences() {
    let mut t = token();
    t.mint_authority = fact(Controller::Immutable);
    t.freeze_authority = fact(Controller::Immutable);
    assert!(score_token(&t, &m()).risks.is_empty());
}

#[test]
fn tok_6_a_multisig_freeze_authority_is_a_medium_sentence() {
    let mut t = token();
    t.mint_authority = fact(Controller::Immutable);
    t.freeze_authority = fact(multisig(3, 5, 0));
    let s = score_token(&t, &m());
    assert_eq!(s.risks.len(), 1);
    assert_eq!(s.risks[0].severity, Severity::Medium);
    assert!(s.risks[0].text.contains("3 of 5"), "{}", s.risks[0].text);
}

#[test]
fn tok_6_permanent_delegate_pause_and_bridge_each_have_a_sentence() {
    let mut t = token();
    t.mint_authority = fact(Controller::Immutable);
    t.freeze_authority = fact(Controller::Immutable);
    t.program = TokenProgram::Token2022;
    t.extensions = fact(TokenExtensions {
        permanent_delegate: Some(single_key()),
        pause_authority: Some(single_key()),
        ..TokenExtensions::default()
    });
    t.provenance = fact(TokenProvenance::Bridged {
        bridge: "Wormhole".into(),
        backing_chain: "ethereum".into(),
        backing_address: "0xabc".into(),
    });
    let texts: Vec<String> = score_token(&t, &m())
        .risks
        .into_iter()
        .map(|r| r.text)
        .collect();
    assert!(
        texts
            .iter()
            .any(|x| x.starts_with("Tokens can be moved from your wallet if")),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|x| x.starts_with("Transfers can be stopped if")),
        "{texts:?}"
    );
    assert!(texts.iter().any(|x| x.contains("Wormhole")), "{texts:?}");
}

#[test]
fn lch_3_token_sentences_use_no_loaded_words() {
    for t in sentences::TEMPLATES {
        assert!(sentences::banned_word(t).is_none(), "{t}");
    }
    assert!(
        sentences::TEMPLATES
            .iter()
            .any(|t| t.starts_with("Your balance can be frozen if"))
    );
}
