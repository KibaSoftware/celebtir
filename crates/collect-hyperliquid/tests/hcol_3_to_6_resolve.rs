//! HCOL-3 (proxy reads), HCOL-4 (classification), HCOL-5 (Safe modules),
//! HCOL-6 (timelock proposers and admin). The Kinetiq capture is real;
//! the other cases are synthetic fixtures built here, named as such.

use std::path::Path;

use celebtir_core::{Address, Controller, MultisigKind};
use celebtir_registry::{ProxyKind, TimelockEntry};
use collect_hyperliquid::evm::{HyperEvm, default_config, hex};
use collect_hyperliquid::fixtures::EvmFixture;
use collect_hyperliquid::resolve::{EvmKnown, Resolver, keccak, selector};
use collect_solana::rpc::RpcClient;

mod common;

use common::*;

async fn run(f: EvmFixture, k: &EvmKnown, contract: &str, proxy: ProxyKind) -> Controller {
    let evm = HyperEvm::connect(RpcClient::new(f.replay(), default_config()))
        .await
        .unwrap();
    let r = Resolver {
        evm: &evm,
        known: k,
    }
    .upgrade_control(&Address::new(contract), proxy)
    .await;
    assert!(!r.evidence.is_empty());
    assert!(r.evidence.iter().all(|e| e.slot == evm.block()));
    r.controller
}

#[test]
fn hcol_3_selectors_match_their_known_values() {
    assert_eq!(hex(&selector("owner()")), "0x8da5cb5b");
    assert_eq!(hex(&selector("getThreshold()")), "0xe75235b8");
    assert_eq!(hex(&selector("getOwners()")), "0xa0e67e2b");
    // EIP-1967: keccak("eip1967.proxy.admin") - 1.
    let mut k = keccak("eip1967.proxy.admin");
    k[31] -= 1;
    assert_eq!(hex(&k), admin_slot());
}

#[tokio::test(start_paused = true)]
async fn hcol_3_transparent_proxy_with_proxy_admin_resolves_to_its_safe() {
    // Real capture: Kinetiq kHYPE StakingManager.
    let f = EvmFixture::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/hyperliquid/evm/kinetiq-khype-manager.json"),
    )
    .unwrap();
    let controllers = EvmKnown::controllers_from_yaml(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registry/controllers.yaml"),
        )
        .unwrap(),
    )
    .unwrap();
    let k = EvmKnown::new(&controllers, &[]);
    let c = run(
        f,
        &k,
        "0x393D0B87Ed38fc779FD9611144aE649BA6082109",
        ProxyKind::Eip1967,
    )
    .await;
    match c {
        Controller::Multisig {
            kind,
            threshold,
            voters,
            config_authority,
            signers,
            ..
        } => {
            assert_eq!(kind, MultisigKind::Safe);
            assert_eq!((threshold, voters), (4, 8));
            assert!(config_authority.is_none());
            assert_eq!(signers.len(), 8);
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn hcol_3_uups_proxy_resolves_through_owner() {
    let f = Chain::default()
        .sig(PROXY, "owner()", "", Some(&word_of_address(KEY)))
        .code(KEY, "0x")
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Uups).await;
    assert_eq!(
        c,
        Controller::SingleKey {
            key: Address::new(KEY)
        }
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_3_plain_contract_is_immutable() {
    let c = run(
        Chain::default().fixture(),
        &known(&[]),
        PROXY,
        ProxyKind::None,
    )
    .await;
    assert_eq!(c, Controller::Immutable);
}

#[tokio::test(start_paused = true)]
async fn hcol_3_empty_admin_slot_is_unavailable_not_immutable() {
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &uint(0))
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Eip1967).await;
    assert!(
        matches!(&c, Controller::Unavailable { reason } if reason.contains("admin slot empty")),
        "{c:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_4_eoa_admin_is_a_single_key() {
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(KEY))
        .code(KEY, "0x")
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Eip1967).await;
    assert_eq!(
        c,
        Controller::SingleKey {
            key: Address::new(KEY)
        }
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_4_safe_3_of_5() {
    let owners = [KEY, ADMIN, MODULE, TIMELOCK, PROXY];
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(SAFE))
        .safe(SAFE, 3, &owners, &[])
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Eip1967).await;
    assert!(
        matches!(
            c,
            Controller::Multisig {
                kind: MultisigKind::Safe,
                threshold: 3,
                voters: 5,
                ..
            }
        ),
        "{c:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_4_unknown_contract_is_unrecognized() {
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(ADMIN))
        .contract(ADMIN)
        .sig(ADMIN, "owner()", "", None)
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Eip1967).await;
    assert_eq!(
        c,
        Controller::Unrecognized {
            key: Address::new(ADMIN),
            owner_program: None
        }
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_4_ownership_loop_is_detected() {
    // ADMIN is owned by MODULE, which is owned by ADMIN.
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(ADMIN))
        .contract(ADMIN)
        .sig(ADMIN, "owner()", "", Some(&word_of_address(MODULE)))
        .contract(MODULE)
        .sig(MODULE, "owner()", "", Some(&word_of_address(ADMIN)))
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Eip1967).await;
    assert!(
        matches!(&c, Controller::Unavailable { reason } if reason.contains("loop")),
        "{c:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn hcol_5_safe_module_is_the_config_authority() {
    let f = Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(SAFE))
        .safe(SAFE, 3, &[KEY, ADMIN, TIMELOCK, PROXY, MODULE], &[MODULE])
        .code(MODULE, "0x")
        .fixture();
    let c = run(f, &known(&[]), PROXY, ProxyKind::Eip1967).await;
    match c {
        Controller::Multisig {
            config_authority: Some(ca),
            ..
        } => assert_eq!(
            *ca,
            Controller::SingleKey {
                key: Address::new(MODULE)
            }
        ),
        other => panic!("{other:?}"),
    }
}

fn timelock_entry() -> TimelockEntry {
    TimelockEntry {
        address: Address::new(TIMELOCK),
        proposers: vec![Address::new(SAFE)],
        source: "https://example.org".parse().unwrap(),
        shared_with: vec![],
    }
}

fn timelock_chain(proposer_ok: bool, safe_is_admin: bool) -> Chain {
    let role = hex(&keccak("PROPOSER_ROLE"));
    Chain::default()
        .storage(PROXY, &admin_slot(), &word_of_address(TIMELOCK))
        .contract(TIMELOCK)
        .sig(TIMELOCK, "getMinDelay()", "", Some(&uint(172_800)))
        .sig(
            TIMELOCK,
            "hasRole(bytes32,address)",
            &format!("{}{}", &role[2..], pad(SAFE)),
            Some(&uint(u64::from(proposer_ok))),
        )
        .sig(
            TIMELOCK,
            "hasRole(bytes32,address)",
            &format!("{}{}", "00".repeat(32), pad(SAFE)),
            Some(&uint(u64::from(safe_is_admin))),
        )
        .safe(
            SAFE,
            5,
            &[
                KEY,
                ADMIN,
                MODULE,
                PROXY,
                SINGLETON,
                "0x0000000000000000000000000000000000000002",
                "0x0000000000000000000000000000000000000003",
                "0x0000000000000000000000000000000000000004",
            ],
            &[],
        )
}

#[tokio::test(start_paused = true)]
async fn hcol_6_timelock_with_confirmed_safe_proposer() {
    let k = known(&[timelock_entry()]);
    let c = run(
        timelock_chain(true, false).fixture(),
        &k,
        PROXY,
        ProxyKind::Eip1967,
    )
    .await;
    match c {
        Controller::Timelock {
            min_delay_secs,
            proposer,
            admin,
            ..
        } => {
            assert_eq!(min_delay_secs, 172_800);
            assert!(matches!(
                *proposer,
                Controller::Multisig {
                    threshold: 5,
                    voters: 8,
                    ..
                }
            ));
            assert!(admin.is_none());
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn hcol_6_unconfirmed_proposer_is_unavailable() {
    let k = known(&[timelock_entry()]);
    let c = run(
        timelock_chain(false, false).fixture(),
        &k,
        PROXY,
        ProxyKind::Eip1967,
    )
    .await;
    match c {
        Controller::Timelock { proposer, .. } => {
            assert!(
                matches!(*proposer, Controller::Unavailable { .. }),
                "{proposer:?}"
            )
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn hcol_6_outside_admin_is_recorded() {
    let k = known(&[timelock_entry()]);
    let c = run(
        timelock_chain(true, true).fixture(),
        &k,
        PROXY,
        ProxyKind::Eip1967,
    )
    .await;
    match c {
        Controller::Timelock { admin: Some(a), .. } => {
            assert!(matches!(*a, Controller::Multisig { .. }), "{a:?}")
        }
        other => panic!("{other:?}"),
    }
}
