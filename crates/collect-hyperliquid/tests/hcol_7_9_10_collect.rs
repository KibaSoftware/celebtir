//! HCOL-7 (admin powers), HCOL-9 (oracle classification), HCOL-10
//! (declared dependencies). Synthetic fixtures (see `common`).

mod common;

use std::collections::BTreeMap;

use celebtir_core::snapshot::{AdminPower, Dependencies, OracleSource};
use celebtir_core::{Address, Controller};
use celebtir_registry::ProtocolEntry;
use celebtir_score::{Factor, Methodology, score_protocol};
use collect_hyperliquid::collect::{EvmOracles, collect_protocol};
use collect_hyperliquid::evm::{HyperEvm, default_config};
use collect_hyperliquid::resolve::Resolver;
use collect_solana::rpc::RpcClient;
use common::*;

const PYTH: &str = "0x52908400098527886E0F7030069857D2E4169EE7";
const DEP: &str = "0x8617E340B3D01FA5F11F306F4090FD50E238070D";

fn entry() -> ProtocolEntry {
    serde_norway::from_str(&format!(
        r#"
id: evm-lend
name: EVM Lend
chain: hyper_evm
category: lending
website: https://example.org
programs:
  - id: "{PROXY}"
    name: Pool
    role: core
    proxy: none
    source: https://example.org
evm_admins:
  - contract: "{PROXY}"
    power: pause
    source: https://example.org
  - contract: "{PROXY}"
    power: parameters
    role: RISK_ADMIN
    holder: "{KEY}"
    source: https://example.org
oracles:
  - account: "{PYTH}"
    market: HYPE
    role: primary
    source: https://example.org
dependencies:
  - address: "{DEP}"
    name: Callee
    proxy: uups
    source: https://example.org
review:
  reviewed_by: test
  last_reviewed: 2026-10-21
"#
    ))
    .unwrap()
}

fn oracles() -> EvmOracles {
    EvmOracles::from_yaml(&format!(
        "evm:\n  - address: \"{PYTH}\"\n    class: network\n    name: pyth\n    source: https://docs.pyth.network/\n"
    ))
    .unwrap()
}

fn chain(owner_reverts: bool, holder_has_role: bool) -> Chain {
    let role = collect_hyperliquid::evm::hex(&collect_hyperliquid::resolve::keccak("RISK_ADMIN"));
    Chain::default()
        .sig(
            PROXY,
            "owner()",
            "",
            (!owner_reverts).then(|| word_of_address(ADMIN)).as_deref(),
        )
        .code(ADMIN, "0x")
        .sig(
            PROXY,
            "hasRole(bytes32,address)",
            &format!("{}{}", &role[2..], pad(KEY)),
            Some(&uint(u64::from(holder_has_role))),
        )
        .code(KEY, "0x")
        .code(PYTH, "0x6080")
        .sig(DEP, "owner()", "", Some(&word_of_address(KEY)))
}

async fn collect(c: Chain) -> celebtir_core::snapshot::ProtocolFacts {
    let evm = HyperEvm::connect(RpcClient::new(c.fixture().replay(), default_config()))
        .await
        .unwrap();
    let k = known(&[]);
    let r = Resolver {
        evm: &evm,
        known: &k,
    };
    collect_protocol(&r, &oracles(), &entry(), &BTreeMap::new()).await
}

#[tokio::test(start_paused = true)]
async fn hcol_7_owner_and_role_holders_are_resolved() {
    let p = collect(chain(false, true)).await;
    let pause = &p.admin[&format!("{PROXY}.owner()")];
    assert_eq!(pause.power, AdminPower::Pause);
    assert_eq!(
        pause.controller.value(),
        &Controller::SingleKey {
            key: Address::new(ADMIN)
        }
    );
    let risk = &p.admin[&format!("{PROXY}.RISK_ADMIN")];
    assert_eq!(
        risk.controller.value(),
        &Controller::SingleKey {
            key: Address::new(KEY)
        }
    );
    assert_eq!((p.min_slot, p.max_slot), (BLOCK, BLOCK));
}

#[tokio::test(start_paused = true)]
async fn hcol_7_reverting_owner_records_unavailable() {
    let p = collect(chain(true, false)).await;
    assert!(matches!(
        p.admin[&format!("{PROXY}.owner()")].controller.value(),
        Controller::Unavailable { .. }
    ));
    assert!(matches!(
        p.admin[&format!("{PROXY}.RISK_ADMIN")].controller.value(),
        Controller::Unavailable { reason } if reason.contains("does not hold")
    ));
}

#[tokio::test(start_paused = true)]
async fn hcol_9_pyth_address_classifies_as_pyth() {
    let p = collect(chain(false, true)).await;
    assert_eq!(
        p.oracles[&Address::new(PYTH)].source.value(),
        &OracleSource::Network {
            name: "pyth".into()
        }
    );
    let other = oracles().classify(&Address::new(KEY));
    assert!(matches!(other, OracleSource::Other { .. }));
}

#[tokio::test(start_paused = true)]
async fn hcol_10_dependencies_are_declared_and_named_in_the_reason() {
    let p = collect(chain(false, true)).await;
    match p.dependencies.value() {
        Dependencies::Sampled {
            callees, declared, ..
        } => {
            assert!(declared);
            assert_eq!(
                callees[&Address::new(DEP)].upgrade_control.value(),
                &Controller::SingleKey {
                    key: Address::new(KEY)
                }
            );
        }
        other => panic!("{other:?}"),
    }
    let s = score_protocol(&p, &Methodology::shipped());
    let reason = &s.factors[&Factor::Dependencies].reason;
    assert!(reason.contains("declared in the registry"), "{reason}");
}
