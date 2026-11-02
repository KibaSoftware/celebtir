//! Edge cases from the spec's table that P3 code decides. The full mapping of
//! all 18 rows to tests (or the later slice that owns them) is in
//! docs/edge-cases.md.

mod common;

use std::collections::BTreeMap;

use celebtir_core::snapshot::{FactsFile, ProgramFacts, ProgramRole, ProtocolFacts};
use celebtir_core::{Address, Chain, Controller, Evidence};
use collect_solana::control::upgrade_controls;
use collect_solana::fixtures::load;
use collect_solana::pda;
use collect_solana::reads::SolanaRpc;
use collect_solana::resolve::{KnownControllers, RegistryMultisig, Resolver};
use collect_solana::squads::{SQUADS_V4_PROGRAM, permission};
use common::*;

/// Squads v3 program ID; confirm it in `registry/controllers.yaml` (P5).
const SQUADS_V3: &str = "SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu";

fn a(s: &str) -> Address {
    Address::new(s)
}

fn origin() -> Evidence {
    Evidence::new(
        Chain::Solana,
        a("9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA"),
        1,
        "upgrade_authority",
    )
}

fn pda_like(seed: u8) -> Address {
    pda::encode(pda::find_program_address(&[&[seed]], &[3u8; 32]).unwrap().0)
}

#[tokio::test]
async fn edge_squads_v3_authority_is_a_pda_with_its_owner_named() {
    let key = pda_like(1);
    let rpc = SolanaRpc::new(account_server(vec![(
        key.to_string(),
        SQUADS_V3.into(),
        vec![0; 8],
    )]));
    let known = KnownControllers::new(BTreeMap::from([(a(SQUADS_V3), "squads-v3".into())]));
    let r = Resolver::new(&rpc, &[], &known);
    let res = r.resolve(&key, origin()).await.unwrap();
    assert_eq!(
        res.fact.value(),
        &Controller::Pda {
            key,
            owner_program: Some(a(SQUADS_V3))
        }
    );
    assert!(
        res.review_needed.is_empty(),
        "a known owner needs no registry review"
    );
    assert_eq!(known.name(&a(SQUADS_V3)), Some("squads-v3"));
}

#[tokio::test]
async fn edge_a_member_that_is_a_multisig_counts_as_one_voter() {
    // Multisig A's members: one wallet and the vault of multisig B.
    let (ma, mb) = (pda_like(10), pda_like(11));
    let vault_b = pda::decode(&pda::squads_vault(&mb, 0).unwrap()).unwrap();
    let rpc = SolanaRpc::new(account_server(vec![(
        ma.to_string(),
        SQUADS_V4_PROGRAM.into(),
        multisig_bytes(
            [0; 32],
            2,
            0,
            &[([1; 32], permission::VOTE), (vault_b, permission::VOTE)],
        ),
    )]));
    let known = KnownControllers::default();
    let r = Resolver::new(
        &rpc,
        &[RegistryMultisig {
            address: ma.clone(),
            vault_index: 0,
            program: celebtir_registry::MultisigProgram::SquadsV4,
        }],
        &known,
    );
    let res = r
        .resolve(&pda::squads_vault(&ma, 0).unwrap(), origin())
        .await
        .unwrap();
    assert!(
        matches!(
            res.fact.value(),
            Controller::Multisig {
                voters: 2,
                threshold: 2,
                ..
            }
        ),
        "{:?}",
        res.fact.value()
    );
}

/// Edge case 5, second half: a member that is another registry multisig's
/// vault is resolved and published, so the graph can show it.
#[tokio::test]
async fn edge_a_member_that_is_a_multisig_is_resolved_for_the_graph() {
    let (ma, mb) = (pda_like(10), pda_like(11));
    let vault_b = pda::squads_vault(&mb, 0).unwrap();
    let rpc = SolanaRpc::new(account_server(vec![
        (
            ma.to_string(),
            SQUADS_V4_PROGRAM.into(),
            multisig_bytes(
                [0; 32],
                2,
                0,
                &[
                    ([1; 32], permission::VOTE),
                    (pda::decode(&vault_b).unwrap(), permission::VOTE),
                ],
            ),
        ),
        (
            mb.to_string(),
            SQUADS_V4_PROGRAM.into(),
            multisig_bytes(
                [0; 32],
                3,
                86_400,
                &[
                    ([2; 32], permission::VOTE),
                    ([3; 32], permission::VOTE),
                    ([4; 32], permission::VOTE),
                ],
            ),
        ),
    ]));
    let known = KnownControllers::default();
    let registry = [
        RegistryMultisig {
            address: ma.clone(),
            vault_index: 0,
            program: celebtir_registry::MultisigProgram::SquadsV4,
        },
        RegistryMultisig {
            address: mb.clone(),
            vault_index: 0,
            program: celebtir_registry::MultisigProgram::SquadsV4,
        },
    ];
    let r = Resolver::new(&rpc, &registry, &known);
    let res = r
        .resolve(&pda::squads_vault(&ma, 0).unwrap(), origin())
        .await
        .unwrap();
    let Controller::Multisig {
        voters,
        nested_signers,
        ..
    } = res.fact.value()
    else {
        panic!("{:?}", res.fact.value());
    };
    assert_eq!(*voters, 2, "still one voter each");
    assert_eq!(nested_signers.len(), 1, "the wallet key is not nested");
    assert!(
        matches!(
            nested_signers[&vault_b].as_ref(),
            Controller::Multisig { address, threshold: 3, voters: 3, time_lock_secs: 86_400, .. } if *address == mb
        ),
        "{nested_signers:?}"
    );
    assert!(
        res.fact.evidence().iter().any(|e| e.address == mb),
        "evidence covers the nested multisig"
    );
}

/// A member that is an unknown program-derived address is published as
/// Unrecognized, never guessed.
#[tokio::test]
async fn edge_an_unknown_pda_member_is_a_pda() {
    let ma = pda_like(12);
    let stranger = pda_like(13);
    let rpc = SolanaRpc::new(account_server(vec![(
        ma.to_string(),
        SQUADS_V4_PROGRAM.into(),
        multisig_bytes(
            [0; 32],
            1,
            0,
            &[
                ([1; 32], permission::VOTE),
                (pda::decode(&stranger).unwrap(), permission::VOTE),
            ],
        ),
    )]));
    let known = KnownControllers::default();
    let r = Resolver::new(
        &rpc,
        &[RegistryMultisig {
            address: ma.clone(),
            vault_index: 0,
            program: celebtir_registry::MultisigProgram::SquadsV4,
        }],
        &known,
    );
    let res = r
        .resolve(&pda::squads_vault(&ma, 0).unwrap(), origin())
        .await
        .unwrap();
    let Controller::Multisig { nested_signers, .. } = res.fact.value() else {
        panic!("{:?}", res.fact.value());
    };
    assert!(matches!(
        nested_signers[&stranger].as_ref(),
        Controller::Pda { .. }
    ));
    assert!(!res.review_needed.is_empty(), "flagged for registry review");
}

#[tokio::test]
async fn edge_closed_core_program_makes_the_protocol_unrated() {
    let rpc = SolanaRpc::new(account_server(vec![]));
    let known = KnownControllers::default();
    let r = Resolver::new(&rpc, &[], &known);
    let closed = a("C1osedProgram111111111111111111111111111111");
    let control = upgrade_controls(&rpc, &r, std::slice::from_ref(&closed))
        .await
        .unwrap()
        .remove(0);
    assert!(
        matches!(control.upgrade_control.value(), Controller::Unavailable { reason } if reason.contains("closed")),
        "{:?}",
        control.upgrade_control.value()
    );

    let facts = FactsFile::new(BTreeMap::from([(
        "p".to_string(),
        ProtocolFacts {
            chain: Chain::Solana,
            category: None,
            last_reviewed: None,
            name: None,
            website: None,
            reviewed_by: None,
            sources: Vec::new(),
            programs: BTreeMap::from([(
                closed,
                ProgramFacts {
                    role: ProgramRole::Core,
                    loader: control.read.kind.loader(),
                    upgrade_control: control.upgrade_control,
                    code_transparency: celebtir_core::Fact::single(
                        celebtir_core::snapshot::CodeTransparency::NoPublicSource,
                        origin(),
                    ),
                },
            )]),
            admin: BTreeMap::new(),
            oracles: BTreeMap::new(),
            admin_review: None,
            oracle_review: None,
            dependencies: celebtir_core::Fact::single(
                celebtir_core::snapshot::Dependencies::Unavailable {
                    reason: "closed".into(),
                },
                origin(),
            ),
            tokens: BTreeMap::new(),
            min_slot: 1,
            max_slot: 1,
        },
    )]));
    let scores = celebtir_score::score(&facts, &celebtir_score::Methodology::shipped());
    assert_eq!(
        scores.protocols["p"].stage.stage,
        celebtir_score::Stage::Unrated
    );
}

#[tokio::test]
async fn edge_unsupported_loader_program_is_unrecognized_not_guessed() {
    let system = load("program-system");
    let rpc = SolanaRpc::new(account_server(vec![(
        system.address.to_string(),
        system.owner.clone().unwrap().to_string(),
        vec![],
    )]));
    let known = KnownControllers::default();
    let r = Resolver::new(&rpc, &[], &known);
    let c = upgrade_controls(&rpc, &r, std::slice::from_ref(&system.address))
        .await
        .unwrap()
        .remove(0);
    assert!(
        matches!(c.upgrade_control.value(), Controller::Unrecognized { owner_program: Some(o), .. } if o.as_str() == "NativeLoader1111111111111111111111111111111")
    );
}
