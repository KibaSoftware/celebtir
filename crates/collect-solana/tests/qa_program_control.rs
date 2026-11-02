mod common;
use celebtir_core::{Address, Chain, Controller, Evidence};
use celebtir_registry::{AuthorityPda, PdaSeed};
use collect_solana::{
    pda, program,
    reads::SolanaRpc,
    resolve::{KnownControllers, Resolver},
};

fn declaration() -> AuthorityPda {
    let mut d = AuthorityPda {
        key: Address::new("pending"),
        program: pda::encode([7; 32]),
        seeds: vec![PdaSeed::Literal("withdraw".into())],
        source: "https://example.org/program-source".parse().unwrap(),
    };
    d.key = pda::declared_address(&d).unwrap();
    d
}

#[tokio::test]
async fn col_8_qa_declared_pda_resolves_immutable_program_with_evidence() {
    let d = declaration();
    let rpc = SolanaRpc::new(common::client(|body| {
        let mut result = common::accounts_result(&body["params"][0], 123, program::LOADER_V2, &[]);
        for a in result["value"].as_array_mut().unwrap() {
            a["executable"] = true.into();
        }
        result
    }));
    let known = KnownControllers::default();
    let resolver = Resolver::new(&rpc, &[], &known).with_pdas([d.clone()]);
    let result = resolver
        .resolve(
            &d.key,
            Evidence::new(Chain::Solana, d.key.clone(), 1, "mint_authority"),
        )
        .await
        .unwrap();
    assert!(
        matches!(result.fact.value(), Controller::ProgramControlled { upgrade, .. } if **upgrade == Controller::Immutable)
    );
    assert!(result.fact.evidence().len() >= 2);
}

#[tokio::test]
async fn col_8_qa_wrong_pda_seeds_remain_an_unresolved_pda() {
    let mut d = declaration();
    d.seeds = vec![PdaSeed::Literal("wrong".into())];
    let rpc = SolanaRpc::new(common::account_server(vec![]));
    let known = KnownControllers::default();
    let resolver = Resolver::new(&rpc, &[], &known).with_pdas([d.clone()]);
    let result = resolver
        .resolve(
            &d.key,
            Evidence::new(Chain::Solana, d.key.clone(), 1, "mint_authority"),
        )
        .await
        .unwrap();
    assert!(matches!(result.fact.value(), Controller::Pda { .. }));
    assert!(!result.review_needed.is_empty());
}

#[test]
fn reg_3_col_8_qa_pda_seed_limits_and_encodings_are_explicit() {
    let mut d = declaration();
    d.seeds = vec![PdaSeed::Literal("x".repeat(33))];
    assert!(pda::declared_address(&d).is_none());
    d.seeds = vec![PdaSeed::Literal("x".into()); 16];
    assert!(pda::declared_address(&d).is_none());
    d.seeds = vec![PdaSeed::Address(Address::new("invalid"))];
    assert!(pda::declared_address(&d).is_none());
}

#[tokio::test]
async fn col_8_qa_declared_pda_cannot_trust_nonexecutable_program() {
    let d = declaration();
    let rpc = SolanaRpc::new(common::account_server(vec![(
        d.program.to_string(),
        program::LOADER_V2.into(),
        vec![],
    )]));
    let known = KnownControllers::default();
    let resolver = Resolver::new(&rpc, &[], &known).with_pdas([d.clone()]);
    let result = resolver
        .resolve(
            &d.key,
            Evidence::new(Chain::Solana, d.key.clone(), 1, "mint_authority"),
        )
        .await
        .unwrap();
    assert!(matches!(
        result.fact.value(),
        Controller::Unavailable { .. }
    ));
    assert!(!result.review_needed.is_empty());
}
