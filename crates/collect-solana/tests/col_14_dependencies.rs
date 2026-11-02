//! COL-14: sample recent finalized transactions per core program, resolve
//! lookup-table addresses, use stack heights to list the programs the
//! protocol calls directly, and drop system, token, compute-budget and memo.

mod common;

use std::collections::BTreeSet;
use std::path::PathBuf;

use celebtir_core::Address;
use collect_solana::deps::{TxCache, direct_callees, sample_dependencies};
use collect_solana::reads::SolanaRpc;
use common::*;
use serde_json::{Value, json};

const KLEND: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const TOKEN: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

fn a(s: &str) -> Address {
    Address::new(s)
}

fn tx_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tx/kamino-lend")
}

fn read(path: PathBuf) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn captured() -> Vec<Value> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(tx_dir()).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name != "signatures.json" {
            out.push(read(e.path())["result"].clone());
        }
    }
    out
}

/// A transaction with explicit instructions: `(program index, stack height)`
/// lists per top-level instruction.
fn synthetic(keys: &[&str], top: &[(usize, Vec<(usize, u64)>)]) -> Value {
    json!({
        "slot": 1,
        "meta": {
            "err": null,
            "loadedAddresses": {"writable": [], "readonly": []},
            "innerInstructions": top.iter().enumerate().map(|(i, (_, inner))| json!({
                "index": i,
                "instructions": inner.iter().map(|(p, h)| json!({"programIdIndex": p, "accounts": [], "data": "", "stackHeight": h})).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        },
        "transaction": {"message": {
            "accountKeys": keys,
            "instructions": top.iter().map(|(p, _)| json!({"programIdIndex": p, "accounts": [], "data": "", "stackHeight": null})).collect::<Vec<_>>()
        }}
    })
}

#[test]
fn col_14_nested_call_lists_only_the_direct_callee() {
    // core → A → B: only A is a direct callee.
    let tx = synthetic(
        &["payer", KLEND, "ProgA", "ProgB"],
        &[(1, vec![(2, 2), (3, 3)])],
    );
    let callees = direct_callees(&tx, &a(KLEND)).unwrap();
    assert_eq!(callees, BTreeSet::from([a("ProgA")]));
}

#[test]
fn col_14_calls_made_when_the_core_program_is_itself_called_count() {
    // aggregator → core → A (heights 1, 2, 3); the aggregator is not a callee.
    let tx = synthetic(
        &["payer", "Aggregator", KLEND, "ProgA"],
        &[(1, vec![(2, 2), (3, 3)])],
    );
    assert_eq!(
        direct_callees(&tx, &a(KLEND)).unwrap(),
        BTreeSet::from([a("ProgA")])
    );
}

#[test]
fn col_14_sibling_calls_after_returning_are_attributed_correctly() {
    // core → A, A returns; then core → B; then a call at height 2 from another top-level ix.
    let tx = synthetic(
        &["payer", KLEND, "ProgA", "ProgB", "Other", "ProgC"],
        &[(1, vec![(2, 2), (3, 2)]), (4, vec![(5, 2)])],
    );
    assert_eq!(
        direct_callees(&tx, &a(KLEND)).unwrap(),
        BTreeSet::from([a("ProgA"), a("ProgB")])
    );
}

#[test]
fn col_14_lookup_table_addresses_are_resolved() {
    let mut tx = synthetic(&["payer", KLEND], &[(1, vec![(2, 2)])]);
    tx["meta"]["loadedAddresses"] = json!({"writable": [], "readonly": ["FromLookupTable"]});
    assert_eq!(
        direct_callees(&tx, &a(KLEND)).unwrap(),
        BTreeSet::from([a("FromLookupTable")])
    );
}

#[test]
fn col_14_kamino_samples_call_only_the_token_program_directly() {
    // Hand-checked call trees of the 4 captured transactions: Kamino runs at
    // height 1, and at height 2 under another program, where it calls the
    // Token program at height 3. Its own self-CPIs are not dependencies.
    let mut all = BTreeSet::new();
    for tx in captured() {
        all.extend(direct_callees(&tx, &a(KLEND)).unwrap());
    }
    assert_eq!(all, BTreeSet::from([a(TOKEN)]));
}

#[test]
fn col_14_malformed_transaction_is_an_error() {
    assert!(direct_callees(&json!({"transaction": {}}), &a(KLEND)).is_err());
    let tx = synthetic(&["payer", KLEND], &[(1, vec![(9, 2)])]);
    assert!(
        direct_callees(&tx, &a(KLEND)).is_err(),
        "program index out of range"
    );
}

fn node() -> SolanaRpc<FnTransport> {
    SolanaRpc::new(client(|body: &Value| {
        match body["method"].as_str().unwrap() {
            "getSignaturesForAddress" => {
                assert_eq!(body["params"][1]["commitment"], "finalized");
                read(tx_dir().join("signatures.json"))["result"].clone()
            }
            "getTransaction" => {
                assert_eq!(
                    body["params"][1]["maxSupportedTransactionVersion"], 1,
                    "v1 transactions exist on mainnet"
                );
                let path = tx_dir().join(format!("{}.json", body["params"][0].as_str().unwrap()));
                if path.exists() {
                    read(path)["result"].clone()
                } else {
                    Value::Null
                }
            }
            m => panic!("unexpected {m}"),
        }
    }))
}

#[tokio::test]
async fn col_14_sample_drops_token_program_and_own_programs() {
    let rpc = node();
    let mut cache = TxCache::default();
    let own = [a(KLEND), a("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr")];
    let s = sample_dependencies(&rpc, &[a(KLEND)], &own, &mut cache, 50)
        .await
        .unwrap();
    assert!(s.callees.is_empty(), "{:?}", s.callees);
    assert_eq!(
        s.transactions, 4,
        "two signatures have no captured transaction and count as unreadable"
    );
    assert_eq!(s.newest_slot, Some(458_014_887));
    let r = rpc.client().report();
    assert_eq!(r.calls["getSignaturesForAddress"], 1);
    assert_eq!(r.calls["getTransaction"], 6);
}

#[tokio::test]
async fn col_14_disabled_sample_does_not_request_an_invalid_zero_limit() {
    let rpc = SolanaRpc::new(client(|_: &Value| {
        panic!("disabled sampling made an RPC request")
    }));
    let sample = sample_dependencies(&rpc, &[a(KLEND)], &[a(KLEND)], &mut TxCache::default(), 0)
        .await
        .unwrap();
    assert_eq!(sample.transactions, 0);
    assert!(sample.callees.is_empty());
    assert_eq!(sample.newest_slot, None);
    let history =
        collect_solana::history::upgrade_history(&rpc, &a(KLEND), &mut TxCache::default(), 0)
            .await
            .unwrap();
    assert!(history.events.is_empty());
    assert!(!history.complete);
}

#[tokio::test]
async fn col_14_cached_transactions_are_not_fetched_again() {
    let rpc = node();
    let mut cache = TxCache::default();
    sample_dependencies(&rpc, &[a(KLEND)], &[a(KLEND)], &mut cache, 50)
        .await
        .unwrap();
    sample_dependencies(&rpc, &[a(KLEND)], &[a(KLEND)], &mut cache, 50)
        .await
        .unwrap();
    let r = rpc.client().report();
    assert_eq!(r.calls["getSignaturesForAddress"], 2);
    assert_eq!(
        r.calls["getTransaction"],
        6 + 2,
        "only the two uncached (missing) ones are retried"
    );
}

/// Found by the first live recording: one transaction the node refuses with
/// a JSON-RPC error must not fail the whole protocol; it is skipped.
#[tokio::test]
async fn col_14_a_transaction_the_node_refuses_is_skipped_not_fatal() {
    use collect_solana::rpc::{RpcClient, RpcConfig, Transport, TransportError};
    struct Refusing;
    impl Transport for Refusing {
        async fn send(&self, body: Value) -> Result<Value, TransportError> {
            let result = match body["method"].as_str().unwrap() {
                "getSignaturesForAddress" => {
                    read(tx_dir().join("signatures.json"))["result"].clone()
                }
                "getTransaction" if body["params"][0].as_str().unwrap().starts_with("zoNS") => {
                    return Ok(
                        json!({"jsonrpc": "2.0", "id": 1, "error": {"code": -32015, "message": "Transaction version (2) is not supported"}}),
                    );
                }
                "getTransaction" => {
                    let p = tx_dir().join(format!("{}.json", body["params"][0].as_str().unwrap()));
                    if p.exists() {
                        read(p)["result"].clone()
                    } else {
                        Value::Null
                    }
                }
                m => panic!("{m}"),
            };
            Ok(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
        }
    }
    let rpc = SolanaRpc::new(RpcClient::new(
        Refusing,
        RpcConfig {
            requests_per_sec: 1000,
            ..RpcConfig::default()
        },
    ));
    let mut cache = TxCache::default();
    let s = sample_dependencies(&rpc, &[a(KLEND)], &[a(KLEND)], &mut cache, 50)
        .await
        .unwrap();
    assert_eq!(
        s.transactions, 3,
        "the refused transaction is skipped, the others are read"
    );
}
