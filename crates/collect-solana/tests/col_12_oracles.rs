//! COL-12: oracle accounts are classified by their owner program with
//! `oracles.yaml`; unknown owners are "other".

mod common;

use celebtir_core::Address;
use celebtir_core::snapshot::{OracleRole, OracleSource};
use collect_solana::fixtures::load;
use collect_solana::oracle::{OracleClassifier, OracleSpec, read_oracles};
use collect_solana::reads::SolanaRpc;
use common::*;

const YAML: &str = r#"
programs:
  - program: rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ
    class: network
    name: pyth
    source: https://docs.pyth.network/price-feeds/contract-addresses/solana
  - program: FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH
    class: network
    name: pyth
    source: https://docs.pyth.network/price-feeds/contract-addresses/solana
  - program: HFn8GnPADiny6XqUoWE8uRPPxb29ikn4yTuPa9MF2fWJ
    class: protocol-run
    name: scope
    source: https://github.com/Kamino-Finance/scope
"#;

fn spec(fixture: &str, market: &str, role: OracleRole) -> OracleSpec {
    OracleSpec {
        account: load(fixture).address,
        market: market.into(),
        role,
        bounds_primary: false,
        verified_by: None,
        guarded_by: vec![],
        provider: None,
    }
}

fn rpc() -> SolanaRpc<FnTransport> {
    let accounts = [
        "oracle-pyth-receiver-sol",
        "oracle-pyth-legacy-sol",
        "oracle-scope-kamino",
    ]
    .iter()
    .map(|n| {
        let f = load(n);
        (
            f.address.to_string(),
            f.owner.clone().unwrap().to_string(),
            f.data(),
        )
    })
    .collect();
    SolanaRpc::new(account_server(accounts))
}

#[tokio::test]
async fn col_12_pyth_owned_accounts_classify_as_pyth() {
    let classifier = OracleClassifier::from_yaml(YAML).unwrap();
    let specs = [
        spec("oracle-pyth-receiver-sol", "SOL", OracleRole::Primary),
        spec("oracle-pyth-legacy-sol", "SOL", OracleRole::Fallback),
    ];
    let facts = read_oracles(&rpc(), &specs, &classifier).await.unwrap();
    for s in &specs {
        assert_eq!(
            facts[&s.account].source.value(),
            &OracleSource::Network {
                name: "pyth".into()
            }
        );
    }
    assert_eq!(facts[&specs[1].account].role, OracleRole::Fallback);
}

#[tokio::test]
async fn col_12_protocol_run_aggregator_classifies_as_protocol_run() {
    let classifier = OracleClassifier::from_yaml(YAML).unwrap();
    let specs = [spec("oracle-scope-kamino", "SOL", OracleRole::Primary)];
    let facts = read_oracles(&rpc(), &specs, &classifier).await.unwrap();
    assert_eq!(
        facts[&specs[0].account].source.value(),
        &OracleSource::ProtocolRun
    );
}

#[tokio::test]
async fn col_12_unknown_owner_is_other() {
    let classifier = OracleClassifier::default();
    let specs = [spec("oracle-pyth-receiver-sol", "SOL", OracleRole::Primary)];
    let facts = read_oracles(&rpc(), &specs, &classifier).await.unwrap();
    assert_eq!(
        facts[&specs[0].account].source.value(),
        &OracleSource::Other {
            owner_program: Address::new("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ")
        }
    );
}

#[tokio::test]
async fn col_12_missing_oracle_account_is_unavailable_and_reads_are_owner_only() {
    let classifier = OracleClassifier::from_yaml(YAML).unwrap();
    let specs = [
        spec("oracle-pyth-receiver-sol", "SOL", OracleRole::Primary),
        OracleSpec {
            account: Address::new("Missing1111111111111111111111111111111111111"),
            market: "BTC".into(),
            role: OracleRole::Primary,
            bounds_primary: false,
            verified_by: None,
            guarded_by: vec![],
            provider: None,
        },
    ];
    let rpc = rpc();
    let facts = read_oracles(&rpc, &specs, &classifier).await.unwrap();
    assert!(matches!(
        facts[&specs[1].account].source.value(),
        OracleSource::Unavailable { .. }
    ));
    let bodies = rpc.client().transport().bodies();
    assert_eq!(bodies.len(), 1, "one batched read");
    assert_eq!(bodies[0]["params"][1]["dataSlice"]["length"], 0);
    assert_eq!(facts[&specs[0].account].source.evidence()[0].field, "owner");
}

#[test]
fn col_12_oracles_yaml_rejects_unknown_fields_and_classes() {
    assert!(
        OracleClassifier::from_yaml(
            "programs:\n  - program: x\n    class: magic\n    name: x\n    source: y\n"
        )
        .is_err()
    );
    assert!(OracleClassifier::from_yaml("programs:\n  - program: x\n    class: network\n    name: x\n    source: y\n    extra: 1\n").is_err());
}
