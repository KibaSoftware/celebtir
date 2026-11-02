//! One day's Hyperliquid collection: every HyperEVM protocol in the
//! registry plus every HIP-3 DEX (HCOL-11, HOPS-1). A chain-wide failure
//! (the RPC is down) fails only Hyperliquid; one protocol's failure fails
//! only that protocol.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{Hip3Snapshot, ProtocolFacts};
use celebtir_registry::{EvmController, Hip3File, ProtocolEntry};
use collect_solana::rpc::{CallReport, RpcClient, RpcConfig, Transport};
use collect_solana::verify::HttpGet;
use serde_json::Value;

use crate::collect::{EvmOracles, collect_protocol};
use crate::evm::HyperEvm;
use crate::hip3;
use crate::resolve::{EvmKnown, Resolver};
use crate::verify::verified_source;

pub struct Inputs<'a> {
    pub protocols: &'a [ProtocolEntry],
    pub controllers: &'a [EvmController],
    pub oracles: &'a EvmOracles,
    pub hip3: &'a Hip3File,
    pub etherscan_key: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub protocols: BTreeMap<String, ProtocolFacts>,
    pub hip3: Hip3Snapshot,
    pub raw_perp_dexs: Option<Value>,
    /// `(scope, error)`, as in `run.json`.
    pub failures: Vec<(String, String)>,
    pub warnings: Vec<String>,
    /// The pinned HyperEVM block, when the RPC answered.
    pub block: Option<u64>,
    pub calls: CallReport,
    pub info_calls: u64,
}

pub async fn collect<E: Transport, I: Transport, H: HttpGet>(
    evm: E,
    info: &I,
    http: &H,
    config: RpcConfig,
    inputs: &Inputs<'_>,
) -> Output {
    let mut failures = Vec::new();
    let budget = config.daily_budget;
    let hyper = match HyperEvm::connect(RpcClient::new(evm, config)).await {
        Ok(h) => h,
        Err(e) => {
            // HCOL-11: every Hyperliquid protocol fails; Solana is untouched.
            failures.extend(
                inputs
                    .protocols
                    .iter()
                    .map(|p| (format!("protocol:{}", p.id), format!("HyperEVM RPC: {e}"))),
            );
            failures.push(("hip3".into(), format!("HyperEVM RPC: {e}")));
            return Output {
                protocols: BTreeMap::new(),
                hip3: Hip3Snapshot::Unavailable {
                    reason: "no HyperEVM block to anchor the HIP-3 reads to (ADR 024 D6)".into(),
                },
                raw_perp_dexs: None,
                failures,
                warnings: vec![],
                block: None,
                calls: CallReport::default(),
                info_calls: 0,
            };
        }
    };

    let timelocks: Vec<_> = inputs
        .protocols
        .iter()
        .flat_map(|p| p.timelocks.clone())
        .collect();
    let known = EvmKnown::new(inputs.controllers, &timelocks);
    let resolver = Resolver {
        evm: &hyper,
        known: &known,
    };

    let mut protocols = BTreeMap::new();
    for entry in inputs.protocols {
        let mut transparency = BTreeMap::new();
        for p in &entry.programs {
            let proxy = p.proxy.unwrap_or(celebtir_registry::ProxyKind::Eip1967);
            transparency.insert(
                p.id.clone(),
                verified_source(&hyper, http, inputs.etherscan_key, &p.id, proxy).await,
            );
        }
        let facts = collect_protocol(&resolver, inputs.oracles, entry, &transparency).await;
        protocols.insert(entry.id.clone(), facts);
    }
    // Reads past the budget come back Unavailable; say so in run.json.
    if hyper.rpc().report().total() >= budget {
        failures.push((
            "hyperliquid".into(),
            format!("HyperEVM budget of {budget} calls reached; later reads are Unavailable"),
        ));
    }

    let h = hip3::collect(info, hyper.block(), inputs.hip3).await;
    if let Hip3Snapshot::Unavailable { reason } = &h.snapshot {
        failures.push(("hip3".into(), reason.clone()));
    }
    Output {
        protocols,
        hip3: h.snapshot,
        raw_perp_dexs: h.raw,
        failures,
        warnings: h.warnings,
        block: Some(hyper.block()),
        calls: hyper.rpc().report(),
        info_calls: h.calls,
    }
}
