//! HCOL-8: Sourcify first, then Etherscan v2 with an optional key; the key
//! never reaches a fact. Mocked HTTP, synthetic chain (see `common`).

mod common;

use std::sync::Mutex;

use celebtir_core::Address;
use celebtir_core::snapshot::CodeTransparency;
use celebtir_registry::ProxyKind;
use collect_hyperliquid::evm::{HyperEvm, default_config};
use collect_hyperliquid::verify::verified_source;
use collect_solana::rpc::{RpcClient, TransportError};
use collect_solana::verify::HttpGet;
use common::*;
use serde_json::{Value, json};

const KEY_SECRET: &str = "SECRETKEY123";

struct Http {
    sourcify: Result<Value, TransportError>,
    etherscan: Value,
    urls: Mutex<Vec<String>>,
}

impl HttpGet for Http {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        self.urls.lock().unwrap().push(url.to_string());
        if url.contains("sourcify") {
            self.sourcify.clone()
        } else {
            Ok(self.etherscan.clone())
        }
    }
}

fn http(sourcify: Result<Value, TransportError>, etherscan_source: &str) -> Http {
    Http {
        sourcify,
        etherscan: json!({"status": "1", "result": [{"SourceCode": etherscan_source}]}),
        urls: Mutex::new(vec![]),
    }
}

async fn run(h: &Http, key: Option<&str>) -> CodeTransparency {
    let impl_slot = format!("0x{}", collect_hyperliquid::resolve::IMPLEMENTATION_SLOT);
    let f = Chain::default()
        .storage(PROXY, &impl_slot, &word_of_address(ADMIN))
        .fixture();
    let evm = HyperEvm::connect(RpcClient::new(f.replay(), default_config()))
        .await
        .unwrap();
    let fact = verified_source(&evm, h, key, &Address::new(PROXY), ProxyKind::Eip1967).await;
    let json = serde_json::to_string(&fact).unwrap();
    assert!(!json.contains(KEY_SECRET), "{json}");
    assert_eq!(
        fact.evidence()[0].address.as_str(),
        ADMIN,
        "the implementation is checked"
    );
    fact.value().clone()
}

#[tokio::test(start_paused = true)]
async fn hcol_8_sourcify_match_marks_verified() {
    let h = http(Ok(json!({"match": "exact_match"})), "");
    assert!(matches!(
        run(&h, None).await,
        CodeTransparency::Verified { .. }
    ));
    assert_eq!(h.urls.lock().unwrap().len(), 1, "Etherscan not needed");
}

#[tokio::test(start_paused = true)]
async fn hcol_8_etherscan_is_asked_only_with_a_key() {
    let h = http(Ok(json!({"match": null})), "contract C {}");
    assert!(matches!(
        run(&h, None).await,
        CodeTransparency::Unavailable { .. }
    ));
    assert!(
        h.urls
            .lock()
            .unwrap()
            .iter()
            .all(|u| !u.contains("etherscan"))
    );

    let h = http(Ok(json!({"match": null})), "contract C {}");
    assert!(matches!(
        run(&h, Some(KEY_SECRET)).await,
        CodeTransparency::Verified { .. }
    ));
}

#[tokio::test(start_paused = true)]
async fn hcol_8_both_say_unverified_means_no_public_source() {
    let h = http(Err(TransportError::Status(404)), "");
    assert_eq!(
        run(&h, Some(KEY_SECRET)).await,
        CodeTransparency::NoPublicSource
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_8_outage_is_unknown_and_never_leaks_the_key() {
    let h = Http {
        sourcify: Err(TransportError::Status(503)),
        etherscan: json!({"status": "0", "message": format!("bad key {KEY_SECRET}"), "result": "x"}),
        urls: Mutex::new(vec![]),
    };
    assert!(matches!(
        run(&h, Some(KEY_SECRET)).await,
        CodeTransparency::Unavailable { .. }
    ));
}
