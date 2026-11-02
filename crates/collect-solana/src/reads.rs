//! Solana read methods over the RPC client. Every read uses `finalized`
//! commitment and returns the slot the node answered at (COL-3); accounts are
//! read in batches of up to 100 (COL-4). `getProgramAccounts` is never used.

use std::collections::BTreeMap;

use base64::Engine;
use celebtir_core::Address;
use serde_json::{Value, json};

use crate::rpc::{RpcClient, RpcError, Transport};

/// `getMultipleAccounts` accepts at most 100 keys.
pub const MAX_ACCOUNTS_PER_CALL: usize = 100;

const COMMITMENT: &str = "finalized";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAccount {
    pub owner: Address,
    pub lamports: u64,
    pub executable: bool,
    /// The whole account, or only the requested slice.
    pub data: Vec<u8>,
}

/// One account as read: `None` when the account does not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRead {
    pub address: Address,
    pub slot: u64,
    pub account: Option<RawAccount>,
}

/// Lowest and highest slot of a protocol's reads (COL-3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SlotRange(Option<(u64, u64)>);

impl SlotRange {
    pub fn record(&mut self, slot: u64) {
        self.0 = Some(match self.0 {
            None => (slot, slot),
            Some((lo, hi)) => (lo.min(slot), hi.max(slot)),
        });
    }

    pub fn get(&self) -> Option<(u64, u64)> {
        self.0
    }
}

pub struct SolanaRpc<T> {
    client: RpcClient<T>,
}

impl<T: Transport> SolanaRpc<T> {
    pub fn new(client: RpcClient<T>) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &RpcClient<T> {
        &self.client
    }

    pub async fn get_slot(&self) -> Result<u64, RpcError> {
        let v = self
            .client
            .call("getSlot", json!([{ "commitment": COMMITMENT }]))
            .await?;
        v.as_u64()
            .ok_or_else(|| malformed("getSlot", "slot is not a number"))
    }

    /// Reads `addresses` in batches of at most 100, each address once, and
    /// returns one read per input address in input order. `slice` is
    /// `(offset, length)` and applies to every account.
    pub async fn get_multiple_accounts(
        &self,
        addresses: &[Address],
        slice: Option<(usize, usize)>,
    ) -> Result<Vec<AccountRead>, RpcError> {
        let mut unique: Vec<&Address> = Vec::new();
        for a in addresses {
            if !unique.contains(&a) {
                unique.push(a);
            }
        }
        let mut by_address: BTreeMap<&Address, AccountRead> = BTreeMap::new();
        for chunk in unique.chunks(MAX_ACCOUNTS_PER_CALL) {
            let mut opts = json!({ "commitment": COMMITMENT, "encoding": "base64" });
            if let Some((offset, length)) = slice {
                opts["dataSlice"] = json!({ "offset": offset, "length": length });
            }
            let keys: Vec<&str> = chunk.iter().map(|a| a.as_str()).collect();
            let result = self
                .client
                .call("getMultipleAccounts", json!([keys, opts]))
                .await?;
            let slot = context_slot("getMultipleAccounts", &result)?;
            let values = result["value"]
                .as_array()
                .filter(|v| v.len() == chunk.len())
                .ok_or_else(|| malformed("getMultipleAccounts", "value length mismatch"))?;
            for (address, value) in chunk.iter().zip(values) {
                by_address.insert(
                    address,
                    AccountRead {
                        address: (*address).clone(),
                        slot,
                        account: parse_account(value)?,
                    },
                );
            }
        }
        Ok(addresses.iter().map(|a| by_address[a].clone()).collect())
    }
}

fn malformed(method: &str, error: &str) -> RpcError {
    RpcError::Malformed {
        method: method.to_string(),
        error: error.to_string(),
    }
}

fn context_slot(method: &str, result: &Value) -> Result<u64, RpcError> {
    result["context"]["slot"]
        .as_u64()
        .ok_or_else(|| malformed(method, "response has no context.slot"))
}

fn parse_account(value: &Value) -> Result<Option<RawAccount>, RpcError> {
    if value.is_null() {
        return Ok(None);
    }
    let bad = |e: &str| malformed("getMultipleAccounts", e);
    let encoded = value["data"][0]
        .as_str()
        .ok_or_else(|| bad("data is not base64"))?;
    let data = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| bad(&e.to_string()))?;
    Ok(Some(RawAccount {
        owner: Address::new(value["owner"].as_str().ok_or_else(|| bad("no owner"))?),
        lamports: value["lamports"].as_u64().unwrap_or(0),
        executable: value["executable"].as_bool().unwrap_or(false),
        data,
    }))
}

/// One entry of `getSignaturesForAddress`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureInfo {
    pub signature: String,
    pub slot: u64,
    pub failed: bool,
    pub block_time: Option<i64>,
}

impl<T: Transport> SolanaRpc<T> {
    /// The newest `limit` finalized signatures touching `address`.
    pub async fn get_signatures_for_address(
        &self,
        address: &Address,
        limit: usize,
    ) -> Result<Vec<SignatureInfo>, RpcError> {
        self.get_signatures_page(address, limit, None).await
    }

    /// One page of signatures, newest first, older than `before` if given.
    pub async fn get_signatures_page(
        &self,
        address: &Address,
        limit: usize,
        before: Option<&str>,
    ) -> Result<Vec<SignatureInfo>, RpcError> {
        let mut opts = json!({ "limit": limit, "commitment": COMMITMENT });
        if let Some(b) = before {
            opts["before"] = json!(b);
        }
        let v = self
            .client
            .call("getSignaturesForAddress", json!([address.as_str(), opts]))
            .await?;
        let list = v
            .as_array()
            .ok_or_else(|| malformed("getSignaturesForAddress", "result is not a list"))?;
        list.iter()
            .map(|s| {
                Ok(SignatureInfo {
                    signature: s["signature"]
                        .as_str()
                        .ok_or_else(|| malformed("getSignaturesForAddress", "no signature"))?
                        .to_string(),
                    slot: s["slot"]
                        .as_u64()
                        .ok_or_else(|| malformed("getSignaturesForAddress", "no slot"))?,
                    failed: !s["err"].is_null(),
                    block_time: s["blockTime"].as_i64(),
                })
            })
            .collect()
    }

    /// A finalized transaction in `json` encoding, or None if the node has
    /// none (pruned or unknown).
    pub async fn get_transaction(&self, signature: &str) -> Result<Option<Value>, RpcError> {
        let v = self
            .client
            .call(
                "getTransaction",
                json!([signature, {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 1,
                    "commitment": COMMITMENT
                }]),
            )
            .await?;
        Ok((!v.is_null()).then_some(v))
    }
}
