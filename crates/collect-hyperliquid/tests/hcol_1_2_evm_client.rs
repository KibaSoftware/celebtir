//! HCOL-1 (rate-limited client, retries, budget, host-only URL) and HCOL-2
//! (every read at one pinned block).

use std::collections::VecDeque;
use std::sync::Mutex;

use collect_hyperliquid::evm::{
    EvmError, HyperEvm, PUBLIC_HYPEREVM_RPC_URL, default_config, hyperevm_endpoint,
};
use collect_solana::rpc::{RpcClient, RpcConfig, RpcError, Transport, TransportError};
use serde_json::{Value, json};

#[derive(Default)]
struct Mock {
    script: Mutex<VecDeque<Result<Value, TransportError>>>,
    sent: Mutex<Vec<Value>>,
}

impl Mock {
    fn new(replies: Vec<Result<Value, TransportError>>) -> Self {
        Self {
            script: Mutex::new(replies.into()),
            ..Default::default()
        }
    }
}

impl Transport for Mock {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        self.sent.lock().unwrap().push(body);
        self.script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| ok(json!("0x")))
    }
}

fn ok(result: Value) -> Result<Value, TransportError> {
    Ok(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
}

const ADDR: &str = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";

#[test]
fn hcol_1_unset_variable_warns_and_uses_public_rpc() {
    let (e, warning) = hyperevm_endpoint(None);
    assert_eq!(e.url(), PUBLIC_HYPEREVM_RPC_URL);
    assert!(warning.unwrap().contains("CELEBTIR_HYPEREVM_RPC_URL"));
    let (e, warning) = hyperevm_endpoint(Some("  ".into()));
    assert_eq!(e.url(), PUBLIC_HYPEREVM_RPC_URL);
    assert!(warning.is_some());
}

#[test]
fn hcol_1_private_url_is_reduced_to_its_host() {
    let (e, warning) = hyperevm_endpoint(Some("https://evm.example.com/v1/secret-key".into()));
    assert!(warning.is_none());
    assert_eq!(e.host(), "evm.example.com");
    assert!(!format!("{e:?}").is_empty());
    assert!(!e.host().contains("secret"));
}

#[test]
fn hcol_1_public_rpc_defaults_are_conservative() {
    let c = default_config();
    assert!(c.requests_per_sec <= 2, "{c:?}");
    assert!(c.daily_budget <= 2_000, "{c:?}");
    assert_eq!(c.max_attempts, 5);
}

#[tokio::test(start_paused = true)]
async fn hcol_1_mocked_429_succeeds_on_retry() {
    let mock = Mock::new(vec![
        ok(json!("0x3e7")),
        Err(TransportError::Status(429)),
        ok(json!("0x112a880")),
    ]);
    let evm = HyperEvm::connect(RpcClient::new(mock, default_config()))
        .await
        .unwrap();
    assert_eq!(evm.block(), 18_000_000);
    assert_eq!(evm.rpc().report().retries, 1);
}

#[tokio::test(start_paused = true)]
async fn hcol_1_budget_halts_with_a_budget_error() {
    let cfg = RpcConfig {
        daily_budget: 2,
        ..default_config()
    };
    let mock = Mock::new(vec![ok(json!("0x3e7")), ok(json!("0x10"))]);
    let evm = HyperEvm::connect(RpcClient::new(mock, cfg)).await.unwrap();
    let err = evm.code(ADDR).await.unwrap_err();
    assert_eq!(
        err,
        EvmError::Rpc(RpcError::BudgetExceeded { budget: 2 }),
        "{err}"
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_1_wrong_chain_is_refused() {
    let mock = Mock::new(vec![ok(json!("0x1"))]);
    let err = HyperEvm::connect(RpcClient::new(mock, default_config()))
        .await
        .err()
        .unwrap();
    assert_eq!(err, EvmError::WrongChain { found: 1 });
}

#[tokio::test(start_paused = true)]
async fn hcol_2_every_read_uses_the_pinned_block() {
    let slot = [0u8; 32];
    let word = format!("0x{}", "00".repeat(31) + "2a");
    let mock = Mock::new(vec![
        ok(json!("0x3e7")),
        ok(json!("0x112a880")),
        ok(json!(word)),
        ok(json!("0x6080")),
        ok(json!("0x01")),
    ]);
    let evm = HyperEvm::connect(RpcClient::new(mock, default_config()))
        .await
        .unwrap();
    let w = evm.storage(ADDR, &slot).await.unwrap();
    assert_eq!(w[31], 0x2a);
    assert_eq!(evm.code(ADDR).await.unwrap(), vec![0x60, 0x80]);
    assert_eq!(evm.call(ADDR, &[0xab]).await.unwrap(), vec![1]);

    let sent = evm.rpc().transport().sent.lock().unwrap().clone();
    let calls: Vec<&Value> = sent
        .iter()
        .filter(|b| b["method"] != "eth_chainId" && b["method"] != "eth_blockNumber")
        .collect();
    assert_eq!(calls.len(), 3);
    for b in calls {
        let params = b["params"].as_array().unwrap();
        assert_eq!(params.last().unwrap(), "0x112a880", "{b}");
    }
    assert_eq!(evm.rpc().report().calls["eth_blockNumber"], 1);
}

#[tokio::test(start_paused = true)]
async fn hcol_2_short_storage_word_is_a_decode_error() {
    let mock = Mock::new(vec![
        ok(json!("0x3e7")),
        ok(json!("0x1")),
        ok(json!("0x2a")),
    ]);
    let evm = HyperEvm::connect(RpcClient::new(mock, default_config()))
        .await
        .unwrap();
    assert!(matches!(
        evm.storage(ADDR, &[0; 32]).await,
        Err(EvmError::Decode { .. })
    ));
}
