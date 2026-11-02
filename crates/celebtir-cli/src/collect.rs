//! `celebtir collect`: registry → facts.json + run.json for one day. The I/O
//! around the collector: registry files, IDLs, the transaction cache, the
//! clock (run.json only) and the RPC endpoint.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use celebtir_core::snapshot::{
    ArchivedProtocol, Hip3Snapshot, RUN_SCHEMA_VERSION, RunChain, RunFailure, RunFile, RunNote,
};
use celebtir_core::to_canonical_json;
use collect_solana::collector::{CollectInputs, collect};
use collect_solana::deps::TxCache;
use collect_solana::idl::Idl;
use collect_solana::oracle::OracleClassifier;
use collect_solana::reads::SolanaRpc;
use collect_solana::replay::{Recording, Replay};
use collect_solana::resolve::KnownControllers;
use collect_solana::rpc::{HttpTransport, RpcClient, RpcConfig, Transport};
use collect_solana::verify::{HttpGet, ReqwestGet};

use crate::{CliError, Format, rpc_endpoint};

pub struct CollectArgs {
    pub registry: PathBuf,
    pub out: Option<PathBuf>,
    pub protocol: Option<String>,
    pub budget: u64,
    pub cache: PathBuf,
    pub date: Option<String>,
    /// Transactions sampled per core program (COL-14).
    pub tx_sample: usize,
    /// New ProgramData transactions read per program per run (VAL-4).
    pub history_limit: usize,
    /// Replay recorded responses from this directory instead of the network (CLI-4).
    pub offline: Option<PathBuf>,
    /// Save every response under this directory while collecting.
    pub record: Option<PathBuf>,
    /// The previous facts.json, to carry over stale dependency lists.
    pub previous: PathBuf,
    /// Also collect Hyperliquid (wave 2, HOPS-1).
    pub hyperliquid: bool,
    pub hyperevm_budget: u64,
}

/// `SystemTime` → RFC 3339 UTC, second precision.
pub fn rfc3339(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    celebtir_core::date::rfc3339(secs)
}

fn read(path: &Path) -> Result<String, CliError> {
    std::fs::read_to_string(path)
        .map_err(|e| CliError::Validation(format!("cannot read {}: {e}", path.display())))
}

fn write(path: &Path, text: &str) -> Result<(), CliError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| CliError::Validation(format!("cannot create {}: {e}", dir.display())))?;
    }
    std::fs::write(path, text)
        .map_err(|e| CliError::Validation(format!("cannot write {}: {e}", path.display())))
}

pub fn run(args: CollectArgs, format: Format) -> Result<(), CliError> {
    if let Some(dir) = args.offline.clone() {
        let replay = || Replay { dir: dir.clone() };
        return run_with(args, format, replay(), replay(), "offline".into(), 100_000);
    }
    let (endpoint, warning) = rpc_endpoint(std::env::var("CELEBTIR_RPC_URL").ok());
    if let Some(w) = warning {
        eprintln!("{w}");
    }
    let host = endpoint.host().to_string();
    match args.record.clone() {
        Some(dir) => {
            let transport = Recording {
                inner: HttpTransport::new(endpoint),
                dir: dir.clone(),
            };
            let http = Recording {
                inner: ReqwestGet::default(),
                dir,
            };
            run_with(args, format, transport, http, host, 8)
        }
        None => run_with(
            args,
            format,
            HttpTransport::new(endpoint),
            ReqwestGet::default(),
            host,
            8,
        ),
    }
}

/// The command with its network parts injected (offline replay uses this).
pub fn run_with<T: Transport, H: HttpGet>(
    args: CollectArgs,
    format: Format,
    transport: T,
    http: H,
    rpc_host: String,
    requests_per_sec: u32,
) -> Result<(), CliError> {
    let started = SystemTime::now();
    let clock = Instant::now();
    let root = &args.registry;

    let report = celebtir_registry::validate(root);
    if !report.is_ok() {
        for d in &report.diagnostics {
            eprintln!("{d}");
        }
        return Err(CliError::Validation(format!(
            "registry has {} problems; run `celebtir validate`",
            report.diagnostics.len()
        )));
    }
    let mut protocols = report.protocols.clone();
    if let Some(id) = &args.protocol {
        protocols.retain(|p| &p.id == id);
        let hyperevm = args.hyperliquid && report.hyperevm_protocols.iter().any(|p| &p.id == id);
        if protocols.is_empty() && !hyperevm {
            return Err(CliError::Validation(format!(
                "no protocol `{id}` in the registry"
            )));
        }
    }

    let oracles = match std::fs::read_to_string(root.join("oracles.yaml")) {
        Ok(t) => OracleClassifier::from_yaml(&t)
            .map_err(|e| CliError::Validation(format!("oracles.yaml: {e}")))?,
        Err(_) => OracleClassifier::default(),
    };
    let mut idls = BTreeMap::new();
    let idl_paths = protocols
        .iter()
        .flat_map(|p| {
            p.admin_accounts.iter().map(|a| &a.idl).chain(
                p.oracles
                    .iter()
                    .filter_map(|o| o.scope_chain.as_ref().map(|c| &c.idl)),
            )
        })
        .chain(oracles.programs().filter_map(|p| p.idl.as_ref()));
    for path in idl_paths {
        if !idls.contains_key(path) {
            let idl = Idl::from_json(&read(&root.join(path))?)
                .map_err(|e| CliError::Validation(format!("{path}: {e}")))?;
            idls.insert(path.clone(), idl);
        }
    }
    let controllers = match std::fs::read_to_string(root.join("controllers.yaml")) {
        Ok(t) => KnownControllers::from_yaml(&t)
            .map_err(|e| CliError::Validation(format!("controllers.yaml: {e}")))?,
        Err(_) => KnownControllers::default(),
    };
    let mut cache: TxCache = std::fs::read_to_string(&args.cache)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();

    let previous: Option<celebtir_core::snapshot::FactsFile> =
        std::fs::read_to_string(&args.previous)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());

    let rpc = SolanaRpc::new(RpcClient::new(
        transport,
        RpcConfig {
            daily_budget: args.budget,
            requests_per_sec,
            ..RpcConfig::default()
        },
    ));
    let inputs = CollectInputs {
        protocols: &protocols,
        tokens: &report.tokens,
        idls: &idls,
        oracles: &oracles,
        controllers: &controllers,
        programs: &report.programs,
        tx_limit: args.tx_sample,
        history_limit: args.history_limit,
        previous: previous.as_ref(),
    };
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::Partial(format!("cannot start the async runtime: {e}")))?;
    let mut out = runtime.block_on(collect(&rpc, &http, &inputs, &mut cache));
    // ADR 017: archived protocols ride along in the facts for the site.
    out.facts.archived = report
        .archived
        .iter()
        .map(|a| {
            (
                a.id.clone(),
                ArchivedProtocol {
                    name: a.name.clone(),
                    archived_on: a.archived_on.clone(),
                    reason: a.reason.clone(),
                    source: a.source.to_string(),
                },
            )
        })
        .collect();

    // Wave 2: Hyperliquid joins the same facts file, after Solana, so a
    // HyperEVM outage never touches Solana results (HCOL-11). Offline
    // replays cover Solana only.
    let hyperliquid = if args.hyperliquid && args.offline.is_none() {
        Some(runtime.block_on(collect_hyperliquid(
            root,
            &report,
            args.protocol.as_deref(),
            args.hyperevm_budget,
        ))?)
    } else {
        None
    };
    if let Some(h) = &hyperliquid {
        for w in &h.output.warnings {
            eprintln!("warning: {w}");
        }
        out.facts.protocols.extend(h.output.protocols.clone());
        out.facts.hip3 = Some(h.output.hip3.clone());
        out.failures
            .extend(h.output.failures.iter().map(|(scope, error)| {
                collect_solana::collector::Failure {
                    scope: scope.clone(),
                    error: error.clone(),
                }
            }));
    }

    let date = args
        .date
        .unwrap_or_else(|| rfc3339(started)[..10].to_string());
    let dir = args
        .out
        .unwrap_or_else(|| PathBuf::from("data/snapshots").join(&date));
    let calls = rpc.client().report();
    let run = RunFile {
        schema_version: RUN_SCHEMA_VERSION.to_string(),
        tool_version: match option_env!("CELEBTIR_COMMIT") {
            Some(c) => format!("{} ({c})", env!("CARGO_PKG_VERSION")),
            None => env!("CARGO_PKG_VERSION").to_string(),
        },
        started_at: rfc3339(started),
        finished_at: rfc3339(SystemTime::now()),
        duration_ms: clock.elapsed().as_millis() as u64,
        rpc_host,
        calls: calls.calls.clone(),
        retries: calls.retries,
        budget: args.budget,
        budget_exceeded: out.budget_exceeded,
        protocols: out.facts.protocols.keys().cloned().collect(),
        failures: out
            .failures
            .iter()
            .map(|f| RunFailure {
                scope: f.scope.clone(),
                error: f.error.clone(),
            })
            .collect(),
        review_needed: out
            .review_needed
            .iter()
            .map(|n| RunNote {
                protocol: n.protocol.clone(),
                note: n.note.clone(),
            })
            .collect(),
        hyperliquid: hyperliquid.as_ref().map(|h| RunChain {
            rpc_host: h.rpc_host.clone(),
            block: h.output.block,
            calls: h.output.calls.calls.clone(),
            retries: h.output.calls.retries,
            budget: args.hyperevm_budget,
            info_calls: h.output.info_calls,
            protocols: h.output.protocols.keys().cloned().collect(),
            hip3_dexs: match &h.output.hip3 {
                Hip3Snapshot::Listed { dexs } => dexs.len() as u64,
                Hip3Snapshot::Unavailable { .. } => 0,
            },
            failures: h.output.failures.len() as u64,
        }),
    };
    write(&dir.join("facts.json"), &json(&out.facts)?)?;
    // HIP-2: the raw response, so a day can be checked against the API.
    if let Some(raw) = hyperliquid
        .as_ref()
        .and_then(|h| h.output.raw_perp_dexs.as_ref())
    {
        write(&dir.join("raw").join("perpDexs.json"), &json(raw)?)?;
    }
    write(&dir.join("run.json"), &json(&run)?)?;
    write(&args.cache, &json(&cache)?)?;

    match format {
        Format::Json => print!("{}", json(&run)?),
        Format::Text => {
            println!(
                "collected {} protocols and {} tokens in {} ms with {} RPC calls → {}",
                run.protocols.len(),
                out.facts.tokens.len(),
                run.duration_ms,
                calls.total(),
                dir.display()
            );
            for f in &run.failures {
                println!("failed: {}: {}", f.scope, f.error);
            }
            for n in &run.review_needed {
                println!("review: {}: {}", n.protocol, n.note);
            }
        }
    }
    if out.budget_exceeded {
        Err(CliError::BudgetExceeded(format!(
            "stopped at the daily budget of {} calls",
            args.budget
        )))
    } else if !run.failures.is_empty() {
        Err(CliError::Partial(format!(
            "{} of the run failed; see run.json",
            run.failures.len()
        )))
    } else {
        Ok(())
    }
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, CliError> {
    to_canonical_json(value).map_err(|e| CliError::Validation(e.to_string()))
}

struct HyperliquidRun {
    output: collect_hyperliquid::pipeline::Output,
    rpc_host: String,
}

/// The live Hyperliquid half of `collect` (HOPS-1).
async fn collect_hyperliquid(
    root: &Path,
    report: &celebtir_registry::Report,
    only: Option<&str>,
    budget: u64,
) -> Result<HyperliquidRun, CliError> {
    use collect_hyperliquid::collect::EvmOracles;
    use collect_hyperliquid::evm::{default_config, hyperevm_endpoint};
    use collect_hyperliquid::resolve::EvmKnown;

    let (endpoint, warning) = hyperevm_endpoint(std::env::var("CELEBTIR_HYPEREVM_RPC_URL").ok());
    if let Some(w) = warning {
        eprintln!("{w}");
    }
    let controllers = EvmKnown::controllers_from_yaml(&read(&root.join("controllers.yaml"))?)
        .map_err(|e| CliError::Validation(format!("controllers.yaml: {e}")))?;
    let oracles = EvmOracles::from_yaml(&read(&root.join("oracles.yaml"))?)
        .map_err(|e| CliError::Validation(format!("oracles.yaml: {e}")))?;
    let hip3_path = root.join("hyperliquid").join("hip3.yaml");
    let hip3: celebtir_registry::Hip3File = serde_norway::from_str(&read(&hip3_path)?)
        .map_err(|e| CliError::Validation(format!("hyperliquid/hip3.yaml: {e}")))?;
    let protocols: Vec<_> = report
        .hyperevm_protocols
        .iter()
        .filter(|p| only.is_none_or(|id| p.id == id))
        .cloned()
        .collect();
    let key = std::env::var("CELEBTIR_ETHERSCAN_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty());
    let info = HttpTransport::new(celebtir_core::RpcEndpoint::new(
        "https://api.hyperliquid.xyz/info",
    ));
    let inputs = collect_hyperliquid::pipeline::Inputs {
        protocols: &protocols,
        controllers: &controllers,
        oracles: &oracles,
        hip3: &hip3,
        etherscan_key: key.as_deref(),
    };
    let rpc_host = endpoint.host().to_string();
    let output = collect_hyperliquid::pipeline::collect(
        HttpTransport::new(endpoint),
        &info,
        &ReqwestGet::default(),
        RpcConfig {
            daily_budget: budget,
            ..default_config()
        },
        &inputs,
    )
    .await;
    Ok(HyperliquidRun { output, rpc_host })
}
