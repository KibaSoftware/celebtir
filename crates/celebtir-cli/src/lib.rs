//! The `celebtir` command line: one subcommand per pipeline stage, so the daily
//! job and a contributor's laptop run the same code.

use std::path::PathBuf;

use celebtir_core::{RpcEndpoint, to_canonical_json};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

pub mod api;
pub mod collect;
mod compact;
pub mod coverage;
mod score;
pub mod site_data;
mod summarize;

/// Used when `CELEBTIR_RPC_URL` is unset (CLI-1). Rate-limited; fine for a
/// local run, not for the daily job.
pub const PUBLIC_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[derive(Debug, Parser)]
#[command(
    name = "celebtir",
    version = concat!(env!("CARGO_PKG_VERSION"), " (commit ", env!("CELEBTIR_COMMIT"), ")"),
    about = "An open book for DeFi: who controls each protocol."
)]
pub struct Cli {
    /// Output format.
    #[arg(long, value_enum, global = true, default_value_t = Format::Text)]
    pub format: Format,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check registry files against the schema and the rules (REG-1 to REG-5).
    Validate {
        /// The registry directory.
        #[arg(long, default_value = "registry")]
        registry: PathBuf,
        /// Also check that every program ID is an executable account on mainnet (REG-2).
        #[arg(long)]
        chain: bool,
    },
    /// Apply the methodology to a facts file and write scores.json next to it.
    Score {
        /// A facts.json file.
        #[arg(long)]
        facts: PathBuf,
        /// Where to write scores.json (default: next to the facts file).
        #[arg(long)]
        out: Option<PathBuf>,
        /// A methodology TOML file (default: the shipped v0.3).
        #[arg(long)]
        methodology: Option<PathBuf>,
    },
    /// Read the chain and external sources for every registry protocol; write
    /// facts.json and run.json.
    Collect {
        #[arg(long, default_value = "registry")]
        registry: PathBuf,
        /// Output folder (default: data/snapshots/<date>).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Collect one protocol only.
        #[arg(long)]
        protocol: Option<String>,
        /// RPC requests allowed in this run (COL-5).
        #[arg(long, default_value_t = 5_000)]
        budget: u64,
        /// Transaction-sample cache kept between runs.
        #[arg(long, default_value = "data/cache/tx-cache.json")]
        cache: PathBuf,
        /// Snapshot date, YYYY-MM-DD (default: today, UTC).
        #[arg(long)]
        date: Option<String>,
        /// Transactions sampled per core program (COL-14).
        #[arg(long, default_value_t = 50)]
        tx_sample: usize,
        /// New ProgramData transactions read per program per run (VAL-4).
        #[arg(long, default_value_t = 300)]
        history_limit: usize,
        /// Replay recorded responses from this directory; no network (CLI-4).
        #[arg(long, conflicts_with = "record")]
        offline: Option<PathBuf>,
        /// Record every response under this directory while collecting.
        #[arg(long)]
        record: Option<PathBuf>,
        /// The previous facts.json (stale dependency lists are carried over).
        #[arg(long, default_value = "data/latest/facts.json")]
        previous: PathBuf,
        /// Also collect Hyperliquid: HyperEVM protocols in
        /// `registry/hyperliquid/` and every HIP-3 DEX (wave 2, HOPS-1).
        #[arg(long)]
        hyperliquid: bool,
        /// HyperEVM JSON-RPC requests allowed in this run.
        #[arg(long, default_value_t = 2_000)]
        hyperevm_budget: u64,
    },
    /// Compare two snapshot folders and write events.json into the newer one.
    /// With `--initial NEW`, write the first snapshot's events: its recorded
    /// ProgramData history, flagged as backfill (F6).
    Diff {
        /// The previous snapshot folder (its name is its date), or with
        /// `--initial` the only snapshot.
        old: PathBuf,
        /// Today's snapshot folder, holding facts.json and scores.json.
        new: Option<PathBuf>,
        #[arg(long)]
        initial: bool,
    },
    /// Derive data/latest/ and data/history/ for the site from the snapshots.
    BuildSiteData {
        /// The data folder holding snapshots/.
        #[arg(long, default_value = "data")]
        data: PathBuf,
        /// Date for "last 30 days" metrics, YYYY-MM-DD (default: today, UTC).
        #[arg(long)]
        as_of: Option<String>,
    },
    /// Compact snapshots older than 180 days to one per week (SNAP-7).
    Compact {
        #[arg(long, default_value = "data")]
        data: PathBuf,
        /// Today, YYYY-MM-DD (default: today, UTC).
        #[arg(long)]
        as_of: Option<String>,
    },
    /// One-line summary of a snapshot, and projected RPC usage (OPS-5, OPS-6).
    Summarize {
        /// A snapshot folder holding run.json (and events.json).
        dir: PathBuf,
    },
    /// Print each factor, the accounts read, the reason and the next step.
    Explain {
        /// Registry protocol ID, e.g. kamino-lend.
        protocol: String,
        #[arg(long, default_value = "data/latest/facts.json")]
        facts: PathBuf,
        #[arg(long)]
        methodology: Option<PathBuf>,
    },
}

/// Process exit codes (CLI-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Exit {
    Success = 0,
    Validation = 1,
    Partial = 2,
    BudgetExceeded = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// Bad input: registry, arguments or files.
    Validation(String),
    /// Some protocols failed; the rest of the run completed.
    Partial(String),
    /// The daily call budget was reached (COL-5).
    BudgetExceeded(String),
}

impl CliError {
    pub fn exit(&self) -> Exit {
        match self {
            CliError::Validation(_) => Exit::Validation,
            CliError::Partial(_) => Exit::Partial,
            CliError::BudgetExceeded(_) => Exit::BudgetExceeded,
        }
    }
}

/// The RPC endpoint from `CELEBTIR_RPC_URL`, or the public endpoint plus a
/// warning to print when it is unset or blank (CLI-1).
pub fn rpc_endpoint(env_value: Option<String>) -> (RpcEndpoint, Option<String>) {
    match env_value.filter(|v| !v.trim().is_empty()) {
        Some(url) => (RpcEndpoint::new(url.trim()), None),
        None => (
            RpcEndpoint::new(PUBLIC_RPC_URL),
            Some(format!(
                "warning: CELEBTIR_RPC_URL is not set; using the rate-limited public endpoint {PUBLIC_RPC_URL}"
            )),
        ),
    }
}

/// Runs a parsed command, writing results to stdout and problems to stderr.
pub fn run(cli: Cli) -> Exit {
    let result = match cli.command {
        Command::Validate { registry, chain } => validate(&registry, chain, cli.format),
        Command::Collect {
            registry,
            out,
            protocol,
            budget,
            cache,
            date,
            tx_sample,
            history_limit,
            offline,
            record,
            previous,
            hyperliquid,
            hyperevm_budget,
        } => collect::run(
            collect::CollectArgs {
                registry,
                out,
                protocol,
                budget,
                cache,
                date,
                tx_sample,
                history_limit,
                offline,
                record,
                previous,
                hyperliquid,
                hyperevm_budget,
            },
            cli.format,
        ),
        Command::Score {
            facts,
            out,
            methodology,
        } => score::score(&facts, out.as_deref(), methodology.as_deref(), cli.format),
        Command::Diff { old, new, initial } => match (initial, new) {
            (true, None) => score::diff_initial(&old, cli.format),
            (false, Some(new)) => score::diff(&old, &new, cli.format),
            _ => Err(CliError::Validation(
                "diff takes OLD NEW, or --initial with one snapshot folder".into(),
            )),
        },
        Command::Summarize { dir } => summarize::run(&dir, cli.format),
        Command::Compact { data, as_of } => {
            let as_of = as_of.unwrap_or_else(|| {
                collect::rfc3339(std::time::SystemTime::now())[..10].to_string()
            });
            compact::run(&data, &as_of, cli.format)
        }
        Command::BuildSiteData { data, as_of } => {
            let as_of = as_of.unwrap_or_else(|| {
                collect::rfc3339(std::time::SystemTime::now())[..10].to_string()
            });
            site_data::run(&data, &as_of, cli.format)
        }
        Command::Explain {
            protocol,
            facts,
            methodology,
        } => score::explain(&protocol, &facts, methodology.as_deref(), cli.format),
    };
    match result {
        Ok(()) => Exit::Success,
        Err(e) => {
            let (CliError::Validation(msg)
            | CliError::Partial(msg)
            | CliError::BudgetExceeded(msg)) = &e;
            eprintln!("error: {msg}");
            e.exit()
        }
    }
}

#[derive(Serialize)]
struct ValidateOutput {
    ok: bool,
    protocols: usize,
    /// HyperEVM protocols in `hyperliquid/` (wave 2).
    hyperevm_protocols: usize,
    tokens: usize,
    diagnostics: Vec<DiagnosticOutput>,
}

#[derive(Serialize)]
struct DiagnosticOutput {
    file: String,
    pointer: String,
    message: String,
}

fn validate(root: &std::path::Path, chain: bool, format: Format) -> Result<(), CliError> {
    if !root.join("solana").is_dir() {
        return Err(CliError::Validation(format!(
            "{} is not a registry directory (no solana/ folder)",
            root.display()
        )));
    }
    let mut report = celebtir_registry::validate(root);
    if chain && report.is_ok() {
        report.diagnostics.extend(chain_check(&report.protocols)?);
    }
    let out = ValidateOutput {
        ok: report.is_ok(),
        protocols: report.protocols.len(),
        hyperevm_protocols: report.hyperevm_protocols.len(),
        tokens: report.tokens.len(),
        diagnostics: report
            .diagnostics
            .iter()
            .map(|d| DiagnosticOutput {
                file: d.file.to_string_lossy().replace('\\', "/"),
                pointer: d.pointer.clone(),
                message: d.message.clone(),
            })
            .collect(),
    };
    match format {
        Format::Json => print!(
            "{}",
            to_canonical_json(&out).map_err(|e| CliError::Validation(e.to_string()))?
        ),
        Format::Text => {
            for d in &report.diagnostics {
                println!("{d}");
            }
            let hyperevm = if out.hyperevm_protocols > 0 {
                format!(" (+{} HyperEVM)", out.hyperevm_protocols)
            } else {
                String::new()
            };
            println!(
                "{}: {} protocols{hyperevm}, {} tokens, {} problems",
                if out.ok { "ok" } else { "failed" },
                out.protocols,
                out.tokens,
                out.diagnostics.len()
            );
        }
    }
    if out.ok {
        Ok(())
    } else {
        Err(CliError::Validation(format!(
            "{} registry problems",
            out.diagnostics.len()
        )))
    }
}

/// REG-2 on-chain: one batched existence read over every registry program.
fn chain_check(
    protocols: &[celebtir_registry::ProtocolEntry],
) -> Result<Vec<celebtir_registry::Diagnostic>, CliError> {
    use collect_solana::reads::SolanaRpc;
    use collect_solana::rpc::{HttpTransport, RpcClient, RpcConfig};

    let (endpoint, warning) = rpc_endpoint(std::env::var("CELEBTIR_RPC_URL").ok());
    if let Some(w) = warning {
        eprintln!("{w}");
    }
    let ids: Vec<celebtir_core::Address> = protocols
        .iter()
        .flat_map(|p| p.programs.iter().map(|g| g.id.clone()))
        .collect();
    let rpc = SolanaRpc::new(RpcClient::new(
        HttpTransport::new(endpoint),
        RpcConfig::default(),
    ));
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::Partial(format!("cannot start the async runtime: {e}")))?;
    let problems = runtime
        .block_on(collect_solana::registry_check::check_programs(&rpc, &ids))
        .map_err(|e| CliError::Partial(format!("on-chain check did not complete: {e}")))?;
    let mut out = Vec::new();
    for (address, why) in problems {
        for p in protocols {
            if let Some(i) = p.programs.iter().position(|g| g.id == address) {
                out.push(celebtir_registry::Diagnostic {
                    file: PathBuf::from(format!("solana/{}.yaml", p.id)),
                    pointer: format!("/programs/{i}/id"),
                    message: format!("{why} (REG-2)"),
                });
            }
        }
    }
    Ok(out)
}
