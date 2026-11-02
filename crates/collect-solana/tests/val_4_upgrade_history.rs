//! VAL-4: every transaction on a ProgramData account, classified as deploy,
//! upgrade, set authority, extend or close, with slots converted to dates.

mod common;

use std::path::PathBuf;

use celebtir_core::Address;
use celebtir_core::snapshot::UpgradeKind;
use collect_solana::deps::TxCache;
use collect_solana::history::{programdata_events, upgrade_history};
use collect_solana::reads::SolanaRpc;
use common::*;
use serde_json::{Value, json};

const LOADER: &str = "BPFLoaderUpgradeab1e11111111111111111111111";
const PD: &str = "9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA";

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tx/kamino-programdata")
}

fn read(p: PathBuf) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// A transaction calling the upgradeable loader with instruction `tag`.
/// Keys: 0 payer, 1 loader, 2 programdata, 3 current authority, 4 new authority.
fn loader_tx(slot: u64, block_time: Option<i64>, tag: u32, accounts: &[u64], inner: bool) -> Value {
    let data = bs58::encode(tag.to_le_bytes()).into_string();
    let ix = json!({"programIdIndex": 1, "accounts": accounts, "data": data, "stackHeight": if inner { json!(2) } else { Value::Null }});
    let (top, inner_ixs) = if inner {
        (
            vec![json!({"programIdIndex": 5, "accounts": [], "data": ""})],
            vec![json!({"index": 0, "instructions": [ix]})],
        )
    } else {
        (vec![ix], vec![])
    };
    json!({
        "slot": slot,
        "blockTime": block_time,
        "meta": {"err": null, "innerInstructions": inner_ixs, "loadedAddresses": {"writable": [], "readonly": []}},
        "transaction": {"message": {
            "accountKeys": ["payer", LOADER, PD, "OldAuth111111111111111111111111111111111111", "NewAuth111111111111111111111111111111111111", "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"],
            "instructions": top
        }}
    })
}

#[test]
fn val_4_classifies_each_loader_instruction() {
    let pd = Address::new(PD);
    let cases = [
        (2, vec![0, 2], UpgradeKind::Deploy),
        (3, vec![2, 0], UpgradeKind::Upgrade),
        (6, vec![2, 0], UpgradeKind::Extend),
        (5, vec![2, 0], UpgradeKind::Close),
        (4, vec![2, 3, 4], UpgradeKind::SetAuthority),
        (4, vec![2, 3], UpgradeKind::RemoveAuthority),
        (7, vec![2, 3, 4], UpgradeKind::SetAuthority),
    ];
    for (tag, accounts, kind) in cases {
        let ev = programdata_events(&loader_tx(1, Some(0), tag, &accounts, false), &pd).unwrap();
        assert_eq!(ev.len(), 1, "tag {tag}");
        assert_eq!(ev[0].0, kind, "tag {tag}");
    }
    let set = programdata_events(&loader_tx(1, Some(0), 4, &[2, 3, 4], false), &pd).unwrap();
    assert_eq!(
        set[0].1.as_ref().map(Address::as_str),
        Some("NewAuth111111111111111111111111111111111111")
    );
}

#[test]
fn val_4_upgrades_executed_through_a_multisig_cpi_are_found() {
    let ev =
        programdata_events(&loader_tx(1, Some(0), 3, &[2, 0], true), &Address::new(PD)).unwrap();
    assert_eq!(ev[0].0, UpgradeKind::Upgrade);
}

#[test]
fn val_4_loader_calls_on_other_accounts_are_ignored() {
    // Writes to a buffer account, not this ProgramData.
    let ev = programdata_events(&loader_tx(1, Some(0), 1, &[0], false), &Address::new(PD)).unwrap();
    assert!(ev.is_empty());
}

/// Serves `txs` by signature, newest first in getSignaturesForAddress.
fn node(txs: Vec<(String, Value)>) -> SolanaRpc<FnTransport> {
    SolanaRpc::new(client(move |body: &Value| {
        match body["method"].as_str().unwrap() {
            "getSignaturesForAddress" => {
                let mut sigs: Vec<Value> = txs
                .iter()
                .map(|(s, t)| json!({"signature": s, "slot": t["slot"], "err": null, "blockTime": t["blockTime"]}))
                .collect();
                sigs.sort_by_key(|s| std::cmp::Reverse(s["slot"].as_u64()));
                if body["params"][1]["before"].is_string() {
                    json!([])
                } else {
                    Value::Array(sigs)
                }
            }
            "getTransaction" => txs
                .iter()
                .find(|(s, _)| s == body["params"][0].as_str().unwrap())
                .map(|(_, t)| t.clone())
                .unwrap_or(Value::Null),
            m => panic!("{m}"),
        }
    }))
}

#[tokio::test]
async fn val_4_program_upgraded_3_times_shows_3_dated_upgrades_and_its_authority_history() {
    let day = 86_400;
    let t0 = 1_700_000_000;
    let txs = vec![
        ("d".to_string(), loader_tx(100, Some(t0), 2, &[0, 2], false)),
        (
            "u1".to_string(),
            loader_tx(200, Some(t0 + day), 3, &[2, 0], false),
        ),
        (
            "a1".to_string(),
            loader_tx(300, Some(t0 + 2 * day), 4, &[2, 3, 4], false),
        ),
        (
            "u2".to_string(),
            loader_tx(400, Some(t0 + 3 * day), 3, &[2, 0], true),
        ),
        ("u3".to_string(), loader_tx(500, None, 3, &[2, 0], false)),
        (
            "a2".to_string(),
            loader_tx(600, Some(t0 + 5 * day), 4, &[2, 3], false),
        ),
    ];
    let rpc = node(txs);
    let history = upgrade_history(&rpc, &Address::new(PD), &mut TxCache::default(), 1_000)
        .await
        .unwrap()
        .events;
    let upgrades: Vec<_> = history
        .iter()
        .filter(|e| e.kind == UpgradeKind::Upgrade)
        .collect();
    assert_eq!(upgrades.len(), 3);
    assert_eq!(upgrades[0].date.as_deref(), Some("2023-11-15"));
    assert_eq!(
        upgrades[2].date, None,
        "no block time: slot only, never a guessed date"
    );
    assert_eq!(upgrades[2].slot, 500);
    let authority: Vec<_> = history
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                UpgradeKind::SetAuthority | UpgradeKind::RemoveAuthority
            )
        })
        .map(|e| (e.kind, e.new_authority.clone()))
        .collect();
    assert_eq!(
        authority,
        vec![
            (
                UpgradeKind::SetAuthority,
                Some(Address::new("NewAuth111111111111111111111111111111111111"))
            ),
            (UpgradeKind::RemoveAuthority, None),
        ]
    );
    assert!(
        history.windows(2).all(|w| w[0].slot <= w[1].slot),
        "oldest first"
    );
    assert_eq!(history[0].kind, UpgradeKind::Deploy);
}

#[tokio::test]
async fn val_4_kaminos_recorded_programdata_transactions_match_the_independent_count() {
    let sigs = read(dir().join("signatures.json"))["result"]
        .as_array()
        .unwrap()
        .clone();
    let txs: Vec<(String, Value)> = sigs
        .iter()
        .map(|s| {
            let sig = s["signature"].as_str().unwrap().to_string();
            let tx = read(dir().join(format!("{sig}.json")))["result"].clone();
            (sig, tx)
        })
        .collect();
    let history = upgrade_history(
        &node(txs),
        &Address::new(PD),
        &mut TxCache::default(),
        1_000,
    )
    .await
    .unwrap()
    .events;
    let count = |k: UpgradeKind| history.iter().filter(|e| e.kind == k).count();
    assert_eq!(count(UpgradeKind::Deploy), 1);
    assert_eq!(count(UpgradeKind::SetAuthority), 4);
    assert_eq!(count(UpgradeKind::Extend), 1);
    assert_eq!(
        count(UpgradeKind::Upgrade),
        2,
        "the 2 newest of 82 are in the fixture"
    );
    assert_eq!(history[0].date.as_deref(), Some("2023-03-15"));
    let last_authority = history
        .iter()
        .rev()
        .find(|e| e.kind == UpgradeKind::SetAuthority)
        .unwrap();
    assert_eq!(
        last_authority.new_authority.as_ref().unwrap().as_str(),
        "GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW"
    );
    assert_eq!(
        history.last().unwrap().slot,
        440_486_775,
        "equals the header's last-deploy slot"
    );
}

#[tokio::test]
async fn val_4_history_is_cached_by_signature() {
    let rpc = node(vec![(
        "u1".to_string(),
        loader_tx(200, Some(0), 3, &[2, 0], false),
    )]);
    let mut cache = TxCache::default();
    upgrade_history(&rpc, &Address::new(PD), &mut cache, 1_000)
        .await
        .unwrap();
    upgrade_history(&rpc, &Address::new(PD), &mut cache, 1_000)
        .await
        .unwrap();
    assert_eq!(rpc.client().report().calls["getTransaction"], 1);
}

/// Busy ProgramData accounts fill in over several runs within the budget.
#[tokio::test]
async fn val_4_history_fills_in_across_runs_when_capped() {
    let txs: Vec<(String, Value)> = (0..5)
        .map(|i| {
            (
                format!("s{i}"),
                loader_tx(100 + i, Some(0), if i == 0 { 2 } else { 3 }, &[2, 0], false),
            )
        })
        .collect();
    let rpc = node(txs);
    let mut cache = TxCache::default();
    let first = upgrade_history(&rpc, &Address::new(PD), &mut cache, 2)
        .await
        .unwrap();
    assert_eq!((first.scanned, first.total, first.complete), (2, 5, false));
    assert_eq!(first.events.len(), 2, "the two newest");
    let second = upgrade_history(&rpc, &Address::new(PD), &mut cache, 2)
        .await
        .unwrap();
    assert_eq!((second.scanned, second.complete), (4, false));
    let third = upgrade_history(&rpc, &Address::new(PD), &mut cache, 2)
        .await
        .unwrap();
    assert!(third.complete);
    assert_eq!(third.events.len(), 5);
    assert_eq!(third.events[0].kind, UpgradeKind::Deploy);
    assert_eq!(
        rpc.client().report().calls["getTransaction"],
        5,
        "each transaction read once"
    );
}
