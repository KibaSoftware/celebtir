//! HIP-3 perp DEXs from the info API (HIP-1 to HIP-5, ADR 024 D6).
//!
//! One `perpDexs` call lists every DEX; one `delegatorSummary` per distinct
//! deployer reads its stake. Facts are anchored to the HyperEVM block read
//! just before. A failed or unparseable `perpDexs` makes the whole section
//! Unavailable and never fails the run.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{Hip3DexFacts, Hip3Snapshot, Hip3Stake};
use celebtir_core::{Address, Chain, Evidence, Fact};
use celebtir_registry::{Hip3File, evm::eip55};
use collect_solana::rpc::Transport;
use serde_json::{Value, json};

/// What one run of HIP-3 collection produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Hip3Output {
    pub snapshot: Hip3Snapshot,
    /// The raw `perpDexs` response, kept per snapshot (HIP-2).
    pub raw: Option<Value>,
    /// `hip3.yaml` names that the API did not return (HREG-4).
    pub warnings: Vec<String>,
    pub calls: u64,
}

fn address(v: &Value) -> Option<Address> {
    v.as_str().and_then(eip55).map(Address::new)
}

fn pairs(v: &Value) -> BTreeMap<String, String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| Some((p[0].as_str()?.to_string(), p[1].as_str()?.to_string())))
        .collect()
}

/// A decimal string compared as a number; None if unparseable.
fn amount(v: &Value) -> Option<f64> {
    v.as_str().and_then(|s| s.parse::<f64>().ok())
}

fn stake(summary: Result<&Value, &str>, required: &str) -> Hip3Stake {
    let v = match summary {
        Ok(v) => v,
        Err(e) => {
            return Hip3Stake::Unavailable {
                reason: e.to_string(),
            };
        }
    };
    let (Some(delegated), Some(req)) = (amount(&v["delegated"]), required.parse::<f64>().ok())
    else {
        return Hip3Stake::Unavailable {
            reason: "delegatorSummary has no `delegated` amount".into(),
        };
    };
    let pending = amount(&v["totalPendingWithdrawal"]).unwrap_or(0.0);
    let text = |k: &str| v[k].as_str().unwrap_or("0").to_string();
    Hip3Stake::Read {
        delegated: text("delegated"),
        undelegated: text("undelegated"),
        pending_withdrawal: text("totalPendingWithdrawal"),
        required: required.to_string(),
        stake_below_required: delegated < req,
        unstaking: pending > 0.0 || v["nPendingWithdrawals"].as_u64().unwrap_or(0) > 0,
    }
}

/// HIP-1, HIP-3, HIP-4: parse `perpDexs` and the deployers' stake.
pub fn parse(
    response: &Value,
    anchor_block: u64,
    stakes: &BTreeMap<Address, Result<Value, String>>,
    registry: &Hip3File,
) -> Result<BTreeMap<String, Hip3DexFacts>, String> {
    let list = response
        .as_array()
        .ok_or("perpDexs did not return a list")?;
    let mut out = BTreeMap::new();
    // Entry 0 is the main Hyperliquid DEX (null); it is not HIP-3.
    for d in list.iter().filter(|d| !d.is_null()) {
        let name = d["name"]
            .as_str()
            .ok_or("a DEX without a name")?
            .to_string();
        let deployer = address(&d["deployer"]).ok_or(format!("{name}: no deployer"))?;
        let ev = |field: &str| {
            Evidence::new(
                Chain::HyperCore,
                deployer.clone(),
                anchor_block,
                format!("info.perpDexs[{name}].{field}"),
            )
        };
        let mut subs: BTreeMap<String, Vec<Address>> = BTreeMap::new();
        for g in d["subDeployers"].as_array().into_iter().flatten() {
            let (Some(action), Some(list)) = (g[0].as_str(), g[1].as_array()) else {
                continue;
            };
            let mut a: Vec<Address> = list.iter().filter_map(address).collect();
            a.sort();
            subs.insert(action.to_string(), a);
        }
        let stake = stake(
            stakes
                .get(&deployer)
                .map(|r| r.as_ref().map_err(String::as_str))
                .unwrap_or(Err("stake not read")),
            &registry.required_stake,
        );
        out.insert(
            name.clone(),
            Hip3DexFacts {
                full_name: d["fullName"].as_str().unwrap_or(&name).to_string(),
                deployer: Fact::single(deployer.clone(), ev("deployer")),
                oracle_updater: Fact::single(address(&d["oracleUpdater"]), ev("oracleUpdater")),
                sub_deployers: Fact::single(subs, ev("subDeployers")),
                fee_recipient: Fact::single(address(&d["feeRecipient"]), ev("feeRecipient")),
                oi_caps: Fact::single(
                    pairs(&d["assetToStreamingOiCap"]),
                    ev("assetToStreamingOiCap"),
                ),
                funding_multipliers: Fact::single(
                    pairs(&d["assetToFundingMultiplier"]),
                    ev("assetToFundingMultiplier"),
                ),
                stake: Fact::single(
                    stake,
                    Evidence::new(
                        Chain::HyperCore,
                        deployer.clone(),
                        anchor_block,
                        "info.delegatorSummary",
                    ),
                ),
            },
        );
    }
    Ok(out)
}

/// HIP-1 to HIP-5 against a live or replayed info API.
pub async fn collect<T: Transport>(info: &T, anchor_block: u64, registry: &Hip3File) -> Hip3Output {
    let mut calls = 1;
    let response = match info.send(json!({"type": "perpDexs"})).await {
        Ok(v) => v,
        Err(e) => {
            return Hip3Output {
                snapshot: Hip3Snapshot::Unavailable {
                    reason: format!("perpDexs: {e}"),
                },
                raw: None,
                warnings: vec![],
                calls,
            };
        }
    };
    let mut deployers: Vec<Address> = response
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|d| address(&d["deployer"]))
        .collect();
    deployers.sort();
    deployers.dedup();
    let mut stakes = BTreeMap::new();
    for d in deployers {
        calls += 1;
        let r = info
            .send(json!({"type": "delegatorSummary", "user": d.as_str().to_ascii_lowercase()}))
            .await
            .map_err(|e| format!("delegatorSummary: {e}"));
        stakes.insert(d, r);
    }
    match parse(&response, anchor_block, &stakes, registry) {
        Ok(dexs) => {
            let warnings = registry
                .dexs
                .keys()
                .filter(|n| !dexs.contains_key(*n))
                .map(|n| format!("hip3.yaml lists `{n}`, which perpDexs did not return"))
                .collect();
            Hip3Output {
                snapshot: Hip3Snapshot::Listed { dexs },
                raw: Some(response),
                warnings,
                calls,
            }
        }
        Err(reason) => Hip3Output {
            snapshot: Hip3Snapshot::Unavailable {
                reason: format!("perpDexs: {reason}"),
            },
            raw: Some(response),
            warnings: vec![],
            calls,
        },
    }
}
