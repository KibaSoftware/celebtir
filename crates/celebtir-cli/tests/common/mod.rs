//! `fixtures/facts/sample-facts.json`: three hand-built protocols covering
//! Stage 0, Stage 1 and Unrated, plus one token. Regenerate with
//! `CELEBTIR_UPDATE_FIXTURES=1 cargo test -p celebtir`.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use celebtir_core::snapshot::{
    AdminFact, AdminPower, CodeTransparency, Dependencies, FactsFile, OracleFact, OracleRole,
    OracleSource, ProgramFacts, ProgramRole, ProtocolFacts, TokenExtensions, TokenFacts,
    TokenProgram, TokenProvenance, TokenRole,
};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact, MultisigKind};

pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn sample_path() -> PathBuf {
    repo().join("fixtures/facts/sample-facts.json")
}

fn a(s: &str) -> Address {
    Address::new(s)
}

fn ev(address: &str, slot: u64, field: &str) -> Evidence {
    Evidence::new(Chain::Solana, a(address), slot, field)
}

fn multisig(threshold: u16, voters: u16, time_lock_secs: u32) -> Controller {
    Controller::Multisig {
        kind: MultisigKind::SquadsV4,
        address: a("6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM"),
        threshold,
        voters,
        time_lock_secs,
        config_authority: None,
        signers: vec![],
        nested_signers: Default::default(),
    }
}

fn program(
    id: &str,
    data: &str,
    role: ProgramRole,
    c: Controller,
    verified: bool,
) -> (Address, ProgramFacts) {
    let transparency = if verified {
        CodeTransparency::Verified {
            repo: "https://github.com/example/program".into(),
            commit: "0a1b2c3".into(),
            on_chain_hash: "h".into(),
            executable_hash: "h".into(),
            warning: None,
        }
    } else {
        CodeTransparency::PublicSourceUnverified {
            repo: "https://github.com/example/program".into(),
            commit: None,
            on_chain_hash: None,
        }
    };
    (
        a(id),
        ProgramFacts {
            role,
            loader: None,
            upgrade_control: Fact::single(c, ev(data, 382_114_880, "upgrade_authority")),
            code_transparency: Fact::single(transparency, ev(id, 382_114_880, "verified_build")),
        },
    )
}

fn no_dependencies(id: &str) -> Fact<Dependencies> {
    Fact::single(
        Dependencies::Sampled {
            callees: BTreeMap::new(),
            stale: false,
            declared: false,
        },
        ev(id, 382_114_900, "transaction_sample"),
    )
}

pub fn sample() -> FactsFile {
    const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let lend = ProtocolFacts {
        chain: Chain::Solana,
        category: None,
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: BTreeMap::from([
            program(
                "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
                "9FtmuBFcCqRHGaqKSAYHwLjcHTvn7WZ1ZnkWgytXu3EA",
                ProgramRole::Core,
                multisig(2, 4, 0),
                true,
            ),
            program(
                "FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr",
                "3a8WZzXxBu5mzkCMJ9pJrUcDMqtvcxJ6dS27z7DFTCgb",
                ProgramRole::Periphery,
                Controller::SingleKey {
                    key: a("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
                },
                false,
            ),
        ]),
        admin: BTreeMap::from([(
            "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF.lending_market_owner".to_string(),
            AdminFact {
                account: a("7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF"),
                field: "lending_market_owner".into(),
                power: AdminPower::Parameters,
                controller: Fact::single(
                    multisig(2, 4, 0),
                    ev(
                        "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
                        382_114_890,
                        "lending_market_owner",
                    ),
                ),
            },
        )]),
        oracles: BTreeMap::from([(
            a("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"),
            OracleFact {
                market: "SOL reserve".into(),
                role: OracleRole::Primary,
                bounds_primary: false,
                guarded_by: vec![],
                provider: None,
                source: Fact::single(
                    OracleSource::Network {
                        name: "pyth".into(),
                    },
                    ev(
                        "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE",
                        382_114_890,
                        "owner",
                    ),
                ),
                underlying: None,
                underlying_markets: Default::default(),
            },
        )]),
        admin_review: None,
        oracle_review: None,
        dependencies: no_dependencies("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
        tokens: BTreeMap::from([(a(USDC), vec![TokenRole::Collateral, TokenRole::Borrowable])]),
        min_slot: 382_114_880,
        max_slot: 382_114_900,
    };

    let dex = ProtocolFacts {
        chain: Chain::Solana,
        category: None,
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: BTreeMap::from([program(
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
            "8YmFNBJ2CR5JLd3P1bzJxZRFcqGBy2DmBxVVcVPmUfAU",
            ProgramRole::Core,
            multisig(5, 8, 5 * 86_400),
            true,
        )]),
        admin: BTreeMap::new(),
        oracles: BTreeMap::new(),
        admin_review: None,
        oracle_review: None,
        dependencies: no_dependencies("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
        tokens: BTreeMap::from([(a(USDC), vec![TokenRole::Lp])]),
        min_slot: 382_114_880,
        max_slot: 382_114_900,
    };

    let perps = ProtocolFacts {
        chain: Chain::Solana,
        category: None,
        last_reviewed: None,
        name: None,
        website: None,
        reviewed_by: None,
        sources: Vec::new(),
        programs: BTreeMap::from([program(
            "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH",
            "BHCNWD9tg2WrQq33SbF1EbMjNhVRjGb8Z1M5qxvXqLXz",
            ProgramRole::Core,
            Controller::Unavailable {
                reason: "ProgramData read failed: RPC timeout".into(),
            },
            true,
        )]),
        admin: BTreeMap::new(),
        oracles: BTreeMap::new(),
        admin_review: None,
        oracle_review: None,
        dependencies: no_dependencies("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH"),
        tokens: BTreeMap::new(),
        min_slot: 382_114_880,
        max_slot: 382_114_900,
    };

    let mut facts = FactsFile::new(BTreeMap::from([
        ("sample-lend".to_string(), lend),
        ("sample-dex".to_string(), dex),
        ("sample-perps".to_string(), perps),
    ]));
    facts.tokens.insert(
        a(USDC),
        TokenFacts {
            symbol: "USDC".into(),
            program: TokenProgram::SplToken,
            supply: 1_000_000,
            decimals: 6,
            mint_authority: Fact::single(
                Controller::Unrecognized {
                    key: a("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG"),
                    owner_program: None,
                },
                ev(USDC, 382_114_895, "mint_authority"),
            ),
            freeze_authority: Fact::single(
                Controller::Unrecognized {
                    key: a("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"),
                    owner_program: None,
                },
                ev(USDC, 382_114_895, "freeze_authority"),
            ),
            extensions: Fact::single(
                TokenExtensions::default(),
                ev(USDC, 382_114_895, "extensions"),
            ),
            provenance: Fact::single(TokenProvenance::Native, ev(USDC, 382_114_895, "mint")),
            identity: None,
        },
    );
    facts
}
