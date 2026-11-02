//! COL-15: one protocol's failure never stops the run; the run lists every
//! failure with its cause. Runs the whole collector against a mock mainnet
//! built from the captured fixtures.

use std::collections::BTreeMap;
use std::path::PathBuf;

use celebtir_core::snapshot::{CodeTransparency, Dependencies, OracleSource, UnderlyingFeeds};
use celebtir_core::{Address, Controller};
use celebtir_registry::{ProtocolEntry, TokenEntry, TokensFile};
use collect_solana::collector::{CollectInputs, collect};
use collect_solana::deps::TxCache;
use collect_solana::fixtures::{accounts_dir, load};
use collect_solana::idl::Idl;
use collect_solana::oracle::OracleClassifier;
use collect_solana::reads::SolanaRpc;
use collect_solana::resolve::KnownControllers;
use collect_solana::rpc::{RpcClient, RpcConfig, Transport, TransportError};
use collect_solana::verify::HttpGet;
use serde_json::{Value, json};

const BROKEN: &str = "Broken1111111111111111111111111111111111111";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `registry/programs.yaml`: source facts per program ID (F1).
fn programs() -> BTreeMap<String, celebtir_registry::ProgramSource> {
    let text = std::fs::read_to_string(repo().join("registry/programs.yaml")).unwrap();
    let file: celebtir_registry::ProgramsFile = serde_norway::from_str(&text).unwrap();
    file.programs
        .into_iter()
        .map(|p| (p.id.to_string(), p))
        .collect()
}

/// Every captured account, plus Kamino's transactions; `Broken…` answers 400.
struct Mainnet {
    accounts: BTreeMap<String, collect_solana::fixtures::AccountFixture>,
}

impl Mainnet {
    fn new() -> Self {
        let mut accounts = BTreeMap::new();
        for e in std::fs::read_dir(accounts_dir()).unwrap().flatten() {
            let name = e
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".json")
                .to_string();
            let f = load(&name);
            accounts.insert(f.address.to_string(), f);
        }
        Self { accounts }
    }
}

fn tx_file(name: &str) -> Value {
    let p = repo().join("fixtures/tx/kamino-lend").join(name);
    match std::fs::read_to_string(p) {
        Ok(t) => serde_json::from_str::<Value>(&t).unwrap()["result"].clone(),
        Err(_) => Value::Null,
    }
}

impl Transport for Mainnet {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        use base64::Engine;
        let params = &body["params"];
        if params.to_string().contains(BROKEN) {
            return Err(TransportError::Status(400));
        }
        let result = match body["method"].as_str().unwrap() {
            "getMultipleAccounts" => {
                let slice = &params[1]["dataSlice"];
                let values: Vec<Value> = params[0]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| match self.accounts.get(a.as_str().unwrap()) {
                        Some(f) if f.owner.is_some() => {
                            let mut d = f.data();
                            if let (Some(o), Some(l)) = (slice["offset"].as_u64(), slice["length"].as_u64()) {
                                d = d.into_iter().skip(o as usize).take(l as usize).collect();
                            }
                            json!({"data": [base64::engine::general_purpose::STANDARD.encode(d), "base64"],
                                   "owner": f.owner, "lamports": f.lamports, "executable": f.executable})
                        }
                        _ => Value::Null,
                    })
                    .collect();
                json!({"context": {"slot": 458_020_000u64}, "value": values})
            }
            "getSignaturesForAddress"
                if params[0] == "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD" =>
            {
                tx_file("signatures.json")
            }
            "getSignaturesForAddress" => json!([]),
            "getTransaction" => tx_file(&format!("{}.json", params[0].as_str().unwrap())),
            m => panic!("unexpected {m}"),
        };
        Ok(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
    }
}

struct OtterSec;

impl HttpGet for OtterSec {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        let id = url.rsplit('/').next().unwrap();
        let p = repo()
            .join("fixtures/http/ottersec")
            .join(format!("{id}.json"));
        let t = std::fs::read_to_string(p).map_err(|_| TransportError::Status(404))?;
        serde_json::from_str(&t).map_err(|e| TransportError::Decode(e.to_string()))
    }
}

const KAMINO: &str = r#"
id: kamino-lend
name: Kamino Lend
chain: solana
category: lending
website: https://kamino.finance
programs:
  - {id: KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD, name: Lending, role: core, source: https://github.com/Kamino-Finance/klend}
multisigs:
  - {address: 6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM, program: squads-v4, vault_index: 0, source: https://example.com}
admin_accounts:
  - {account: 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF, idl: idl/kamino-lend.json, field: lending_market_owner, power: parameters, source: https://example.com}
oracles:
  - {account: 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE, market: SOL, role: primary, source: https://example.com}
  - {account: 3NJYftD5sjVfxSnUdZ1wVML8f3aC6mp1CXCL6L7TnU8C, market: SOL, role: fallback, source: https://example.com}
  - {account: 3t4JZcueEzTbVP6kLxXrL3VpWx45jDer4eqysweBchNH, market: SOL (Scope), role: fallback, source: https://example.com, scope_chain: {account: d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q, idl: idl/kamino-lend.json, feed_field: config.token_info.scope_configuration.price_feed, chain_field: config.token_info.scope_configuration.price_chain}}
tokens:
  - {mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v, roles: [collateral]}
review: {reviewed_by: test, last_reviewed: 2026-10-17}
"#;

fn protocol(yaml: &str) -> ProtocolEntry {
    serde_norway::from_str(yaml).unwrap()
}

fn entries() -> Vec<ProtocolEntry> {
    let token = KAMINO.replace("id: kamino-lend", "id: spl-token").replace(
        "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    );
    let token: String = token
        .lines()
        .filter(|l| {
            !l.contains("admin")
                && !l.contains("lending_market_owner")
                && !l.contains("multisig")
                && !l.contains("6hhB")
        })
        .map(|l| format!("{l}\n"))
        .collect();
    let broken = KAMINO
        .replace("id: kamino-lend", "id: broken")
        .replace("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD", BROKEN);
    vec![protocol(&broken), protocol(KAMINO), protocol(&token)]
}

fn tokens() -> Vec<TokenEntry> {
    let f: TokensFile = serde_norway::from_str(
        r#"
tokens:
  - mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    symbol: USDC
    type: stablecoin
    source: https://example.com
    issuer: {name: Circle, source: https://example.com}
    provenance: {kind: native, source: https://example.com}
"#,
    )
    .unwrap();
    f.tokens
}

fn idls() -> BTreeMap<String, Idl> {
    ["kamino-lend.json", "scope.json"]
        .into_iter()
        .map(|name| {
            let text = std::fs::read_to_string(repo().join("fixtures/idl").join(name)).unwrap();
            (format!("idl/{name}"), Idl::from_json(&text).unwrap())
        })
        .collect()
}

fn classifier() -> OracleClassifier {
    OracleClassifier::from_yaml(
        "programs:\n  - {program: rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ, class: network, name: pyth, source: x}\n  - {program: HFn8GnPADiny6XqUoWE8uRPPxb29ikn4yTuPa9MF2fWJ, class: protocol-run, name: scope, source: x, idl: idl/scope.json}\n",
    )
    .unwrap()
}

async fn run(budget: u64) -> (collect_solana::collector::CollectOutput, SolanaRpc<Mainnet>) {
    let rpc = SolanaRpc::new(RpcClient::new(
        Mainnet::new(),
        RpcConfig {
            requests_per_sec: 10_000,
            daily_budget: budget,
            ..RpcConfig::default()
        },
    ));
    let (protocols, tokens, idls, oracles, controllers) = (
        entries(),
        tokens(),
        idls(),
        classifier(),
        KnownControllers::default(),
    );
    let inputs = CollectInputs {
        protocols: &protocols,
        tokens: &tokens,
        idls: &idls,
        oracles: &oracles,
        controllers: &controllers,
        programs: &programs(),
        tx_limit: 50,
        history_limit: 300,
        previous: None,
    };
    let mut cache = TxCache::default();
    let out = collect(&rpc, &OtterSec, &inputs, &mut cache).await;
    (out, rpc)
}

#[tokio::test]
async fn col_15_one_broken_protocol_does_not_stop_the_run() {
    let (out, _) = run(5_000).await;
    assert!(!out.budget_exceeded);
    assert_eq!(
        out.facts.protocols.keys().collect::<Vec<_>>(),
        vec!["kamino-lend", "spl-token"]
    );
    assert_eq!(out.failures.len(), 1, "{:#?}", out.failures);
    assert_eq!(out.failures[0].scope, "broken");
    assert!(
        out.failures[0].error.contains("400"),
        "{}",
        out.failures[0].error
    );
}

#[tokio::test]
async fn col_15_collected_kamino_facts_are_complete_and_scorable() {
    let (out, _) = run(5_000).await;
    let k = &out.facts.protocols["kamino-lend"];
    let program = &k.programs[&Address::new("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD")];
    assert!(matches!(
        program.upgrade_control.value(),
        Controller::Multisig {
            threshold: 5,
            voters: 10,
            time_lock_secs: 86_400,
            ..
        }
    ));
    assert!(matches!(
        program.code_transparency.value(),
        CodeTransparency::Verified { warning: None, .. }
    ));
    let admin = k.admin.values().next().unwrap();
    assert!(
        matches!(admin.controller.value(), Controller::SingleKey { key } | Controller::Pda { key, .. } if key.as_str() == "24LjDBukaUSHgPowcF2wY1XscnhChcBUDETN2UhBZMMT")
    );
    let sources: Vec<&OracleSource> = k.oracles.values().map(|o| o.source.value()).collect();
    assert!(sources.contains(&&OracleSource::Network {
        name: "pyth".into()
    }));
    assert!(sources.contains(&&OracleSource::ProtocolRun));
    assert!(
        matches!(k.dependencies.value(), Dependencies::Sampled { callees, .. } if callees.is_empty()),
        "token program dropped"
    );
    assert!(k.min_slot > 0 && k.min_slot <= k.max_slot);
    assert!(out.facts.tokens.contains_key(&Address::new(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    )));

    let scores = celebtir_score::score(&out.facts, &celebtir_score::Methodology::shipped());
    let s = &scores.protocols["kamino-lend"];
    assert_eq!(
        s.factors[&celebtir_score::Factor::UpgradeControl].rating,
        celebtir_score::Rating::Medium,
        "5 of 10 voters is 50 %: under the 60 % Low bar, meets Medium (3+ voters at 50 %+)"
    );
}

#[tokio::test]
async fn col_15_immutable_program_without_transactions_has_unavailable_dependencies() {
    let (out, _) = run(5_000).await;
    let t = &out.facts.protocols["spl-token"];
    let p = t.programs.values().next().unwrap();
    assert_eq!(p.upgrade_control.value(), &Controller::Immutable);
    assert!(matches!(
        t.dependencies.value(),
        Dependencies::Unavailable { .. }
    ));
}

#[tokio::test]
async fn col_5_budget_exceeded_stops_the_run_cleanly() {
    let (out, rpc) = run(3).await;
    assert!(out.budget_exceeded);
    assert_eq!(
        rpc.client().report().total(),
        3,
        "no call beyond the budget"
    );
    assert!(out.failures.iter().any(|f| f.error.contains("budget")));
}

/// Edge case: a program with no transactions in the sample window keeps the
/// previous dependency list, marked stale, with its old evidence.
#[tokio::test]
async fn edge_no_transactions_in_window_keeps_previous_list_marked_stale() {
    use celebtir_core::snapshot::{DependencyFact, FactsFile};
    use celebtir_core::{Chain, Evidence, Fact};

    let (first, _) = run(5_000).await;
    let mut previous: FactsFile = first.facts.clone();
    let callee = Address::new("Ca11ee1111111111111111111111111111111111111");
    let old = Evidence::new(
        Chain::Solana,
        Address::new("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        400_000_000,
        "transaction sample (7 transactions)",
    );
    previous
        .protocols
        .get_mut("spl-token")
        .unwrap()
        .dependencies = Fact::single(
        Dependencies::Sampled {
            callees: BTreeMap::from([(
                callee.clone(),
                DependencyFact {
                    upgrade_control: Fact::single(Controller::Immutable, old.clone()),
                },
            )]),
            stale: false,
            declared: false,
        },
        old.clone(),
    );

    let rpc = SolanaRpc::new(RpcClient::new(
        Mainnet::new(),
        RpcConfig {
            requests_per_sec: 10_000,
            ..RpcConfig::default()
        },
    ));
    let (protocols, tokens, idls, oracles, controllers) = (
        entries(),
        tokens(),
        idls(),
        classifier(),
        KnownControllers::default(),
    );
    let inputs = CollectInputs {
        protocols: &protocols,
        tokens: &tokens,
        idls: &idls,
        oracles: &oracles,
        controllers: &controllers,
        programs: &programs(),
        tx_limit: 50,
        history_limit: 300,
        previous: Some(&previous),
    };
    let out = collect(&rpc, &OtterSec, &inputs, &mut TxCache::default()).await;
    let deps = &out.facts.protocols["spl-token"].dependencies;
    match deps.value() {
        Dependencies::Sampled {
            callees,
            stale: true,
            ..
        } => assert!(callees.contains_key(&callee)),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        deps.evidence()[0],
        old,
        "the stale list keeps its original evidence and slot"
    );

    let s = celebtir_score::factors::dependencies(deps, &celebtir_score::Methodology::shipped());
    assert!(s.reason.contains("carried over"), "{}", s.reason);
}

/// VAL-7: with DefiLlama down (the mock answers 404), TVL shows as
/// unavailable and the ratings are still published.
#[tokio::test]
async fn val_7_defillama_outage_still_publishes_ratings() {
    let (out, _) = run(5_000).await;
    let m = &out.facts.metrics["kamino-lend"];
    assert!(
        matches!(m.tvl, celebtir_core::snapshot::Metric::Unavailable { .. }),
        "{:?}",
        m.tvl
    );
    let scores = celebtir_score::score(&out.facts, &celebtir_score::Methodology::shipped());
    assert_eq!(scores.protocols.len(), 2);
}

/// Edge case 11 end to end: a Scope oracle with a registry chain carries the
/// feeds behind it; an oracle without one carries none.
#[tokio::test]
async fn edge_11_collect_lists_the_feeds_behind_a_scope_price() {
    let (out, _) = run(5_000).await;
    let k = &out.facts.protocols["kamino-lend"];
    let scope = &k.oracles[&Address::new("3t4JZcueEzTbVP6kLxXrL3VpWx45jDer4eqysweBchNH")];
    let Some(underlying) = &scope.underlying else {
        panic!("no underlying feeds: {scope:?}");
    };
    let UnderlyingFeeds::Listed { feeds } = underlying.value() else {
        panic!("{:?}", underlying.value());
    };
    let types: Vec<&str> = feeds.iter().map(|f| f.source_type.as_str()).collect();
    assert_eq!(types, vec!["Chainlink", "PythLazer"]);
    let old = &k.oracles[&Address::new("3NJYftD5sjVfxSnUdZ1wVML8f3aC6mp1CXCL6L7TnU8C")];
    assert_eq!(old.underlying, None);
}
