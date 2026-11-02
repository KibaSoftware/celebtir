//! Launch audit F10: an unlisted TimelockController publishes its delay, and
//! a UUPS proxy guarded by a role registry resolves through the registry's
//! owner. Synthetic fixtures, named as such.

use celebtir_core::{Address, Controller};
use celebtir_registry::ProxyKind;
use collect_hyperliquid::evm::{HyperEvm, default_config};
use collect_hyperliquid::fixtures::EvmFixture;
use collect_hyperliquid::resolve::Resolver;
use collect_solana::rpc::RpcClient;

mod common;

use common::*;

async fn run(f: EvmFixture, contract: &str, proxy: ProxyKind) -> Controller {
    let evm = HyperEvm::connect(RpcClient::new(f.replay(), default_config()))
        .await
        .unwrap();
    let k = known(&[]);
    let r = Resolver {
        evm: &evm,
        known: &k,
    }
    .upgrade_control(&Address::new(contract), proxy)
    .await;
    assert!(!r.evidence.is_empty());
    r.controller
}

#[tokio::test(start_paused = true)]
async fn aud_10_unlisted_timelock_publishes_its_delay() {
    // stHYPE's shape: proxy admin → ProxyAdmin → owner() is a timelock.
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(ADMIN))
        .contract(ADMIN)
        .sig(ADMIN, "owner()", "", Some(&word_of_address(TIMELOCK)))
        .contract(TIMELOCK)
        .sig(TIMELOCK, "owner()", "", None)
        .sig(TIMELOCK, "getMinDelay()", "", Some(&uint(172_800)))
        .fixture();
    match run(f, PROXY, ProxyKind::Eip1967).await {
        Controller::Timelock {
            address,
            min_delay_secs,
            proposer,
            admin,
        } => {
            assert_eq!(address.as_str(), TIMELOCK);
            assert_eq!(min_delay_secs, 172_800);
            assert!(matches!(*proposer, Controller::Unavailable { .. }));
            assert!(admin.is_none());
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn aud_10_role_registry_proxy_resolves_through_the_registry_owner() {
    // Hyperbeat's shape: `_authorizeUpgrade` → roleRegistry.onlyProtocolUpgrader.
    let f = Chain::default()
        .sig(PROXY, "roleRegistry()", "", Some(&word_of_address(ADMIN)))
        .sig(ADMIN, "owner()", "", Some(&word_of_address(KEY)))
        .code(KEY, "0x")
        .fixture();
    assert_eq!(
        run(f, PROXY, ProxyKind::UupsRoleRegistry).await,
        Controller::SingleKey {
            key: Address::new(KEY)
        }
    );
}

#[tokio::test(start_paused = true)]
async fn aud_10_role_registry_without_an_owner_is_unavailable() {
    let f = Chain::default()
        .sig(PROXY, "roleRegistry()", "", Some(&word_of_address(ADMIN)))
        .sig(ADMIN, "owner()", "", None)
        .fixture();
    assert!(matches!(
        run(f, PROXY, ProxyKind::UupsRoleRegistry).await,
        Controller::Unavailable { .. }
    ));
}
