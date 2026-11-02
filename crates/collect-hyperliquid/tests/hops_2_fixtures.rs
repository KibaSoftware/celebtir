//! HOPS-2: Hyperliquid fixtures are recorded with their block and host, and
//! tests replay them with no network.

use std::path::{Path, PathBuf};

use collect_hyperliquid::evm::{CHAIN_ID, HyperEvm, default_config};
use collect_hyperliquid::fixtures::{EvmFixture, InfoFixture};
use collect_solana::rpc::RpcClient;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/hyperliquid")
}

fn files(dir: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    v.sort();
    v
}

#[test]
fn hops_2_every_fixture_has_a_block_and_a_host_only() {
    let info = files("info");
    let evm = files("evm");
    assert!(!info.is_empty() && !evm.is_empty());
    for p in &info {
        let f = InfoFixture::load(p).unwrap();
        assert!(f.anchor_block > 0, "{}", p.display());
        assert!(!f.host.contains('/'), "{}: host only", p.display());
        assert!(!f.note.is_empty(), "{}", p.display());
    }
    for p in &evm {
        let f = EvmFixture::load(p).unwrap();
        assert_eq!(f.chain_id, CHAIN_ID, "{}", p.display());
        assert!(f.block > 0, "{}", p.display());
        assert!(!f.host.contains('/'), "{}: host only", p.display());
        let tag = format!("0x{:x}", f.block);
        for c in &f.calls {
            assert_eq!(
                c.params.as_array().unwrap().last().unwrap(),
                &serde_json::json!(tag),
                "{}: every call at the pinned block",
                p.display()
            );
        }
    }
}

#[test]
fn hops_2_perp_dexs_fixture_lists_hip3_dexs() {
    let f = InfoFixture::load(&root().join("info/perp-dexs.json")).unwrap();
    let dexs = f.response.as_array().unwrap();
    assert!(dexs[0].is_null(), "entry 0 is the main Hyperliquid DEX");
    assert!(dexs.len() > 1);
    for d in &dexs[1..] {
        assert!(d["name"].is_string() && d["deployer"].is_string(), "{d}");
    }
}

#[tokio::test(start_paused = true)]
async fn hops_2_replay_serves_recorded_calls_offline() {
    let f = EvmFixture::load(&root().join("evm/safe-singletons.json")).unwrap();
    let evm = HyperEvm::connect(RpcClient::new(f.replay(), default_config()))
        .await
        .unwrap();
    assert_eq!(evm.block(), f.block);
    // Recorded with checksummed case; looked up in lower case.
    let code = evm
        .code("0x41675c099f32341bf84bfc5382af534df5c7461a")
        .await
        .unwrap();
    assert!(!code.is_empty(), "Safe v1.4.1 is deployed on chain 999");
    // A call that was never recorded fails instead of going online.
    assert!(
        evm.code("0x0000000000000000000000000000000000000001")
            .await
            .is_err()
    );
}
