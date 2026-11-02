//! COL-3 (finalized commitment, slot on every read, min/max slot per
//! protocol) and COL-4 (getMultipleAccounts batches of up to 100).

mod common;

use celebtir_core::Address;
use collect_solana::reads::{SlotRange, SolanaRpc};
use common::*;
use serde_json::{Value, json};

fn addrs(n: usize) -> Vec<Address> {
    (0..n)
        .map(|i| Address::new(format!("Acct{i:040}")))
        .collect()
}

fn rpc_at_slot(slot: u64) -> SolanaRpc<FnTransport> {
    SolanaRpc::new(client(move |body: &Value| {
        match body["method"].as_str().unwrap() {
            "getMultipleAccounts" => accounts_result(
                &body["params"][0],
                slot,
                "BPFLoaderUpgradeab1e11111111111111111111111",
                &[1, 2, 3],
            ),
            "getSlot" => json!(slot),
            m => panic!("unexpected {m}"),
        }
    }))
}

#[tokio::test]
async fn col_4_ten_accounts_cost_one_call() {
    let rpc = rpc_at_slot(100);
    let reads = rpc.get_multiple_accounts(&addrs(10), None).await.unwrap();
    assert_eq!(reads.len(), 10);
    assert_eq!(rpc.client().report().calls["getMultipleAccounts"], 1);
}

#[tokio::test]
async fn col_4_batches_hold_at_most_100_accounts() {
    let rpc = rpc_at_slot(100);
    let reads = rpc.get_multiple_accounts(&addrs(250), None).await.unwrap();
    assert_eq!(reads.len(), 250);
    let sizes: Vec<usize> = rpc
        .client()
        .transport()
        .bodies()
        .iter()
        .map(|b| b["params"][0].as_array().unwrap().len())
        .collect();
    assert_eq!(sizes, vec![100, 100, 50]);
}

#[tokio::test]
async fn col_4_duplicate_addresses_are_read_once_and_order_is_kept() {
    let rpc = rpc_at_slot(100);
    let mut list = addrs(3);
    list.push(list[0].clone());
    let reads = rpc.get_multiple_accounts(&list, None).await.unwrap();
    assert_eq!(reads.len(), 4);
    assert_eq!(reads[3].address, list[0]);
    let sent = rpc.client().transport().bodies();
    assert_eq!(sent[0]["params"][0].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn col_3_reads_use_finalized_commitment_and_base64() {
    let rpc = rpc_at_slot(100);
    rpc.get_multiple_accounts(&addrs(2), Some((0, 45)))
        .await
        .unwrap();
    rpc.get_slot().await.unwrap();
    for body in rpc.client().transport().bodies() {
        let opts = body["params"].as_array().unwrap().last().unwrap();
        assert_eq!(opts["commitment"], "finalized", "{body}");
    }
    let first = &rpc.client().transport().bodies()[0];
    assert_eq!(first["params"][1]["encoding"], "base64");
    assert_eq!(
        first["params"][1]["dataSlice"],
        json!({"offset": 0, "length": 45})
    );
}

#[tokio::test]
async fn col_3_every_read_carries_the_response_slot() {
    let rpc = rpc_at_slot(382_114_880);
    let reads = rpc.get_multiple_accounts(&addrs(3), None).await.unwrap();
    for r in &reads {
        assert_eq!(r.slot, 382_114_880);
        let acct = r.account.as_ref().unwrap();
        assert_eq!(acct.data, vec![1, 2, 3]);
        assert_eq!(
            acct.owner.as_str(),
            "BPFLoaderUpgradeab1e11111111111111111111111"
        );
    }
}

#[tokio::test]
async fn col_3_missing_accounts_are_none_not_errors() {
    let rpc = SolanaRpc::new(client(|body: &Value| {
        let n = body["params"][0].as_array().unwrap().len();
        json!({"context": {"slot": 5}, "value": vec![Value::Null; n]})
    }));
    let reads = rpc.get_multiple_accounts(&addrs(2), None).await.unwrap();
    assert!(reads.iter().all(|r| r.account.is_none() && r.slot == 5));
}

#[test]
fn col_3_slot_range_tracks_min_and_max() {
    let mut r = SlotRange::default();
    assert_eq!(r.get(), None);
    for s in [300, 100, 200] {
        r.record(s);
    }
    assert_eq!(r.get(), Some((100, 300)));
}
