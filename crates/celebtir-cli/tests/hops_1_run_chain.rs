//! HOPS-1: run.json carries a `hyperliquid` section when both chains are
//! collected, and it validates against the published run schema.

use std::collections::BTreeMap;

use celebtir_core::schema;
use celebtir_core::snapshot::{RUN_SCHEMA_VERSION, RunChain, RunFile};

fn run(hyperliquid: Option<RunChain>) -> serde_json::Value {
    serde_json::to_value(RunFile {
        schema_version: RUN_SCHEMA_VERSION.into(),
        tool_version: "0.1.0".into(),
        started_at: "2026-10-21T02:17:00Z".into(),
        finished_at: "2026-10-21T02:20:00Z".into(),
        duration_ms: 1,
        rpc_host: "api.mainnet-beta.solana.com".into(),
        calls: BTreeMap::new(),
        retries: 0,
        budget: 5_000,
        budget_exceeded: false,
        protocols: vec![],
        failures: vec![],
        review_needed: vec![],
        hyperliquid,
    })
    .unwrap()
}

#[test]
fn hops_1_run_json_has_a_hyperliquid_section() {
    let v = run(Some(RunChain {
        rpc_host: "rpc.hyperliquid.xyz".into(),
        block: Some(49_146_200),
        calls: BTreeMap::from([("eth_call".to_string(), 12)]),
        retries: 0,
        budget: 2_000,
        info_calls: 11,
        protocols: vec!["kinetiq-khype".into()],
        hip3_dexs: 10,
        failures: 0,
    }));
    assert_eq!(v["hyperliquid"]["hip3_dexs"], 10);
    let validator = jsonschema::validator_for(&schema::run_schema()).unwrap();
    let errors: Vec<_> = validator.iter_errors(&v).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn hops_1_solana_only_run_json_has_no_hyperliquid_section() {
    assert!(run(None).get("hyperliquid").is_none());
}
