//! Recorded Hyperliquid responses for offline tests (HOPS-2, ADR 009).
//!
//! - `fixtures/hyperliquid/info/<name>.json`: one info API request and its
//!   response, anchored to the HyperEVM block read just before (ADR 024 D6).
//! - `fixtures/hyperliquid/evm/<name>.json`: JSON-RPC calls made at one
//!   pinned block, replayed by [`Replay`].
//!
//! Files record the host only, never a URL (SNAP-5).

use std::path::Path;

use collect_solana::rpc::{Transport, TransportError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfoFixture {
    pub host: String,
    /// HyperEVM block read right before the request.
    pub anchor_block: u64,
    pub request: Value,
    pub response: Value,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvmCall {
    pub method: String,
    pub params: Value,
    /// The JSON-RPC `result`, or None when the node returned an error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvmFixture {
    pub host: String,
    pub chain_id: u64,
    pub block: u64,
    pub calls: Vec<EvmCall>,
    pub note: String,
}

fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

impl InfoFixture {
    pub fn load(path: &Path) -> Result<Self, String> {
        read(path)
    }
}

impl EvmFixture {
    pub fn load(path: &Path) -> Result<Self, String> {
        read(path)
    }

    /// A transport that answers `eth_chainId` and `eth_blockNumber` from the
    /// header and every other call from the recording. An unrecorded call is
    /// a network error, so a test can never silently go online.
    pub fn replay(&self) -> Replay {
        Replay(self.clone())
    }
}

pub struct Replay(EvmFixture);

impl Transport for Replay {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        let method = body["method"].as_str().unwrap_or_default();
        let params = &body["params"];
        let f = &self.0;
        let result = match method {
            "eth_chainId" => json!(format!("0x{:x}", f.chain_id)),
            "eth_blockNumber" => json!(format!("0x{:x}", f.block)),
            _ => {
                let call = f
                    .calls
                    .iter()
                    .find(|c| c.method == method && same_params(&c.params, params))
                    .ok_or_else(|| {
                        TransportError::Network(format!("not in fixture: {method} {params}"))
                    })?;
                if let Some(e) = &call.error {
                    return Ok(json!({"jsonrpc": "2.0", "id": 1, "error": e}));
                }
                call.result.clone().unwrap_or(Value::Null)
            }
        };
        Ok(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
    }
}

/// Addresses and hex compare case-insensitively.
fn same_params(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(x), Value::String(y)) => x.eq_ignore_ascii_case(y),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same_params(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| same_params(v, w)))
        }
        _ => a == b,
    }
}

/// Wraps a live transport and records every call, so a capture holds
/// exactly the reads the collector makes.
pub struct Recorder<T> {
    inner: T,
    calls: std::sync::Mutex<Vec<EvmCall>>,
}

impl<T> Recorder<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            calls: Default::default(),
        }
    }

    /// Recorded calls, without the chain ID and block number reads that the
    /// fixture header holds.
    pub fn calls(&self) -> Vec<EvmCall> {
        self.calls.lock().expect("recorder lock").clone()
    }
}

impl<T: Transport> Transport for Recorder<T> {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        let r = self.inner.send(body.clone()).await?;
        let method = body["method"].as_str().unwrap_or_default().to_string();
        if method != "eth_chainId" && method != "eth_blockNumber" {
            let mut calls = self.calls.lock().expect("recorder lock");
            let params = body["params"].clone();
            if !calls
                .iter()
                .any(|c| c.method == method && same_params(&c.params, &params))
            {
                calls.push(EvmCall {
                    method,
                    params,
                    result: r.get("result").cloned(),
                    error: r.get("error").cloned(),
                });
            }
        }
        Ok(r)
    }
}

/// Replays info API fixtures by request body; anything unrecorded fails.
pub struct InfoReplay(pub Vec<InfoFixture>);

impl Transport for InfoReplay {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        self.0
            .iter()
            .find(|f| same_params(&f.request, &body))
            .map(|f| f.response.clone())
            .ok_or_else(|| TransportError::Network(format!("not in fixture: {body}")))
    }
}
