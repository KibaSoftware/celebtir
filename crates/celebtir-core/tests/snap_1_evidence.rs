//! SNAP-1: every fact carries at least one Evidence item; a fact without
//! evidence cannot be built or deserialized.

use celebtir_core::{Address, Chain, Controller, Evidence, Fact, FactError, MultisigKind};

fn evidence(field: &str) -> Evidence {
    Evidence::new(
        Chain::Solana,
        Address::new("BPFLoaderUpgradeab1e11111111111111111111111"),
        382_114_880,
        field,
    )
}

#[test]
fn snap_1_fact_without_evidence_is_an_error() {
    let err = Fact::new(Controller::Immutable, vec![]).unwrap_err();
    assert_eq!(err, FactError::NoEvidence);
}

#[test]
fn snap_1_fact_with_evidence_builds() {
    let fact = Fact::single(Controller::Immutable, evidence("upgrade_authority"));
    assert_eq!(fact.evidence().len(), 1);
    assert_eq!(fact.value(), &Controller::Immutable);
}

#[test]
fn snap_1_deserializing_fact_with_empty_evidence_fails() {
    let json = r#"{"value":{"kind":"immutable"},"evidence":[]}"#;
    let result: Result<Fact<Controller>, _> = serde_json::from_str(json);
    assert!(result.is_err(), "empty evidence must not deserialize");
}

#[test]
fn snap_1_deserializing_fact_without_evidence_field_fails() {
    let json = r#"{"value":{"kind":"immutable"}}"#;
    let result: Result<Fact<Controller>, _> = serde_json::from_str(json);
    assert!(result.is_err(), "missing evidence must not deserialize");
}

#[test]
fn snap_1_nested_controller_fact_round_trips() {
    let inner = Controller::SingleKey {
        key: Address::new("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
    };
    let multisig = Controller::Multisig {
        kind: MultisigKind::SquadsV4,
        address: Address::new("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"),
        threshold: 3,
        voters: 5,
        time_lock_secs: 0,
        config_authority: Some(Box::new(inner)),
        signers: vec![],
        nested_signers: Default::default(),
    };
    let fact = Fact::new(
        multisig,
        vec![evidence("upgrade_authority"), evidence("config_authority")],
    )
    .unwrap();

    let json = serde_json::to_string(&fact).unwrap();
    let back: Fact<Controller> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, fact);
    assert!(json.contains(r#""kind":"multisig""#));
}

#[test]
fn snap_1_evidence_has_a_solscan_link() {
    let e = evidence("upgrade_authority");
    assert_eq!(
        e.explorer_url.as_str(),
        "https://solscan.io/account/BPFLoaderUpgradeab1e11111111111111111111111"
    );
}
