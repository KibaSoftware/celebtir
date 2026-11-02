//! HCOL-11 (a chain-wide failure fails only Hyperliquid) and HOPS-1 (one
//! run collects HyperEVM protocols and HIP-3 DEXs, with per-chain counts),
//! replayed from the captured fixtures.

use std::path::Path;

use celebtir_core::snapshot::Hip3Snapshot;
use celebtir_core::{Controller, RpcEndpoint};
use celebtir_registry::{Hip3File, ProtocolEntry};
use collect_hyperliquid::collect::EvmOracles;
use collect_hyperliquid::evm::default_config;
use collect_hyperliquid::fixtures::{EvmFixture, InfoFixture, InfoReplay};
use collect_hyperliquid::pipeline::{Inputs, collect};
use collect_hyperliquid::resolve::EvmKnown;
use collect_solana::rpc::{Transport, TransportError};
use collect_solana::verify::HttpGet;
use serde_json::{Value, json};

fn repo(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn info(name: &str) -> InfoFixture {
    InfoFixture::load(&repo(&format!("fixtures/hyperliquid/info/{name}.json"))).unwrap()
}

fn hip3() -> Hip3File {
    serde_norway::from_str(
        &std::fs::read_to_string(repo("registry/hyperliquid/hip3.yaml")).unwrap(),
    )
    .unwrap()
}

fn kinetiq() -> ProtocolEntry {
    serde_norway::from_str(
        r#"
id: kinetiq-khype
name: Kinetiq kHYPE
chain: hyper_evm
category: liquid-staking
website: https://kinetiq.xyz
programs:
  - id: "0x393D0B87Ed38fc779FD9611144aE649BA6082109"
    name: StakingManager
    role: core
    proxy: eip1967
    source: https://github.com/DefiLlama/DefiLlama-Adapters/blob/main/projects/kinetiq/index.js
review:
  reviewed_by: test
  last_reviewed: 2026-10-21
"#,
    )
    .unwrap()
}

struct Down;

impl Transport for Down {
    async fn send(&self, _: Value) -> Result<Value, TransportError> {
        Err(TransportError::Status(503))
    }
}

/// Sourcify answers "not verified" for everything here.
struct NoSource;

impl HttpGet for NoSource {
    async fn get_json(&self, _: &str) -> Result<Value, TransportError> {
        Ok(json!({"match": null}))
    }
}

/// Serves the recorded calls, plus Sourcify's implementation-slot read,
/// which the resolver capture did not need.
fn evm_replay() -> EvmFixture {
    let mut f =
        EvmFixture::load(&repo("fixtures/hyperliquid/evm/kinetiq-khype-manager.json")).unwrap();
    f.calls.push(collect_hyperliquid::fixtures::EvmCall {
        method: "eth_getStorageAt".into(),
        params: json!([
            "0x393D0B87Ed38fc779FD9611144aE649BA6082109",
            format!("0x{}", collect_hyperliquid::resolve::IMPLEMENTATION_SLOT),
            format!("0x{:x}", f.block)
        ]),
        result: Some(json!(
            "0x00000000000000000000000023fda3b9f944dbeaf2dcd5e7b9f8207b102c933f"
        )),
        error: None,
    });
    f
}

fn controllers() -> Vec<celebtir_registry::EvmController> {
    EvmKnown::controllers_from_yaml(
        &std::fs::read_to_string(repo("registry/controllers.yaml")).unwrap(),
    )
    .unwrap()
}

#[tokio::test(start_paused = true)]
async fn hops_1_one_run_collects_hyperevm_and_hip3() {
    let protocols = [kinetiq()];
    let c = controllers();
    let inputs = Inputs {
        protocols: &protocols,
        controllers: &c,
        oracles: &EvmOracles::default(),
        hip3: &hip3(),
        etherscan_key: None,
    };
    let replay = InfoReplay(vec![info("perp-dexs"), info("delegator-summary-xyz")]);
    let out = collect(
        evm_replay().replay(),
        &replay,
        &NoSource,
        default_config(),
        &inputs,
    )
    .await;
    let p = &out.protocols["kinetiq-khype"];
    let control = p.programs.values().next().unwrap().upgrade_control.value();
    assert!(
        matches!(
            control,
            Controller::Multisig {
                threshold: 4,
                voters: 8,
                ..
            }
        ),
        "{control:?}"
    );
    assert!(matches!(&out.hip3, Hip3Snapshot::Listed { dexs } if dexs.len() == 10));
    assert!(out.raw_perp_dexs.is_some());
    assert!(out.block.is_some());
    assert!(out.calls.total() > 0 && out.info_calls > 1);
    assert!(out.failures.is_empty(), "{:?}", out.failures);
    // The endpoint helper reduces any URL to its host (SNAP-5).
    assert_eq!(
        RpcEndpoint::new("https://rpc.hyperliquid.xyz/evm").host(),
        "rpc.hyperliquid.xyz"
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_11_rpc_down_fails_only_hyperliquid() {
    let protocols = [kinetiq()];
    let c = controllers();
    let inputs = Inputs {
        protocols: &protocols,
        controllers: &c,
        oracles: &EvmOracles::default(),
        hip3: &hip3(),
        etherscan_key: None,
    };
    let out = collect(Down, &Down, &NoSource, default_config(), &inputs).await;
    assert!(out.protocols.is_empty());
    assert!(matches!(out.hip3, Hip3Snapshot::Unavailable { .. }));
    assert!(out.block.is_none());
    let scopes: Vec<&str> = out.failures.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(scopes, ["protocol:kinetiq-khype", "hip3"]);
}
