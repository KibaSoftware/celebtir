//! Dependencies (COL-14): programs a protocol's core programs call directly,
//! found in a sample of recent finalized transactions. Stack heights tell a
//! direct call (one level below the core program) from a nested one.
//! Transactions are cached by signature between runs.

use std::collections::{BTreeMap, BTreeSet};

use celebtir_core::Address;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::program::DecodeError;
use crate::reads::SolanaRpc;
use crate::rpc::{RpcError, Transport};

/// System, SPL Token, Token-2022, compute budget and memo programs carry no
/// control risk of their own for this factor.
pub const DROPPED_PROGRAMS: &[&str] = &[
    "11111111111111111111111111111111",
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
    "ComputeBudget111111111111111111111111111111",
    "Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo",
    "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
];

/// Direct callees per transaction signature, kept between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxCache {
    pub by_signature: BTreeMap<String, BTreeSet<Address>>,
    /// ProgramData events per `programdata:signature` (VAL-4).
    #[serde(default)]
    pub upgrades: BTreeMap<String, Vec<celebtir_core::snapshot::UpgradeEvent>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencySample {
    /// Direct callees, minus dropped programs and the protocol's own programs.
    pub callees: BTreeSet<Address>,
    /// Transactions read (from the node or the cache).
    pub transactions: usize,
    pub newest_slot: Option<u64>,
}

fn bad(msg: &str) -> DecodeError {
    DecodeError(msg.to_string())
}

/// Every program `core` invokes directly in one transaction (`json` encoding).
pub fn direct_callees(tx: &Value, core: &Address) -> Result<BTreeSet<Address>, DecodeError> {
    let msg = &tx["transaction"]["message"];
    let mut keys: Vec<&str> = msg["accountKeys"]
        .as_array()
        .ok_or_else(|| bad("transaction has no account keys"))?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for list in ["writable", "readonly"] {
        if let Some(extra) = tx["meta"]["loadedAddresses"][list].as_array() {
            keys.extend(extra.iter().filter_map(Value::as_str));
        }
    }
    let program = |ix: &Value| -> Result<&str, DecodeError> {
        let i = ix["programIdIndex"]
            .as_u64()
            .ok_or_else(|| bad("instruction without programIdIndex"))? as usize;
        keys.get(i)
            .copied()
            .ok_or_else(|| bad("programIdIndex outside the account keys"))
    };

    let top = msg["instructions"]
        .as_array()
        .ok_or_else(|| bad("transaction has no instructions"))?;
    let inner: BTreeMap<u64, &Vec<Value>> = tx["meta"]["innerInstructions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|g| Some((g["index"].as_u64()?, g["instructions"].as_array()?)))
        .collect();

    let mut out = BTreeSet::new();
    for (i, ix) in top.iter().enumerate() {
        // stack[h] = program running at stack height h (top level is 1).
        let mut stack: Vec<&str> = vec!["", program(ix)?];
        for cpi in inner
            .get(&(i as u64))
            .map(|v| v.as_slice())
            .unwrap_or_default()
        {
            let h = cpi["stackHeight"]
                .as_u64()
                .ok_or_else(|| bad("inner instruction without stackHeight"))?
                as usize;
            if h < 2 || h > stack.len() {
                return Err(bad("inner instruction stack height out of order"));
            }
            let callee = program(cpi)?;
            stack.truncate(h);
            if stack[h - 1] == core.as_str() && callee != core.as_str() {
                out.insert(Address::new(callee));
            }
            stack.push(callee);
        }
    }
    Ok(out)
}

/// Samples the newest `limit` finalized transactions of each core program.
/// Failed and unreadable transactions are skipped; decoded ones are cached.
pub async fn sample_dependencies<T: Transport>(
    rpc: &SolanaRpc<T>,
    core: &[Address],
    own_programs: &[Address],
    cache: &mut TxCache,
    limit: usize,
) -> Result<DependencySample, RpcError> {
    let mut sample = DependencySample::default();
    // An explicitly disabled sample must not send an invalid RPC limit of zero.
    if limit == 0 {
        return Ok(sample);
    }
    for program in core {
        for sig in rpc.get_signatures_for_address(program, limit).await? {
            if sig.failed {
                continue;
            }
            let key = format!("{program}:{}", sig.signature);
            let callees = match cache.by_signature.get(&key) {
                Some(c) => c.clone(),
                None => {
                    let tx = match rpc.get_transaction(&sig.signature).await {
                        Ok(Some(tx)) => tx,
                        Ok(None) => continue,
                        // A transaction the node refuses (e.g. an unsupported
                        // version) is unreadable, not a protocol failure.
                        Err(RpcError::Rpc { .. } | RpcError::Malformed { .. }) => continue,
                        Err(e) => return Err(e),
                    };
                    let Ok(c) = direct_callees(&tx, program) else {
                        continue;
                    };
                    cache.by_signature.insert(key, c.clone());
                    c
                }
            };
            sample.transactions += 1;
            sample.newest_slot = sample.newest_slot.max(Some(sig.slot));
            sample.callees.extend(
                callees.into_iter().filter(|c| {
                    !DROPPED_PROGRAMS.contains(&c.as_str()) && !own_programs.contains(c)
                }),
            );
        }
    }
    Ok(sample)
}
