//! TOK-4 (every Token-2022 extension parsed; unknown ones listed by type
//! number) and TOK-5 (mint, freeze, delegate and hook authorities resolved
//! like program authorities).

mod common;

use celebtir_core::snapshot::{TokenProgram, TokenProvenance};
use celebtir_core::{Address, Controller};
use collect_solana::fixtures::load;
use collect_solana::mint::decode_mint;
use collect_solana::reads::SolanaRpc;
use collect_solana::resolve::{KnownControllers, RegistryMultisig, Resolver};
use collect_solana::squads::SQUADS_V4_PROGRAM;
use collect_solana::token2022::{parse_extensions, token_facts};
use common::*;

const PAXOS: &str = "2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk";
const KAMINO_MS: &str = "6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM";
const KAMINO_VAULT: &str = "GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW";
const KLEND: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

fn a(s: &str) -> Address {
    Address::new(s)
}

fn tlv(t: u16, value: &[u8]) -> Vec<u8> {
    let mut v = t.to_le_bytes().to_vec();
    v.extend((value.len() as u16).to_le_bytes());
    v.extend(value);
    v
}

fn key(s: &str) -> [u8; 32] {
    collect_solana::pda::decode(&a(s)).unwrap()
}

#[test]
fn tok_4_pyusd_reports_permanent_delegate_and_transfer_hook_with_authorities() {
    let m = decode_mint(&load("mint-pyusd").read()).unwrap();
    let e = parse_extensions(&m.extension_data).unwrap();
    assert_eq!(e.permanent_delegate, Some(Some(a(PAXOS))));
    assert_eq!(
        e.transfer_hook,
        Some((Some(a(PAXOS)), None)),
        "hook authority set, no hook program"
    );
    assert_eq!(e.transfer_fee_authority, Some(Some(a(PAXOS))));
    assert_eq!(e.mint_close_authority, Some(Some(a(PAXOS))));
    assert_eq!(e.metadata_update_authority, Some(Some(a(PAXOS))));
    assert!(!e.default_frozen);
    assert_eq!(e.pause_authority, None);
    for name in [
        "confidential_transfer_mint",
        "confidential_transfer_fee_config",
        "metadata_pointer",
    ] {
        assert!(
            e.other.iter().any(|o| o == name),
            "missing {name}: {:?}",
            e.other
        );
    }
    assert!(e.unknown.is_empty());
}

#[test]
fn tok_4_unknown_extensions_are_listed_by_type_number() {
    let mut data = tlv(99, &[1, 2, 3]);
    data.extend(tlv(12, &[0u8; 32]));
    let e = parse_extensions(&data).unwrap();
    assert_eq!(e.unknown, vec![99]);
    assert_eq!(
        e.permanent_delegate,
        Some(None),
        "zero key = delegate renounced"
    );
}

#[test]
fn tok_4_pausable_and_default_frozen_are_decoded() {
    let mut pausable = key(PAXOS).to_vec();
    pausable.push(0);
    let mut data = tlv(26, &pausable);
    data.extend(tlv(6, &[2]));
    let e = parse_extensions(&data).unwrap();
    assert_eq!(e.pause_authority, Some(Some(a(PAXOS))));
    assert!(e.default_frozen);
}

#[test]
fn tok_4_truncated_tlv_is_an_error() {
    let mut data = tlv(12, &key(PAXOS));
    data.truncate(20);
    assert!(parse_extensions(&data).is_err());
    assert!(
        parse_extensions(&tlv(12, &[1, 2])).is_err(),
        "delegate value too short"
    );
}

fn kamino_node() -> SolanaRpc<FnTransport> {
    let mut accounts = vec![(
        KAMINO_MS.to_string(),
        SQUADS_V4_PROGRAM.to_string(),
        load("squads-multisig-kamino").data(),
    )];
    for n in ["program-kamino-lend", "programdata-kamino-lend"] {
        let f = load(n);
        accounts.push((
            f.address.to_string(),
            f.owner.clone().unwrap().to_string(),
            f.data(),
        ));
    }
    SolanaRpc::new(account_server(accounts))
}

#[tokio::test]
async fn tok_5_freeze_authority_held_by_a_squads_vault_shows_the_multisig_threshold() {
    let mut read = load("mint-usdc").read();
    read.account.as_mut().unwrap().data[50..82].copy_from_slice(&key(KAMINO_VAULT));
    let mint = decode_mint(&read).unwrap();
    let rpc = kamino_node();
    let known = KnownControllers::default();
    let resolver = Resolver::new(
        &rpc,
        &[RegistryMultisig {
            address: a(KAMINO_MS),
            vault_index: 0,
            program: celebtir_registry::MultisigProgram::SquadsV4,
        }],
        &known,
    );
    let t = token_facts(&rpc, &resolver, &mint, "USDC", TokenProvenance::Native)
        .await
        .unwrap()
        .0;
    assert!(
        matches!(
            t.freeze_authority.value(),
            Controller::Multisig {
                threshold: 5,
                voters: 10,
                ..
            }
        ),
        "{:?}",
        t.freeze_authority.value()
    );
    assert_eq!(t.freeze_authority.evidence()[0].field, "freeze_authority");
    assert_eq!(t.program, TokenProgram::SplToken);
}

#[tokio::test]
async fn tok_5_hook_program_upgrade_control_is_resolved() {
    // Edge case: a transfer hook pointing at an upgradeable program. Synthetic
    // mint: PYUSD's base with one TransferHook extension whose program is
    // Kamino Lend (upgradeable, controlled by Kamino's multisig).
    let mut read = load("mint-pyusd").read();
    let acct = read.account.as_mut().unwrap();
    acct.data.truncate(166);
    let mut hook = [0u8; 32].to_vec();
    hook.extend(key(KLEND));
    acct.data.extend(tlv(14, &hook));
    let mint = decode_mint(&read).unwrap();
    let rpc = kamino_node();
    let known = KnownControllers::default();
    let resolver = Resolver::new(
        &rpc,
        &[RegistryMultisig {
            address: a(KAMINO_MS),
            vault_index: 0,
            program: celebtir_registry::MultisigProgram::SquadsV4,
        }],
        &known,
    );
    let t = token_facts(&rpc, &resolver, &mint, "PYUSD", TokenProvenance::Native)
        .await
        .unwrap()
        .0;
    let h = t.extensions.value().transfer_hook.clone().unwrap();
    assert_eq!(h.program, Some(a(KLEND)));
    assert_eq!(h.authority, Controller::Immutable);
    assert!(
        matches!(
            h.program_controller,
            Some(Controller::Multisig {
                threshold: 5,
                voters: 10,
                ..
            })
        ),
        "{:?}",
        h.program_controller
    );
}

#[tokio::test]
async fn tok_5_pyusd_authorities_resolve_to_controllers() {
    let mint = decode_mint(&load("mint-pyusd").read()).unwrap();
    let rpc = SolanaRpc::new(account_server(vec![]));
    let known = KnownControllers::default();
    let resolver = Resolver::new(&rpc, &[], &known);
    let t = token_facts(&rpc, &resolver, &mint, "PYUSD", TokenProvenance::Native)
        .await
        .unwrap()
        .0;
    let e = t.extensions.value();
    for c in [
        &e.permanent_delegate,
        &e.transfer_fee_authority,
        &e.mint_close_authority,
        &e.metadata_update_authority,
    ] {
        let c = c.as_ref().unwrap();
        assert!(
            matches!(c, Controller::SingleKey { key } | Controller::Unrecognized { key, .. } if *key == a(PAXOS)),
            "{c:?}"
        );
    }
    assert_eq!(t.symbol, "PYUSD");
    assert!(
        t.extensions
            .evidence()
            .iter()
            .any(|ev| ev.field == "extensions")
    );
}
