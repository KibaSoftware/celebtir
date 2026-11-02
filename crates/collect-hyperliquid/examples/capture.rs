//! Records Hyperliquid fixtures (HOPS-2). Network use is limited to this
//! script; tests replay the files offline.
//!
//! ```text
//! cargo run -p celebtir-collect-hyperliquid --example capture -- info <name> '<request json>' [--note "why"]
//! cargo run -p celebtir-collect-hyperliquid --example capture -- evm <name> '<[[method, params], …]>' [--note "why"]
//! cargo run -p celebtir-collect-hyperliquid --example capture -- resolve <name> <contract> <eip1967|uups|none> [--note "why"]
//! ```
//! In `evm` params, the string `"BLOCK"` is replaced with the pinned block.
//! `resolve` runs the controller resolver live and records every call it
//! makes, using `registry/controllers.yaml` and any `--timelock` entries
//! from `registry/hyperliquid/`.

use std::path::PathBuf;

use celebtir_core::{Address, RpcEndpoint, to_canonical_json};
use celebtir_registry::ProxyKind;
use collect_hyperliquid::evm::{HyperEvm, default_config, hyperevm_endpoint, quantity};
use collect_hyperliquid::fixtures::{EvmCall, EvmFixture, InfoFixture, Recorder};
use collect_hyperliquid::resolve::{EvmKnown, Resolver};
use collect_solana::rpc::{HttpTransport, RpcClient, Transport};
use serde_json::{Value, json};

const INFO_URL: &str = "https://api.hyperliquid.xyz/info";

fn spec_raw(s: &str) -> String {
    s.to_string()
}

fn usage() -> ! {
    eprintln!("usage: capture (info|evm) <name> <json> [--note TEXT]");
    std::process::exit(2);
}

async fn rpc(t: &HttpTransport, method: &str, params: Value) -> Value {
    t.send(json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}))
        .await
        .unwrap_or_else(|e| panic!("{method}: {e}"))
}

fn substitute(v: &Value, block: &str) -> Value {
    match v {
        Value::String(s) if s == "BLOCK" => json!(block),
        Value::Array(a) => Value::Array(a.iter().map(|x| substitute(x, block)).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, x)| (k.clone(), substitute(x, block)))
                .collect(),
        ),
        other => other.clone(),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        usage();
    }
    let (kind, name, spec) = (&args[0], &args[1], &args[2]);
    let note = match args.get(3).map(String::as_str) {
        Some("--note") => args.get(4).cloned().unwrap_or_default(),
        Some(_) if kind == "resolve" => String::new(),
        Some(_) => usage(),
        None => String::new(),
    };
    let spec: Value = if kind == "resolve" {
        Value::Null
    } else {
        serde_json::from_str(spec).expect("argument is JSON")
    };

    let (endpoint, warning) = hyperevm_endpoint(std::env::var("CELEBTIR_HYPEREVM_RPC_URL").ok());
    if let Some(w) = warning {
        eprintln!("{w}");
    }
    let evm = HttpTransport::new(endpoint.clone());
    let chain_id = quantity(
        "eth_chainId",
        &rpc(&evm, "eth_chainId", json!([])).await["result"],
    )
    .expect("chain id");
    let block = quantity(
        "eth_blockNumber",
        &rpc(&evm, "eth_blockNumber", json!([])).await["result"],
    )
    .expect("block number");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/hyperliquid");
    if kind == "resolve" {
        let proxy: ProxyKind = serde_json::from_value(json!(args[3])).expect("proxy kind");
        let note = match args.get(4).map(String::as_str) {
            Some("--note") => args.get(5).cloned().unwrap_or_default(),
            _ => String::new(),
        };
        let registry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry");
        let controllers = EvmKnown::controllers_from_yaml(
            &std::fs::read_to_string(registry.join("controllers.yaml")).unwrap(),
        )
        .unwrap();
        let report = celebtir_registry::validate(&registry);
        let timelocks: Vec<_> = report
            .hyperevm_protocols
            .iter()
            .flat_map(|p| p.timelocks.clone())
            .collect();
        let known = EvmKnown::new(&controllers, &timelocks);
        let client = RpcClient::new(Recorder::new(evm), default_config());
        let hyper = HyperEvm::connect(client).await.expect("connect");
        let resolver = Resolver {
            evm: &hyper,
            known: &known,
        };
        let r = resolver
            .upgrade_control(&Address::new(spec_raw(&args[2])), proxy)
            .await;
        eprintln!("{:?}", r.controller);
        let f = EvmFixture {
            host: endpoint.host().to_string(),
            chain_id,
            block: hyper.block(),
            calls: hyper.rpc().transport().calls(),
            note,
        };
        let dir = root.join("evm");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("{name}.json"));
        std::fs::write(
            &file,
            to_canonical_json(&f).unwrap()
                + "
",
        )
        .unwrap();
        println!("{} (block {})", file.display(), hyper.block());
        return;
    }
    let (path, text) = match kind.as_str() {
        "info" => {
            let info = HttpTransport::new(RpcEndpoint::new(INFO_URL));
            let response = info.send(spec.clone()).await.expect("info API");
            let f = InfoFixture {
                host: info.endpoint().host().to_string(),
                anchor_block: block,
                request: spec,
                response,
                note,
            };
            (root.join("info"), to_canonical_json(&f).unwrap())
        }
        "evm" => {
            let tag = format!("0x{block:x}");
            let mut calls = Vec::new();
            for c in spec.as_array().expect("an array of [method, params]") {
                let method = c[0].as_str().expect("method").to_string();
                let params = substitute(&c[1], &tag);
                let r = rpc(&evm, &method, params.clone()).await;
                calls.push(EvmCall {
                    method,
                    params,
                    result: r.get("result").cloned(),
                    error: r.get("error").cloned(),
                });
            }
            let f = EvmFixture {
                host: endpoint.host().to_string(),
                chain_id,
                block,
                calls,
                note,
            };
            (root.join("evm"), to_canonical_json(&f).unwrap())
        }
        _ => usage(),
    };
    std::fs::create_dir_all(&path).unwrap();
    let file = path.join(format!("{name}.json"));
    std::fs::write(&file, text + "\n").unwrap();
    println!("{} (block {block})", file.display());
}
