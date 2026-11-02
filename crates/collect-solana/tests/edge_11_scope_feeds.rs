//! Edge case 11 / slice 3.17: a protocol-run aggregator (Kamino Scope) is
//! rated as protocol-run, and the feeds it combines for a market are listed
//! where decodable. Fixtures: Kamino's SOL reserve, the Scope price account
//! it reads, and Scope's mappings, captured from mainnet.

mod common;

use celebtir_core::Address;
use celebtir_core::snapshot::{UnderlyingFeed, UnderlyingFeeds};
use collect_solana::fixtures::load;
use collect_solana::idl::Idl;
use collect_solana::reads::SolanaRpc;
use collect_solana::scope::{ScopeChainSpec, read_underlying};
use common::*;

const RESERVE: &str = "d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q";
const PRICES: &str = "3t4JZcueEzTbVP6kLxXrL3VpWx45jDer4eqysweBchNH";
const MAPPINGS: &str = "4zh6bmb77qX2CL7t5AJYCqa6YqFafbz3QJNeFvZjLowg";
const SCOPE: &str = "HFn8GnPADiny6XqUoWE8uRPPxb29ikn4yTuPa9MF2fWJ";
const FEED_FIELD: &str = "config.token_info.scope_configuration.price_feed";
const CHAIN_FIELD: &str = "config.token_info.scope_configuration.price_chain";

fn a(s: &str) -> Address {
    Address::new(s)
}

fn idl(name: &str) -> Idl {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/idl")
        .join(name);
    Idl::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn rpc() -> SolanaRpc<FnTransport> {
    let acct = |name: &str| {
        let f = load(name);
        (
            f.address.to_string(),
            f.owner.clone().unwrap().to_string(),
            f.data(),
        )
    };
    SolanaRpc::new(account_server(vec![
        acct("kamino-reserve-sol"),
        acct("scope-oracle-prices-kamino"),
        acct("scope-oracle-mappings-kamino"),
    ]))
}

#[test]
fn col_11_idl_reads_u16_arrays_indexed_entries_and_enum_names() {
    let kamino = idl("kamino-lend.json");
    let scope = idl("scope.json");
    let reserve = load("kamino-reserve-sol").data();
    assert_eq!(
        kamino.read_u16s(&reserve, CHAIN_FIELD).unwrap(),
        vec![3, u16::MAX, u16::MAX, u16::MAX]
    );
    let mappings = load("scope-oracle-mappings-kamino").data();
    assert_eq!(scope.read_u8(&mappings, "price_types[3]").unwrap(), 28);
    assert_eq!(scope.enum_variant("OracleType", 28), Some("MostRecentOf"));
    assert_eq!(
        scope
            .read_pubkey(&mappings, "price_info_accounts[3]")
            .unwrap()
            .map(|b| bs58::encode(b).into_string()),
        Some(SCOPE.to_string())
    );
    assert_eq!(scope.read_bytes(&mappings, "generic[3]").unwrap().len(), 20);
}

#[tokio::test]
async fn edge_11_scope_chain_lists_the_feeds_behind_most_recent_of() {
    let (kamino, scope) = (idl("kamino-lend.json"), idl("scope.json"));
    let spec = ScopeChainSpec {
        chain_account: a(RESERVE),
        chain_idl: &kamino,
        feed_field: FEED_FIELD,
        chain_field: CHAIN_FIELD,
        scope_idl: &scope,
    };
    let out = read_underlying(&rpc(), &a(PRICES), &spec).await.unwrap();
    assert!(out.review.is_empty(), "{:?}", out.review);
    let UnderlyingFeeds::Listed { feeds } = out.fact.value() else {
        panic!("{:?}", out.fact.value());
    };
    // Entry 3 is MostRecentOf(entries 1 and 2): a Chainlink stream and a
    // Pyth Lazer feed. The aggregator entry itself is not a leaf.
    assert_eq!(
        feeds,
        &vec![
            UnderlyingFeed {
                source_type: "Chainlink".into(),
                address: a("14HefJnxgiYQ6qiGL5b6GVZVUSAFpmehRdBJRaZ6GKx"),
            },
            UnderlyingFeed {
                source_type: "PythLazer".into(),
                address: a(SCOPE),
            },
        ]
    );
    let cited: Vec<&Address> = out.fact.evidence().iter().map(|e| &e.address).collect();
    for addr in [RESERVE, PRICES, MAPPINGS] {
        assert!(cited.contains(&&a(addr)), "{addr} is not cited: {cited:?}");
    }
}

#[tokio::test]
async fn edge_11_a_reserve_that_reads_another_feed_is_flagged_for_review() {
    let (kamino, scope) = (idl("kamino-lend.json"), idl("scope.json"));
    let spec = ScopeChainSpec {
        chain_account: a(RESERVE),
        chain_idl: &kamino,
        feed_field: FEED_FIELD,
        chain_field: CHAIN_FIELD,
        scope_idl: &scope,
    };
    // The registry still lists Kamino's old Scope price account.
    let old = a("3NJYftD5sjVfxSnUdZ1wVML8f3aC6mp1CXCL6L7TnU8C");
    let out = read_underlying(&rpc(), &old, &spec).await.unwrap();
    let UnderlyingFeeds::Unavailable { reason } = out.fact.value() else {
        panic!("{:?}", out.fact.value());
    };
    assert!(reason.contains(PRICES), "{reason}");
    assert_eq!(out.review.len(), 1, "{:?}", out.review);
}
