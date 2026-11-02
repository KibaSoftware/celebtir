//! HyperEVM reads over JSON-RPC (HCOL-1, HCOL-2). Every call goes through
//! the shared rate-limited client from `collect-solana` and is made at one
//! block, pinned once per run, which every fact records as its `slot`.

use celebtir_core::RpcEndpoint;
use collect_solana::rpc::{RpcClient, RpcConfig, RpcError, Transport};
use serde_json::{Value, json};

/// Used when `CELEBTIR_HYPEREVM_RPC_URL` is unset (ADR 024).
pub const PUBLIC_HYPEREVM_RPC_URL: &str = "https://rpc.hyperliquid.xyz/evm";
/// HyperEVM mainnet.
pub const CHAIN_ID: u64 = 999;

/// The endpoint from `CELEBTIR_HYPEREVM_RPC_URL`, or the public RPC plus a
/// warning to print when it is unset or blank (HCOL-1, as CLI-1).
pub fn hyperevm_endpoint(env_value: Option<String>) -> (RpcEndpoint, Option<String>) {
    match env_value.filter(|v| !v.trim().is_empty()) {
        Some(url) => (RpcEndpoint::new(url.trim()), None),
        None => (
            RpcEndpoint::new(PUBLIC_HYPEREVM_RPC_URL),
            Some(format!(
                "warning: CELEBTIR_HYPEREVM_RPC_URL is not set; using the rate-limited public endpoint {PUBLIC_HYPEREVM_RPC_URL}"
            )),
        ),
    }
}

/// Defaults for the public RPC, which allows far fewer calls than Helius.
pub fn default_config() -> RpcConfig {
    RpcConfig {
        requests_per_sec: 2,
        max_concurrency: 2,
        daily_budget: 2_000,
        ..RpcConfig::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EvmError {
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error("{method}: {error}")]
    Decode { method: String, error: String },
    #[error("chain ID is {found}, expected {CHAIN_ID}")]
    WrongChain { found: u64 },
}

/// A 32-byte word, as returned by `eth_getStorageAt` or a one-word `eth_call`.
pub type Word = [u8; 32];

pub struct HyperEvm<T> {
    rpc: RpcClient<T>,
    block: u64,
}

impl<T: Transport> HyperEvm<T> {
    /// Checks the chain ID, then pins the latest block for the whole run.
    pub async fn connect(rpc: RpcClient<T>) -> Result<Self, EvmError> {
        let chain = quantity("eth_chainId", &rpc.call("eth_chainId", json!([])).await?)?;
        if chain != CHAIN_ID {
            return Err(EvmError::WrongChain { found: chain });
        }
        let block = quantity(
            "eth_blockNumber",
            &rpc.call("eth_blockNumber", json!([])).await?,
        )?;
        Ok(Self { rpc, block })
    }

    /// The block every read in this run is made at (HCOL-2).
    pub fn block(&self) -> u64 {
        self.block
    }

    pub fn rpc(&self) -> &RpcClient<T> {
        &self.rpc
    }

    fn tag(&self) -> String {
        format!("0x{:x}", self.block)
    }

    /// `eth_getStorageAt(address, slot)` at the pinned block.
    pub async fn storage(&self, address: &str, slot: &Word) -> Result<Word, EvmError> {
        let v = self
            .rpc
            .call(
                "eth_getStorageAt",
                json!([address, hex_word(slot), self.tag()]),
            )
            .await?;
        word("eth_getStorageAt", &v)
    }

    /// `eth_getCode(address)` at the pinned block.
    pub async fn code(&self, address: &str) -> Result<Vec<u8>, EvmError> {
        let v = self
            .rpc
            .call("eth_getCode", json!([address, self.tag()]))
            .await?;
        bytes("eth_getCode", &v)
    }

    /// `eth_call` with raw calldata at the pinned block. A revert surfaces
    /// as an RPC error, which callers record as Unavailable.
    pub async fn call(&self, to: &str, data: &[u8]) -> Result<Vec<u8>, EvmError> {
        let v = self
            .rpc
            .call(
                "eth_call",
                json!([{"to": to, "data": hex(data)}, self.tag()]),
            )
            .await?;
        bytes("eth_call", &v)
    }
}

pub fn hex(data: &[u8]) -> String {
    let mut s = String::with_capacity(2 + data.len() * 2);
    s.push_str("0x");
    for b in data {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_word(w: &Word) -> String {
    hex(w)
}

fn decode_err(method: &str, error: impl Into<String>) -> EvmError {
    EvmError::Decode {
        method: method.to_string(),
        error: error.into(),
    }
}

/// `"0x3e7"` → 999.
pub fn quantity(method: &str, v: &Value) -> Result<u64, EvmError> {
    let s = v
        .as_str()
        .and_then(|s| s.strip_prefix("0x"))
        .ok_or_else(|| decode_err(method, format!("not a hex quantity: {v}")))?;
    u64::from_str_radix(s, 16).map_err(|e| decode_err(method, e.to_string()))
}

/// `"0x…"` → bytes.
pub fn bytes(method: &str, v: &Value) -> Result<Vec<u8>, EvmError> {
    let s = v
        .as_str()
        .and_then(|s| s.strip_prefix("0x"))
        .ok_or_else(|| decode_err(method, format!("not hex data: {v}")))?;
    if s.len() % 2 != 0 {
        return Err(decode_err(method, "odd-length hex"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<_, _>>()
        .map_err(|e| decode_err(method, e.to_string()))
}

fn word(method: &str, v: &Value) -> Result<Word, EvmError> {
    let b = bytes(method, v)?;
    b.try_into()
        .map_err(|b: Vec<u8>| decode_err(method, format!("{} bytes, not 32", b.len())))
}
