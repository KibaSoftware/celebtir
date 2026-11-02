//! A transport that answers every request with a closure and records bodies.
#![allow(dead_code)]

use std::sync::Mutex;

use collect_solana::rpc::{RpcClient, RpcConfig, Transport, TransportError};
use serde_json::{Value, json};

type Handler = Box<dyn Fn(&Value) -> Value + Send + Sync>;

pub struct FnTransport {
    handler: Handler,
    pub bodies: Mutex<Vec<Value>>,
}

impl FnTransport {
    pub fn new(handler: impl Fn(&Value) -> Value + Send + Sync + 'static) -> Self {
        Self {
            handler: Box::new(handler),
            bodies: Mutex::new(Vec::new()),
        }
    }

    pub fn bodies(&self) -> Vec<Value> {
        self.bodies.lock().unwrap().clone()
    }
}

impl Transport for FnTransport {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        self.bodies.lock().unwrap().push(body.clone());
        Ok(json!({"jsonrpc": "2.0", "id": 1, "result": (self.handler)(&body)}))
    }
}

pub fn client(handler: impl Fn(&Value) -> Value + Send + Sync + 'static) -> RpcClient<FnTransport> {
    RpcClient::new(
        FnTransport::new(handler),
        RpcConfig {
            requests_per_sec: 1000,
            ..RpcConfig::default()
        },
    )
}

/// A getMultipleAccounts result at `slot` where every account is owned by
/// `owner` and holds `data`.
pub fn accounts_result(addresses: &Value, slot: u64, owner: &str, data: &[u8]) -> Value {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
    let n = addresses.as_array().unwrap().len();
    json!({
        "context": {"slot": slot},
        "value": (0..n).map(|_| json!({
            "data": [b64, "base64"], "executable": false, "lamports": 1, "owner": owner, "rentEpoch": 0, "space": data.len()
        })).collect::<Vec<_>>()
    })
}

/// A mock node holding accounts by address; honours dataSlice; unknown
/// addresses read as missing.
pub fn account_server(accounts: Vec<(String, String, Vec<u8>)>) -> RpcClient<FnTransport> {
    use base64::Engine;
    client(move |body: &Value| {
        assert_eq!(
            body["method"], "getMultipleAccounts",
            "unexpected {}",
            body["method"]
        );
        let slice = &body["params"][1]["dataSlice"];
        let values: Vec<Value> = body["params"][0]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| match accounts.iter().find(|(addr, _, _)| addr == a.as_str().unwrap()) {
                None => Value::Null,
                Some((_, owner, data)) => {
                    let mut d = data.clone();
                    if let (Some(o), Some(l)) = (slice["offset"].as_u64(), slice["length"].as_u64()) {
                        d = d.into_iter().skip(o as usize).take(l as usize).collect();
                    }
                    json!({"data": [base64::engine::general_purpose::STANDARD.encode(d), "base64"],
                           "owner": owner, "lamports": 1, "executable": false})
                }
            })
            .collect();
        json!({"context": {"slot": 458_100_000u64}, "value": values})
    })
}

/// Squads v4 Multisig bytes (same layout as the decoder).
pub fn multisig_bytes(
    config_authority: [u8; 32],
    threshold: u16,
    time_lock: u32,
    members: &[([u8; 32], u8)],
) -> Vec<u8> {
    let mut d = collect_solana::squads::MULTISIG_DISCRIMINATOR.to_vec();
    d.extend([9u8; 32]);
    d.extend(config_authority);
    d.extend(threshold.to_le_bytes());
    d.extend(time_lock.to_le_bytes());
    d.extend([0u8; 16]);
    d.push(0);
    d.push(255);
    d.extend((members.len() as u32).to_le_bytes());
    for (k, m) in members {
        d.extend(k);
        d.push(*m);
    }
    d
}
