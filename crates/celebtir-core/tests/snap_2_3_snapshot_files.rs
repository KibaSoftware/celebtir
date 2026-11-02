//! SNAP-2 (deterministic output) and SNAP-3 (schema_version + published JSON
//! Schema, no field removed without a major version bump).

use std::path::PathBuf;

use celebtir_core::schema::{self, CompatError};
use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, FACTS_SCHEMA_VERSION, FactsFile, ProgramFacts, ProgramRole,
    ProtocolFacts,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact, to_canonical_json};
use serde_json::json;

fn program(addr: &str, controller: Controller) -> (Address, ProgramFacts) {
    let address = Address::new(addr);
    let evidence = Evidence::new(Chain::Solana, address.clone(), 100, "upgrade_authority");
    (
        address,
        ProgramFacts {
            role: ProgramRole::Core,
            loader: None,
            upgrade_control: Fact::single(controller, evidence.clone()),
            code_transparency: Fact::single(CodeTransparency::NoPublicSource, evidence),
        },
    )
}

fn protocol(programs: Vec<(Address, ProgramFacts)>) -> ProtocolFacts {
    ProtocolFacts {
        chain: Chain::Solana,
        category: None,
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: programs.into_iter().collect(),
        admin: Default::default(),
        oracles: Default::default(),
        admin_review: None,
        oracle_review: None,
        dependencies: Fact::single(
            Dependencies::Unavailable {
                reason: "not sampled".into(),
            },
            Evidence::new(
                Chain::Solana,
                Address::new("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
                100,
                "transactions",
            ),
        ),
        tokens: Default::default(),
        min_slot: 100,
        max_slot: 100,
    }
}

fn sample(reverse: bool) -> FactsFile {
    let mut protocols = vec![
        (
            "kamino-lend".to_string(),
            protocol(vec![
                program(
                    "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
                    Controller::Immutable,
                ),
                program(
                    "FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr",
                    Controller::SingleKey {
                        key: Address::new("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
                    },
                ),
            ]),
        ),
        (
            "drift".to_string(),
            protocol(vec![program(
                "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH",
                Controller::Immutable,
            )]),
        ),
    ];
    if reverse {
        protocols.reverse();
        for (_, p) in &mut protocols {
            let mut v: Vec<_> = std::mem::take(&mut p.programs).into_iter().collect();
            v.reverse();
            p.programs = v.into_iter().collect();
        }
    }
    FactsFile::new(protocols.into_iter().collect())
}

fn schemas_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas")
}

#[test]
fn snap_2_identical_inputs_produce_byte_identical_facts_json() {
    let a = to_canonical_json(&sample(false)).unwrap();
    let b = to_canonical_json(&sample(true)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn snap_2_object_keys_are_sorted_at_every_level() {
    #[derive(serde::Serialize)]
    struct Unsorted {
        zeta: u8,
        alpha: Inner,
    }
    #[derive(serde::Serialize)]
    struct Inner {
        y: u8,
        b: u8,
    }
    let out = to_canonical_json(&Unsorted {
        zeta: 1,
        alpha: Inner { y: 2, b: 3 },
    })
    .unwrap();
    let alpha = out.find("\"alpha\"").unwrap();
    let zeta = out.find("\"zeta\"").unwrap();
    let b = out.find("\"b\"").unwrap();
    let y = out.find("\"y\"").unwrap();
    assert!(alpha < zeta && b < y, "keys not sorted:\n{out}");
    assert!(out.ends_with('\n'));
}

#[test]
fn snap_2_facts_schema_has_no_timestamp_fields() {
    fn names(node: &serde_json::Value, out: &mut Vec<String>) {
        match node {
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::Object(props)) = map.get("properties") {
                    out.extend(props.keys().cloned());
                }
                map.values().for_each(|v| names(v, out));
            }
            serde_json::Value::Array(items) => items.iter().for_each(|v| names(v, out)),
            _ => {}
        }
    }
    // The allowed dates are source data, identical for identical inputs, not
    // run timestamps: the calendar day of a DefiLlama data point, and the
    // block date of a ProgramData transaction.
    let mut facts = schema::facts_schema();
    for def in ["DailyUsd", "UpgradeEvent"] {
        let removed = facts["$defs"][def]["properties"]
            .as_object_mut()
            .and_then(|p| p.remove("date"));
        assert!(
            removed.is_some(),
            "allowlisted {def}.date moved; review this exception"
        );
    }
    let mut props = Vec::new();
    names(&facts, &mut props);
    assert!(
        props.iter().any(|p| p == "schema_version"),
        "walker found no properties"
    );
    for p in props {
        let p = p.to_lowercase();
        let banned = p.contains("timestamp") || p.ends_with("_at") || p == "date" || p == "time";
        assert!(!banned, "facts schema has a timestamp-like field: {p}");
    }
}

#[test]
fn snap_3_facts_file_declares_schema_version() {
    let out = to_canonical_json(&sample(false)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["schema_version"], FACTS_SCHEMA_VERSION);
}

#[test]
fn snap_3_facts_output_validates_against_published_schema() {
    let schema = schema::facts_schema();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let instance: serde_json::Value =
        serde_json::from_str(&to_canonical_json(&sample(false)).unwrap()).unwrap();
    let errors: Vec<_> = validator
        .iter_errors(&instance)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");

    let mut broken = instance.clone();
    broken["protocols"]["drift"]["programs"]["dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH"]["upgrade_control"]
        ["evidence"] = json!([]);
    assert!(
        !validator.is_valid(&broken),
        "empty evidence must fail the schema"
    );
}

/// Regenerate with `CELEBTIR_UPDATE_SCHEMAS=1 cargo test -p celebtir-core`.
#[test]
fn snap_3_committed_schemas_match_the_code() {
    for (name, generated) in schema::all() {
        let path = schemas_dir().join(format!("{name}.schema.json"));
        let generated = to_canonical_json(&generated).unwrap();
        if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
            std::fs::write(&path, &generated).unwrap();
        }
        let committed = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "{} missing; run with CELEBTIR_UPDATE_SCHEMAS=1",
                path.display()
            )
        });
        assert_eq!(
            committed, generated,
            "{name} schema is stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
        );
    }
}

/// Baselines are frozen copies of a schema at the release of a major version.
/// They live in `schemas/baseline/<name>.v<major>.schema.json`.
#[test]
fn snap_3_published_schemas_are_compatible_with_their_baselines() {
    for (name, generated) in schema::all() {
        let major = schema::major(FACTS_SCHEMA_VERSION);
        let path = schemas_dir().join(format!("baseline/{name}.v{major}.schema.json"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let baseline: serde_json::Value = serde_json::from_str(&text).unwrap();
        schema::check_compatible(&baseline, &generated).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn snap_3_removing_a_field_without_major_bump_fails() {
    let baseline = json!({"type":"object","properties":{"a":{"type":"string"},"b":{"type":"object","properties":{"c":{}}}}});
    let current = json!({"type":"object","properties":{"a":{"type":"string"},"b":{"type":"object","properties":{}}}});
    let err = schema::check_compatible(&baseline, &current).unwrap_err();
    assert_eq!(err, CompatError::Removed(vec!["/b/c".to_string()]));
}

#[test]
fn snap_3_adding_a_field_is_compatible() {
    let baseline = json!({"type":"object","properties":{"a":{}}});
    let current = json!({"type":"object","properties":{"a":{},"new":{}}});
    assert!(schema::check_compatible(&baseline, &current).is_ok());
}
