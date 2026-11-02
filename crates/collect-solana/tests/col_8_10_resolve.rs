//! COL-8 (controller resolution) and COL-10 (config authorities resolved up to
//! 3 levels, loops detected).

mod common;

use std::collections::BTreeMap;

use celebtir_core::{Address, Chain, Controller, Evidence, MultisigKind};
use collect_solana::fixtures::load;
use collect_solana::pda;
use collect_solana::reads::SolanaRpc;
use collect_solana::resolve::{KnownControllers, RegistryMultisig, Resolver};
use collect_solana::squads::{SQUADS_V4_PROGRAM, permission};
use common::*;

const KAMINO_MS: &str = "6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM";
const KAMINO_VAULT: &str = "GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW";
const REALMS: &str = "GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw";
const VOTE: u8 = permission::VOTE;

fn a(s: &str) -> Address {
    Address::new(s)
}

fn origin() -> Evidence {
    Evidence::new(
        Chain::Solana,
        a("9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA"),
        458_009_921,
        "upgrade_authority",
    )
}

fn known() -> KnownControllers {
    KnownControllers::new(BTreeMap::from([(a(REALMS), "spl-governance".to_string())]))
}

fn server(extra: Vec<(String, String, Vec<u8>)>) -> SolanaRpc<FnTransport> {
    let ms = load("squads-multisig-kamino");
    let mut accounts = vec![(
        KAMINO_MS.to_string(),
        SQUADS_V4_PROGRAM.to_string(),
        ms.data(),
    )];
    accounts.extend(extra);
    SolanaRpc::new(account_server(accounts))
}

fn squads_account(address: &Address, data: Vec<u8>) -> (String, String, Vec<u8>) {
    (address.to_string(), SQUADS_V4_PROGRAM.to_string(), data)
}

fn reg(address: &Address) -> RegistryMultisig {
    RegistryMultisig {
        address: address.clone(),
        vault_index: 0,
        program: celebtir_registry::MultisigProgram::SquadsV4,
    }
}

fn vault(ms: &Address) -> Address {
    pda::squads_vault(ms, 0).unwrap()
}

fn raw(addr: &Address) -> [u8; 32] {
    pda::decode(addr).unwrap()
}

/// An off-curve address that is not anyone's vault.
fn pda_like(seed: u8) -> Address {
    let (bytes, _) = pda::find_program_address(&[&[seed]], &[7u8; 32]).unwrap();
    pda::encode(bytes)
}

/// A wallet: a member key from Kamino's multisig that is on the curve.
fn wallet() -> Address {
    let m =
        collect_solana::squads::decode_multisig(&load("squads-multisig-kamino").read()).unwrap();
    m.members
        .into_iter()
        .map(|m| m.key)
        .find(|k| pda::is_on_curve(&raw(k)))
        .unwrap()
}

#[tokio::test]
async fn col_8_kamino_authority_resolves_to_its_squads_multisig() {
    let rpc = server(vec![]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&a(KAMINO_MS))], &k);
    let res = r.resolve(&a(KAMINO_VAULT), origin()).await.unwrap();
    let Controller::Multisig { signers, .. } = res.fact.value() else {
        panic!("{:?}", res.fact.value());
    };
    assert_eq!(
        res.fact.value(),
        &Controller::Multisig {
            kind: MultisigKind::SquadsV4,
            address: a(KAMINO_MS),
            threshold: 5,
            voters: 10,
            time_lock_secs: 86_400,
            config_authority: None,
            signers: signers.clone(),
            nested_signers: Default::default(),
        }
    );
    assert!(res.review_needed.is_empty());
    let ev = res.fact.evidence();
    assert_eq!(ev[0], origin());
    assert!(
        ev.iter()
            .any(|e| e.address == a(KAMINO_MS) && e.slot == 458_100_000)
    );
}

#[tokio::test]
async fn col_8_wallet_resolves_to_single_key_without_a_read() {
    let rpc = server(vec![]);
    let k = known();
    let r = Resolver::new(&rpc, &[], &k);
    let w = wallet();
    let res = r.resolve(&w, origin()).await.unwrap();
    assert_eq!(res.fact.value(), &Controller::SingleKey { key: w });
    assert!(rpc.client().transport().bodies().is_empty());
}

#[tokio::test]
async fn col_8_unknown_pda_is_a_pda_with_its_owner_and_flags_review() {
    let key = pda_like(1);
    let owner = "SomeUnknownProgram1111111111111111111111111";
    let rpc = server(vec![(key.to_string(), owner.into(), vec![1, 2, 3])]);
    let k = known();
    let r = Resolver::new(&rpc, &[], &k);
    let res = r.resolve(&key, origin()).await.unwrap();
    assert_eq!(
        res.fact.value(),
        &Controller::Pda {
            key: key.clone(),
            owner_program: Some(a(owner))
        }
    );
    assert_eq!(res.review_needed.len(), 1);
}

#[tokio::test]
async fn col_8_governance_pda_is_a_pda_with_known_owner_and_no_review_flag() {
    let key = pda_like(2);
    let rpc = server(vec![(key.to_string(), REALMS.into(), vec![0; 10])]);
    let k = known();
    let r = Resolver::new(&rpc, &[], &k);
    let res = r.resolve(&key, origin()).await.unwrap();
    assert!(matches!(
        res.fact.value(),
        Controller::Pda { owner_program: Some(o), .. } if *o == a(REALMS)
    ));
    assert!(res.review_needed.is_empty());
    assert_eq!(k.name(&a(REALMS)), Some("spl-governance"));
}

#[tokio::test]
async fn col_8_vault_of_an_unregistered_multisig_is_a_pda_and_flags_review() {
    // Edge case: Kamino's vault, but the registry does not list its multisig.
    let rpc = server(vec![]);
    let k = known();
    let r = Resolver::new(&rpc, &[], &k);
    let res = r.resolve(&a(KAMINO_VAULT), origin()).await.unwrap();
    assert_eq!(
        res.fact.value(),
        &Controller::Pda {
            key: a(KAMINO_VAULT),
            owner_program: None
        }
    );
    assert_eq!(res.review_needed.len(), 1, "registry review needed");
}

#[tokio::test]
async fn col_8_undecodable_multisig_is_unavailable_never_a_default() {
    let ms = pda_like(3);
    let rpc = server(vec![squads_account(&ms, vec![0; 12])]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&ms)], &k);
    let res = r.resolve(&vault(&ms), origin()).await.unwrap();
    assert!(
        matches!(res.fact.value(), Controller::Unavailable { .. }),
        "{:?}",
        res.fact.value()
    );
}

#[tokio::test]
async fn col_8_multisig_that_cannot_reach_its_threshold_is_cannot_act() {
    let ms = pda_like(4);
    let rpc = server(vec![squads_account(
        &ms,
        multisig_bytes([0; 32], 3, 0, &[([1; 32], VOTE), ([2; 32], 1)]),
    )]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&ms)], &k);
    let res = r.resolve(&vault(&ms), origin()).await.unwrap();
    assert_eq!(
        res.fact.value(),
        &Controller::CannotAct {
            address: ms,
            threshold: 3,
            voters: 1
        }
    );
}

/// Two registry multisigs; `a`'s config authority is `b`'s vault.
#[tokio::test]
async fn col_10_multisig_controlled_by_another_multisig_resolves_both() {
    let (ma, mb) = (pda_like(10), pda_like(11));
    let rpc = server(vec![
        squads_account(
            &ma,
            multisig_bytes(
                raw(&vault(&mb)),
                2,
                0,
                &[([1; 32], VOTE), ([2; 32], VOTE), ([3; 32], VOTE)],
            ),
        ),
        squads_account(
            &mb,
            multisig_bytes(
                [0; 32],
                4,
                86_400,
                &[
                    ([4; 32], VOTE),
                    ([5; 32], VOTE),
                    ([6; 32], VOTE),
                    ([7; 32], VOTE),
                    ([8; 32], VOTE),
                ],
            ),
        ),
    ]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&ma), reg(&mb)], &k);
    let res = r.resolve(&vault(&ma), origin()).await.unwrap();
    match res.fact.value() {
        Controller::Multisig {
            address,
            threshold: 2,
            voters: 3,
            config_authority: Some(ca),
            ..
        } => {
            assert_eq!(*address, ma);
            assert!(matches!(
                **ca,
                Controller::Multisig { ref address, threshold: 4, voters: 5, time_lock_secs: 86_400, .. } if *address == mb
            ));
        }
        other => panic!("{other:?}"),
    }
    assert!(
        res.fact.evidence().iter().any(|e| e.address == mb),
        "evidence covers both multisigs"
    );
}

#[tokio::test]
async fn col_10_single_key_config_authority_is_resolved() {
    let ms = pda_like(12);
    let w = wallet();
    let rpc = server(vec![squads_account(
        &ms,
        multisig_bytes(raw(&w), 1, 0, &[([1; 32], VOTE)]),
    )]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&ms)], &k);
    let res = r.resolve(&vault(&ms), origin()).await.unwrap();
    assert!(matches!(
        res.fact.value(),
        Controller::Multisig { config_authority: Some(ca), .. } if **ca == Controller::SingleKey { key: w.clone() }
    ));
}

#[tokio::test]
async fn col_10_a_config_authority_loop_stops_with_an_error() {
    let (ma, mb) = (pda_like(20), pda_like(21));
    let rpc = server(vec![
        squads_account(
            &ma,
            multisig_bytes(raw(&vault(&mb)), 1, 0, &[([1; 32], VOTE)]),
        ),
        squads_account(
            &mb,
            multisig_bytes(raw(&vault(&ma)), 1, 0, &[([2; 32], VOTE)]),
        ),
    ]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&ma), reg(&mb)], &k);
    let res = r.resolve(&vault(&ma), origin()).await.unwrap();
    let text = format!("{:?}", res.fact.value());
    assert!(text.contains("loop"), "{text}");
}

#[tokio::test]
async fn col_10_chains_deeper_than_3_levels_stop() {
    let ms: Vec<Address> = (30..35).map(pda_like).collect();
    let mut accounts = Vec::new();
    for i in 0..5 {
        let ca = if i + 1 < 5 {
            raw(&vault(&ms[i + 1]))
        } else {
            [0; 32]
        };
        accounts.push(squads_account(
            &ms[i],
            multisig_bytes(ca, 1, 0, &[([i as u8 + 1; 32], VOTE)]),
        ));
    }
    let rpc = server(accounts);
    let k = known();
    let regs: Vec<RegistryMultisig> = ms.iter().map(reg).collect();
    let r = Resolver::new(&rpc, &regs, &k);
    let res = r.resolve(&vault(&ms[0]), origin()).await.unwrap();
    let text = format!("{:?}", res.fact.value());
    assert!(text.contains("deeper than 3"), "{text}");
}

#[tokio::test]
async fn ui_20_a_multisig_publishes_its_signers_sorted_one_per_voter() {
    let rpc = server(vec![]);
    let k = known();
    let r = Resolver::new(&rpc, &[reg(&a(KAMINO_MS))], &k);
    let res = r.resolve(&a(KAMINO_VAULT), origin()).await.unwrap();
    let Controller::Multisig {
        signers, voters, ..
    } = res.fact.value()
    else {
        panic!("{:?}", res.fact.value());
    };
    assert_eq!(signers.len(), usize::from(*voters));
    let mut sorted = signers.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(&sorted, signers, "sorted and unique");
}

/// COL-8 / COL-9: the SPL Stake Pool's upgrade authority is authority index 1
/// of a Squads v3 multisig listed in the registry.
#[tokio::test]
async fn col_8_squads_v3_authority_resolves_to_its_multisig() {
    const V3_MS: &str = "3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW";
    const V3_AUTHORITY: &str = "dPpFfahSWhm5M2RB3nVaHsXfnh9ooz5bWtR3WeUguDH";
    let ms = load("squads-v3-multisig-spl-stake-pool");
    let rpc = SolanaRpc::new(account_server(vec![(
        V3_MS.to_string(),
        collect_solana::squads::SQUADS_V3_PROGRAM.to_string(),
        ms.data(),
    )]));
    let k = known();
    let reg = RegistryMultisig {
        address: a(V3_MS),
        vault_index: 1,
        program: celebtir_registry::MultisigProgram::SquadsV3,
    };
    let r = Resolver::new(&rpc, &[reg], &k);
    let res = r.resolve(&a(V3_AUTHORITY), origin()).await.unwrap();
    let Controller::Multisig {
        kind,
        address,
        threshold,
        voters,
        time_lock_secs,
        config_authority,
        signers,
        ..
    } = res.fact.value()
    else {
        panic!("{:?}", res.fact.value());
    };
    assert_eq!(*kind, MultisigKind::SquadsV3);
    assert_eq!(address, &a(V3_MS));
    assert_eq!((*threshold, *voters, *time_lock_secs), (6, 10, 0));
    assert_eq!(config_authority, &None);
    assert_eq!(signers.len(), 10);
    assert!(res.review_needed.is_empty(), "{:?}", res.review_needed);
    assert!(res.fact.evidence().iter().any(|e| e.address == a(V3_MS)));
}
