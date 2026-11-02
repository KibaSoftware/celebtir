//! HSCO-1: Safe and timelock controllers serialize, validate against the
//! published facts schema and round-trip (wave 2, ADR 024).

use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, FactsFile, ProgramFacts, ProgramRole, ProtocolFacts,
};
use celebtir_core::{
    Address, Chain, Controller, Evidence, Fact, MultisigKind, schema, to_canonical_json,
};

const PROXY: &str = "0x1111111111111111111111111111111111111111";

fn ev(field: &str) -> Evidence {
    Evidence::new(Chain::HyperEvm, Address::new(PROXY), 18_000_000, field)
}

fn safe() -> Controller {
    Controller::Multisig {
        kind: MultisigKind::Safe,
        address: Address::new("0x2222222222222222222222222222222222222222"),
        threshold: 3,
        voters: 5,
        time_lock_secs: 0,
        config_authority: None,
        signers: vec![],
        nested_signers: Default::default(),
    }
}

fn facts(admin: Option<Controller>) -> FactsFile {
    let controller = Controller::Timelock {
        address: Address::new("0x3333333333333333333333333333333333333333"),
        min_delay_secs: 172_800,
        proposer: Box::new(safe()),
        admin: admin.map(Box::new),
    };
    let protocol = ProtocolFacts {
        chain: Chain::HyperEvm,
        category: None,
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: [(
            Address::new(PROXY),
            ProgramFacts {
                role: ProgramRole::Core,
                loader: None,
                upgrade_control: Fact::single(controller, ev("eip1967.admin")),
                code_transparency: Fact::single(CodeTransparency::NoPublicSource, ev("source")),
            },
        )]
        .into_iter()
        .collect(),
        admin: Default::default(),
        oracles: Default::default(),
        admin_review: None,
        oracle_review: None,
        dependencies: Fact::single(
            Dependencies::Unavailable {
                reason: "declared, not traced".into(),
            },
            ev("registry"),
        ),
        tokens: Default::default(),
        min_slot: 18_000_000,
        max_slot: 18_000_000,
    };
    FactsFile::new([("evm-test".to_string(), protocol)].into_iter().collect())
}

#[test]
fn hsco_1_safe_and_timelock_validate_and_round_trip() {
    let validator = jsonschema::validator_for(&schema::facts_schema()).unwrap();
    for admin in [
        None,
        Some(Controller::SingleKey {
            key: Address::new(PROXY),
        }),
    ] {
        let file = facts(admin);
        let json = to_canonical_json(&file).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let errors: Vec<_> = validator
            .iter_errors(&value)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{errors:#?}");
        let back: FactsFile = serde_json::from_str(&json).unwrap();
        assert_eq!(to_canonical_json(&back).unwrap(), json);
    }
}

#[test]
fn hsco_1_timelock_without_admin_omits_the_field() {
    let json = to_canonical_json(&facts(None)).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let c = &v["protocols"]["evm-test"]["programs"][PROXY]["upgrade_control"]["value"];
    assert_eq!(c["min_delay_secs"], 172_800, "{c}");
    assert!(c.get("admin").is_none(), "{c}");
    assert_eq!(c["proposer"]["multisig_kind"], "safe", "{c}");
}
