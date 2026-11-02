//! Captures one mainnet account into `fixtures/accounts/<name>.json` (QA-1).
//!
//! cargo run -p collect-solana --example capture -- <name> <address> [--slice 0:45] [--note "why"]
//!
//! Uses CELEBTIR_RPC_URL if set, otherwise the public endpoint. Only the host
//! is written to the file.

use celebtir_core::{Address, RpcEndpoint, to_canonical_json};
use collect_solana::fixtures::{AccountFixture, accounts_dir};
use collect_solana::reads::SolanaRpc;
use collect_solana::rpc::{HttpTransport, RpcClient, RpcConfig};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (name, address) = match (args.first(), args.get(1)) {
        (Some(n), Some(a)) => (n.clone(), Address::new(a)),
        _ => {
            eprintln!("usage: capture <name> <address> [--slice OFFSET:LEN] [--note TEXT]");
            std::process::exit(1);
        }
    };
    let flag = |f: &str| {
        args.iter()
            .position(|a| a == f)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let slice = flag("--slice").map(|s| {
        let (o, l) = s.split_once(':').expect("--slice OFFSET:LEN");
        (o.parse().expect("offset"), l.parse().expect("length"))
    });
    let note = flag("--note").unwrap_or_default();

    let url = std::env::var("CELEBTIR_RPC_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".into());
    let endpoint = RpcEndpoint::new(url);
    let host = endpoint.host().to_string();
    let rpc = SolanaRpc::new(RpcClient::new(
        HttpTransport::new(endpoint),
        RpcConfig {
            requests_per_sec: 2,
            ..RpcConfig::default()
        },
    ));
    let read = rpc
        .get_multiple_accounts(std::slice::from_ref(&address), slice)
        .await
        .unwrap_or_else(|e| panic!("read failed: {e}"))
        .remove(0);
    let source = format!("{host} getMultipleAccounts finalized");
    let fixture = AccountFixture::from_read(&name, &read, slice, &source, &note);
    let path = accounts_dir().join(format!("{name}.json"));
    std::fs::write(&path, to_canonical_json(&fixture).unwrap()).unwrap();
    println!("{} @ slot {} → {}", address, read.slot, path.display());
}
