//! COL-11: admin accounts are fetched, their owner checked against the
//! protocol's programs, and the named field decoded with the vendored IDL.

mod common;

use celebtir_core::snapshot::AdminPower;
use celebtir_core::{Address, Controller};
use collect_solana::admin::{AdminSpec, read_admins};
use collect_solana::fixtures::load;
use collect_solana::idl::Idl;
use collect_solana::pda;
use collect_solana::reads::SolanaRpc;
use collect_solana::resolve::{KnownControllers, Resolver};
use common::*;

const MARKET: &str = "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF";
const KLEND: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const OWNER: &str = "24LjDBukaUSHgPowcF2wY1XscnhChcBUDETN2UhBZMMT";

fn a(s: &str) -> Address {
    Address::new(s)
}

fn kamino_idl() -> Idl {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/idl/kamino-lend.json");
    Idl::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn market_rpc(owner: &str, data: Vec<u8>) -> SolanaRpc<FnTransport> {
    SolanaRpc::new(account_server(vec![(MARKET.into(), owner.into(), data)]))
}

async fn admin_controller(
    rpc: &SolanaRpc<FnTransport>,
    field: &str,
    idl: &Idl,
) -> (Controller, Vec<celebtir_core::Evidence>) {
    let known = KnownControllers::default();
    let resolver = Resolver::new(rpc, &[], &known);
    let specs = [AdminSpec {
        account: a(MARKET),
        field: field.into(),
        power: AdminPower::Parameters,
        idl,
    }];
    let facts = read_admins(rpc, &resolver, &specs, &[a(KLEND)])
        .await
        .unwrap();
    let f = &facts[&format!("{MARKET}.{field}")];
    assert_eq!(f.power, AdminPower::Parameters);
    (
        f.controller.value().clone(),
        f.controller.evidence().to_vec(),
    )
}

#[test]
fn col_11_idl_reads_kaminos_lending_market_owner() {
    let data = load("kamino-lending-market-main").data();
    let idl = kamino_idl();
    assert_eq!(idl.account_type(&data), Some("LendingMarket"));
    let key = idl
        .read_pubkey(&data, "lending_market_owner")
        .unwrap()
        .unwrap();
    assert_eq!(pda::encode(key).as_str(), OWNER);
    // camelCase from the IDL matches too.
    assert_eq!(
        idl.read_pubkey(&data, "lendingMarketOwner")
            .unwrap()
            .unwrap(),
        key
    );
}

#[tokio::test]
async fn col_11_kamino_admin_key_is_read_and_resolved() {
    let rpc = market_rpc(KLEND, load("kamino-lending-market-main").data());
    let idl = kamino_idl();
    let (controller, evidence) = admin_controller(&rpc, "lending_market_owner", &idl).await;
    let key = match &controller {
        Controller::SingleKey { key } | Controller::Pda { key, .. } => key.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(key.as_str(), OWNER);
    assert_eq!(evidence[0].address.as_str(), MARKET);
    assert_eq!(evidence[0].field, "lending_market_owner");
}

#[tokio::test]
async fn col_11_wrong_owner_records_unavailable_never_a_default_key() {
    let rpc = market_rpc(
        "11111111111111111111111111111111",
        load("kamino-lending-market-main").data(),
    );
    let idl = kamino_idl();
    let (controller, evidence) = admin_controller(&rpc, "lending_market_owner", &idl).await;
    assert!(
        matches!(&controller, Controller::Unavailable { reason } if reason.contains("owned by")),
        "{controller:?}"
    );
    assert_eq!(
        evidence.len(),
        1,
        "an unavailable fact still cites the account read"
    );
}

#[tokio::test]
async fn col_11_unknown_field_records_unavailable() {
    let rpc = market_rpc(KLEND, load("kamino-lending-market-main").data());
    let idl = kamino_idl();
    let (controller, _) = admin_controller(&rpc, "no_such_field", &idl).await;
    assert!(
        matches!(&controller, Controller::Unavailable { reason } if reason.contains("no field")),
        "{controller:?}"
    );
}

#[tokio::test]
async fn col_11_wrong_account_type_records_unavailable() {
    let mut data = load("kamino-lending-market-main").data();
    data[0] ^= 0xff;
    let rpc = market_rpc(KLEND, data);
    let idl = kamino_idl();
    let (controller, _) = admin_controller(&rpc, "lending_market_owner", &idl).await;
    assert!(
        matches!(&controller, Controller::Unavailable { reason } if reason.contains("discriminator")),
        "{controller:?}"
    );
}

#[tokio::test]
async fn col_11_non_key_field_records_unavailable() {
    let rpc = market_rpc(KLEND, load("kamino-lending-market-main").data());
    let idl = kamino_idl();
    let (controller, _) = admin_controller(&rpc, "referral_fee_bps", &idl).await;
    assert!(
        matches!(&controller, Controller::Unavailable { reason } if reason.contains("not a public key")),
        "{controller:?}"
    );
}

#[tokio::test]
async fn col_11_zeroed_admin_field_means_renounced() {
    let mut data = load("kamino-lending-market-main").data();
    data[24..56].fill(0);
    let rpc = market_rpc(KLEND, data);
    let idl = kamino_idl();
    let (controller, _) = admin_controller(&rpc, "lending_market_owner", &idl).await;
    assert_eq!(controller, Controller::Immutable);
}

#[tokio::test]
async fn col_11_missing_admin_account_records_unavailable() {
    let rpc = SolanaRpc::new(account_server(vec![]));
    let idl = kamino_idl();
    let (controller, _) = admin_controller(&rpc, "lending_market_owner", &idl).await;
    assert!(
        matches!(&controller, Controller::Unavailable { reason } if reason.contains("does not exist")),
        "{controller:?}"
    );
}

/// Synthetic 0.30-format IDL: explicit discriminator, Vec/Option/String before
/// the key, nested struct path, Option<Pubkey>.
#[test]
fn col_11_idl_sizes_variable_fields_and_follows_nested_paths() {
    let idl = Idl::from_json(r#"{
      "accounts": [{"name": "State", "discriminator": [1,2,3,4,5,6,7,8]}],
      "types": [
        {"name": "State", "type": {"kind": "struct", "fields": [
          {"name": "label", "type": "string"},
          {"name": "fees", "type": {"vec": "u16"}},
          {"name": "maybe", "type": {"option": "u64"}},
          {"name": "mode", "type": {"defined": {"name": "Mode"}}},
          {"name": "config", "type": {"defined": {"name": "Config"}}},
          {"name": "guardian", "type": {"option": "pubkey"}}
        ]}},
        {"name": "Mode", "type": {"kind": "enum", "variants": [{"name": "Off"}, {"name": "On", "fields": ["u32"]}]}},
        {"name": "Config", "type": {"kind": "struct", "fields": [
          {"name": "version", "type": "u8"},
          {"name": "admin", "type": "pubkey"}
        ]}}
      ]
    }"#).unwrap();
    let mut d = vec![1, 2, 3, 4, 5, 6, 7, 8];
    d.extend(3u32.to_le_bytes());
    d.extend(b"abc");
    d.extend(2u32.to_le_bytes());
    d.extend([0u8; 4]);
    d.push(1);
    d.extend(9u64.to_le_bytes());
    d.push(1);
    d.extend(7u32.to_le_bytes());
    d.push(1);
    d.extend([42u8; 32]);
    d.push(0);
    assert_eq!(
        idl.read_pubkey(&d, "config.admin").unwrap(),
        Some([42u8; 32])
    );
    assert_eq!(idl.read_pubkey(&d, "guardian").unwrap(), None);
    assert!(
        idl.read_pubkey(&d[..40], "config.admin").is_err(),
        "truncated data is an error"
    );
}
