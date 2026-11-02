mod common;

use celebtir_core::snapshot::FactsFile;
use celebtir_core::to_canonical_json;
use celebtir_score::value::{ControlClass, control_class_with_methodology};
use celebtir_score::{Factor, Methodology, Rating, Stage, score, score_protocol};
use common::*;

#[test]
fn sco_3_tok_6_qa_program_control_rates_the_verified_program() {
    use celebtir_core::Controller;
    let current = Methodology::bundled("0.2").unwrap();
    for (upgrade, expected) in [
        (Controller::Immutable, Rating::Low),
        (multisig(6, 10, DAY), Rating::Low),
        (single_key(), Rating::High),
    ] {
        let c = Controller::ProgramControlled {
            key: addr("authority"),
            program: addr("program"),
            upgrade: Box::new(upgrade),
        };
        assert_eq!(
            celebtir_score::factors::upgrade_control(&fact(c), &current).rating,
            expected
        );
    }
}

#[test]
fn sco_2_qa_v02_delay_boundary_and_legacy_replay() {
    let current = Methodology::bundled("0.2").unwrap();
    let legacy = Methodology::from_toml(include_str!("../../../methodology/v0.1.toml")).unwrap();
    assert_eq!(current.version, "0.2");
    for delay in [0, 600, DAY - 1, DAY, DAY + 1] {
        let c = multisig(6, 10, delay);
        assert_eq!(
            control_class_with_methodology(&c, &current),
            if delay >= DAY {
                ControlClass::MultisigWithDelay
            } else {
                ControlClass::MultisigWithoutDelay
            }
        );
        assert_eq!(
            control_class_with_methodology(&c, &legacy),
            if delay > 0 {
                ControlClass::MultisigWithDelay
            } else {
                ControlClass::MultisigWithoutDelay
            }
        );
    }
    let facts: FactsFile =
        serde_json::from_str(include_str!("../../../fixtures/golden/expected/facts.json")).unwrap();
    let actual = to_canonical_json(&score(&facts, &legacy)).unwrap();
    assert_eq!(
        actual.trim_end(),
        include_str!("../../../fixtures/golden/expected/scores.json").trim_end()
    );
}

#[test]
fn sco_7_qa_missing_required_oracles_are_unknown_only_in_v02() {
    let facts: FactsFile =
        serde_json::from_str(include_str!("../../../fixtures/golden/expected/facts.json")).unwrap();
    let mut p = facts.protocols.values().next().unwrap().clone();
    p.oracles.clear();
    let current = Methodology::bundled("0.2").unwrap();
    let legacy = Methodology::from_toml(include_str!("../../../methodology/v0.1.toml")).unwrap();
    for category in ["lending", "perps"] {
        p.category = Some(category.into());
        let result = score_protocol(&p, &current);
        assert_eq!(
            result.factors[&Factor::OracleDependency].rating,
            Rating::Unknown
        );
        assert_eq!(result.stage.stage, Stage::Stage0);
        assert_eq!(
            score_protocol(&p, &legacy).factors[&Factor::OracleDependency].rating,
            Rating::NotApplicable
        );
    }
    p.category = Some("liquid-staking".into());
    assert_eq!(
        score_protocol(&p, &current).factors[&Factor::OracleDependency].rating,
        Rating::NotApplicable
    );
}

#[test]
fn tok_6_qa_v02_every_elevated_factor_has_a_reason_in_risks() {
    let mut t = token();
    t.extensions = fact(celebtir_core::snapshot::TokenExtensions {
        default_frozen: true,
        ..Default::default()
    });
    let s = celebtir_score::score_token(&t, &Methodology::bundled("0.2").unwrap());
    for (factor, finding) in &s.factors {
        if matches!(finding.rating, Rating::High | Rating::Medium) {
            assert!(
                s.risks
                    .iter()
                    .any(|r| r.factor == *factor && !r.text.is_empty())
            );
        }
    }
}
