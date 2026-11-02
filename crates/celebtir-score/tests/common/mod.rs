//! Builders for hand-written facts.
#![allow(dead_code)]

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    AdminFact, AdminPower, CodeTransparency, Dependencies, DependencyFact, OracleFact, OracleRole,
    OracleSource,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact, MultisigKind};

pub const DAY: u32 = 86_400;

pub fn addr(s: &str) -> Address {
    Address::new(s)
}

pub fn ev(field: &str) -> Evidence {
    Evidence::new(
        Chain::Solana,
        addr("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
        382_114_880,
        field,
    )
}

pub fn fact<T>(value: T) -> Fact<T> {
    Fact::single(value, ev("test"))
}

pub fn single_key() -> Controller {
    Controller::SingleKey {
        key: addr("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
    }
}

pub fn multisig(threshold: u16, voters: u16, time_lock_secs: u32) -> Controller {
    Controller::Multisig {
        kind: MultisigKind::SquadsV4,
        address: addr("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"),
        threshold,
        voters,
        time_lock_secs,
        config_authority: None,
        signers: vec![],
        nested_signers: Default::default(),
    }
}

pub fn with_config_authority(c: Controller, authority: Controller) -> Controller {
    match c {
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            ..
        } => Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority: Some(Box::new(authority)),
            signers: vec![],
            nested_signers: Default::default(),
        },
        other => other,
    }
}

pub fn admin(entries: Vec<(AdminPower, Controller)>) -> BTreeMap<String, AdminFact> {
    entries
        .into_iter()
        .enumerate()
        .map(|(i, (power, controller))| {
            (
                format!("acct{i}.owner"),
                AdminFact {
                    account: addr(&format!("acct{i}")),
                    field: "owner".into(),
                    power,
                    controller: fact(controller),
                },
            )
        })
        .collect()
}

pub fn network(name: &str) -> OracleSource {
    OracleSource::Network { name: name.into() }
}

/// `(market, role, source)` triples.
pub fn oracles(entries: Vec<(&str, OracleRole, OracleSource)>) -> BTreeMap<Address, OracleFact> {
    entries
        .into_iter()
        .enumerate()
        .map(|(i, (market, role, source))| {
            (
                addr(&format!("oracle{i}")),
                OracleFact {
                    market: market.into(),
                    role,
                    bounds_primary: false,
                    guarded_by: vec![],
                    provider: None,
                    source: fact(source),
                    underlying: None,
                    underlying_markets: Default::default(),
                },
            )
        })
        .collect()
}

pub fn dependencies(callees: Vec<Controller>) -> Fact<Dependencies> {
    fact(Dependencies::Sampled {
        callees: callees
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                (
                    addr(&format!("callee{i}")),
                    DependencyFact {
                        upgrade_control: fact(c),
                    },
                )
            })
            .collect(),
        stale: false,
        declared: false,
    })
}

pub fn verified() -> Fact<CodeTransparency> {
    fact(CodeTransparency::Verified {
        repo: "https://github.com/Kamino-Finance/klend".into(),
        commit: "abc123".into(),
        on_chain_hash: "h".into(),
        executable_hash: "h".into(),
        warning: None,
    })
}

use celebtir_core::snapshot::{ProgramFacts, ProgramRole, ProtocolFacts};

/// A protocol with the given programs, no admin keys, no oracles and no
/// dependencies.
pub fn protocol(programs: Vec<(&str, ProgramRole, Controller)>) -> ProtocolFacts {
    ProtocolFacts {
        chain: Chain::Solana,
        category: None,
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: programs
            .into_iter()
            .map(|(a, role, c)| {
                (
                    addr(a),
                    ProgramFacts {
                        role,
                        loader: None,
                        upgrade_control: fact(c),
                        code_transparency: verified(),
                    },
                )
            })
            .collect(),
        admin: BTreeMap::new(),
        oracles: BTreeMap::new(),
        admin_review: None,
        oracle_review: None,
        dependencies: dependencies(vec![]),
        tokens: BTreeMap::new(),
        min_slot: 382_114_880,
        max_slot: 382_114_880,
    }
}

use celebtir_core::snapshot::{
    TokenExtensions, TokenFacts, TokenProgram, TokenProvenance, TransferHook,
};

/// A classic SPL mint with no authorities, native to Solana.
pub fn token() -> TokenFacts {
    TokenFacts {
        symbol: "TKN".into(),
        program: TokenProgram::SplToken,
        supply: 1_000_000,
        decimals: 6,
        mint_authority: fact(Controller::Immutable),
        freeze_authority: fact(Controller::Immutable),
        extensions: fact(TokenExtensions::default()),
        provenance: fact(TokenProvenance::Native),
        identity: None,
    }
}

pub fn hook(authority: Controller, program_controller: Controller) -> TransferHook {
    TransferHook {
        program: Some(addr("HooK1111111111111111111111111111111111111111")),
        authority,
        program_controller: Some(program_controller),
    }
}
