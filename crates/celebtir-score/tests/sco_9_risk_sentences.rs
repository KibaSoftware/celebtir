//! SCO-9: L2Beat-style risk sentences generated from facts, with severity and
//! a link to their factor. LCH-3: templates never use loaded words.

mod common;

use celebtir_core::Controller;
use celebtir_core::snapshot::{
    AdminPower, CodeTransparency, OracleRole, OracleSource, ProgramRole,
};
use celebtir_score::{
    Factor, Methodology, Rating, RiskCategory, Severity, score_protocol, sentences,
};
use common::*;

const CORE: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

fn worst_case() -> celebtir_core::snapshot::ProtocolFacts {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, single_key())]);
    p.programs.get_mut(&addr(CORE)).unwrap().code_transparency =
        fact(CodeTransparency::NoPublicSource);
    p.admin = admin(vec![
        (AdminPower::Funds, single_key()),
        (AdminPower::Pause, single_key()),
        (AdminPower::Parameters, single_key()),
    ]);
    p.oracles = oracles(vec![(
        "SOL",
        OracleRole::Primary,
        OracleSource::ProtocolRun,
    )]);
    p.dependencies = dependencies(vec![single_key()]);
    p
}

#[test]
fn sco_9_every_high_rating_produces_a_sentence() {
    let s = score_protocol(&worst_case(), &Methodology::bundled("0.2").unwrap());
    for f in Factor::ALL {
        assert_eq!(s.factors[&f].rating, Rating::High, "{f:?}");
        assert!(
            s.risks
                .iter()
                .any(|r| r.factor == f && r.severity == Severity::High),
            "no sentence for {f:?}: {:#?}",
            s.risks
        );
    }
}

#[test]
fn sco_9_low_ratings_produce_no_sentences() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, Controller::Immutable)]);
    p.oracles = oracles(vec![
        ("SOL", OracleRole::Primary, network("pyth")),
        ("SOL", OracleRole::Fallback, network("switchboard")),
    ]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert!(s.factors.values().all(|f| f.rating == Rating::Low));
    assert!(s.risks.is_empty(), "{:#?}", s.risks);
}

#[test]
fn sco_9_legacy_sentences_use_the_three_l2beat_openings() {
    let legacy = Methodology::from_toml(include_str!("../../../methodology/v0.1.toml")).unwrap();
    let s = score_protocol(&worst_case(), &legacy);
    for r in &s.risks {
        let opening = match r.category {
            RiskCategory::Stolen => "Funds can be stolen if ",
            RiskCategory::Frozen => "Funds can be frozen if ",
            RiskCategory::PriceManipulation => "Prices can be manipulated if ",
        };
        assert!(r.text.starts_with(opening), "{}", r.text);
        assert!(r.text.ends_with('.'), "{}", r.text);
        assert!(!r.text.contains('{'), "unfilled placeholder: {}", r.text);
    }
    let cats: Vec<_> = s.risks.iter().map(|r| r.category).collect();
    assert!(
        cats.contains(&RiskCategory::Frozen),
        "pause key should freeze funds"
    );
    assert!(cats.contains(&RiskCategory::PriceManipulation));
}

#[test]
fn sco_9_sentences_are_sorted_by_severity_and_name_the_holder() {
    let mut p = protocol(vec![(CORE, ProgramRole::Core, multisig(2, 4, 0))]);
    p.admin = admin(vec![(AdminPower::Funds, single_key())]);
    let s = score_protocol(&p, &Methodology::bundled("0.2").unwrap());
    assert_eq!(s.risks.first().unwrap().severity, Severity::High);
    assert_eq!(s.risks.last().unwrap().severity, Severity::Medium);
    let upgrade = s
        .risks
        .iter()
        .find(|r| r.factor == Factor::UpgradeControl)
        .unwrap();
    assert!(upgrade.text.contains("2 of 4"), "{}", upgrade.text);
}

#[test]
fn lch_3_templates_contain_no_loaded_words() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sentences.rs"),
    )
    .unwrap();
    assert_eq!(
        sentences::banned_word(&src),
        None,
        "src/sentences.rs uses a banned word"
    );
    for t in sentences::TEMPLATES {
        assert!(sentences::banned_word(t).is_none(), "{t}");
    }
}

#[test]
fn lch_3_banned_word_check_catches_each_word() {
    assert_eq!(
        sentences::banned_word("This protocol is UNSAFE"),
        Some("unsafe")
    );
    assert_eq!(sentences::banned_word("a scam"), Some("scam"));
    assert_eq!(sentences::banned_word("rug pull"), Some("rug"));
    assert_eq!(
        sentences::banned_word("Funds can be stolen if one key acts."),
        None
    );
}

#[test]
fn sco_9_lch_3_qa_v02_sentences_describe_configuration_without_withdrawal_claims() {
    let s = score_protocol(&worst_case(), &Methodology::bundled("0.2").unwrap());
    assert!(s.risks.iter().all(|r| !r.text.contains("stolen")
        && !r.text.contains("before users can withdraw")
        && !r.text.contains('{')));
    assert!(s.risks.iter().any(|r| {
        r.text
            .starts_with("An approved upgrade takes effect immediately.")
    }));
}
