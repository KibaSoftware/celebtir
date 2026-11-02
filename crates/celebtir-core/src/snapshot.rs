//! Types of the files written per daily run. They grow slice by slice; every
//! collection is a map keyed by a stable ID so output order is deterministic.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Address, Chain, Controller, Fact};

/// Version of the `facts.json` format. Bump the major version for any removal
/// or rename (SNAP-3).
pub const FACTS_SCHEMA_VERSION: &str = "0.2.0";

/// `data/snapshots/<date>/facts.json`. Holds no timestamps: the date lives in
/// the folder name and run timing lives in `run.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FactsFile {
    pub schema_version: String,
    /// Keyed by registry protocol ID.
    pub protocols: BTreeMap<String, ProtocolFacts>,
    /// Keyed by mint address.
    #[serde(default)]
    pub tokens: BTreeMap<Address, TokenFacts>,
    /// Context numbers from DefiLlama, keyed by protocol ID. Shown, never
    /// used in a rating (VAL-1, VAL-7).
    #[serde(default)]
    pub metrics: BTreeMap<String, ProtocolMetrics>,
    /// Token prices from DefiLlama, keyed by mint; context only (TOK-9).
    #[serde(default)]
    pub token_metrics: BTreeMap<Address, TokenMetrics>,
    /// Transactions on each upgradeable core program's ProgramData, oldest
    /// first, keyed by program (VAL-4).
    #[serde(default)]
    pub program_history: BTreeMap<Address, ProgramHistory>,
    /// Protocols that left coverage, from `registry/archived.yaml` (ADR 017),
    /// keyed by their former ID.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub archived: BTreeMap<String, ArchivedProtocol>,
    /// HIP-3 perp DEXs (wave 2); None when Hyperliquid was not collected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hip3: Option<Hip3Snapshot>,
}

/// HIP-3 perp DEXs from the info API (wave 2, HIP-1 to HIP-5). Absent in
/// Solana-only runs, so their output is unchanged (HSCO-5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Hip3Snapshot {
    /// Keyed by DEX name.
    Listed {
        dexs: BTreeMap<String, Hip3DexFacts>,
    },
    /// The info API failed or answered something unparseable (HIP-5).
    Unavailable { reason: String },
}

/// One builder-deployed perp DEX. Every fact is anchored to the HyperEVM
/// block read right before the request (ADR 024 D6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hip3DexFacts {
    pub full_name: String,
    pub deployer: Fact<Address>,
    /// None: not delegated, the deployer pushes prices.
    pub oracle_updater: Fact<Option<Address>>,
    /// Per-action grants to other addresses (`setOracle`, `registerAsset`,
    /// …), each list sorted (ADR 024, 9.5 note).
    pub sub_deployers: Fact<BTreeMap<String, Vec<Address>>>,
    pub fee_recipient: Fact<Option<Address>>,
    /// Asset → streaming open-interest cap, as the API's decimal string.
    pub oi_caps: Fact<BTreeMap<String, String>>,
    /// Asset → funding multiplier, as the API's decimal string.
    pub funding_multipliers: Fact<BTreeMap<String, String>>,
    pub stake: Fact<Hip3Stake>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Hip3Stake {
    /// From `delegatorSummary`; amounts are HYPE as decimal strings.
    Read {
        delegated: String,
        undelegated: String,
        pending_withdrawal: String,
        /// The required stake from `registry/hyperliquid/hip3.yaml`.
        required: String,
        stake_below_required: bool,
        unstaking: bool,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArchivedProtocol {
    pub name: String,
    /// `YYYY-MM-DD`.
    pub archived_on: String,
    pub reason: String,
    pub source: String,
}

impl FactsFile {
    pub fn new(protocols: BTreeMap<String, ProtocolFacts>) -> Self {
        Self {
            schema_version: FACTS_SCHEMA_VERSION.to_string(),
            protocols,
            tokens: BTreeMap::new(),
            metrics: BTreeMap::new(),
            program_history: BTreeMap::new(),
            token_metrics: BTreeMap::new(),
            archived: BTreeMap::new(),
            hip3: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtocolFacts {
    pub chain: Chain,
    /// Registry category (`lending`, `dex`, `perps`, `liquid-staking`), for
    /// grouping only.
    #[serde(default)]
    pub category: Option<String>,
    /// The registry entry's `last_reviewed` date, for the review-due note (REG-7).
    #[serde(default)]
    pub last_reviewed: Option<String>,
    /// Display name and website from the registry.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    /// Who last reviewed the registry entry.
    #[serde(default)]
    pub reviewed_by: Option<String>,
    /// Where each registry address was confirmed, sorted by kind then
    /// address (shown on the protocol page, UI-12).
    #[serde(default)]
    pub sources: Vec<RegistrySource>,
    /// Keyed by program address.
    pub programs: BTreeMap<Address, ProgramFacts>,
    /// Admin keys outside upgrades, keyed by `<account>.<field>`.
    pub admin: BTreeMap<String, AdminFact>,
    /// Price feeds, keyed by oracle account.
    pub oracles: BTreeMap<Address, OracleFact>,
    /// A sourced registry review that found no admin accounts. Without it,
    /// an empty `admin` map means "not yet read", never "none" (v0.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_review: Option<ReviewedNone>,
    /// A sourced registry review that found no price feeds (v0.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle_review: Option<ReviewedNone>,
    /// Programs the core programs call with user funds.
    pub dependencies: Fact<Dependencies>,
    /// Tokens the protocol uses and how, from the registry (TOK-2); keyed by mint.
    #[serde(default)]
    pub tokens: BTreeMap<Address, Vec<TokenRole>>,
    /// Lowest and highest slot among this protocol's reads (COL-3).
    #[serde(default)]
    pub min_slot: u64,
    #[serde(default)]
    pub max_slot: u64,
}

/// A registry statement that a protocol has none of something (admin
/// accounts, price feeds), with the public page that shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewedNone {
    /// The source or documentation the review relied on.
    pub source: String,
    /// `YYYY-MM-DD`.
    pub reviewed_on: String,
    /// What was checked, in one sentence.
    pub note: String,
}

/// A registry address and the public page that confirms it (REG-3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegistrySource {
    pub kind: SourceKind,
    pub address: Address,
    /// Program name, admin field or oracle market.
    pub label: String,
    pub url: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Admin,
    Multisig,
    Oracle,
    Program,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgramFacts {
    pub role: ProgramRole,
    /// Owner of the program account (COL-7); None when it was not read.
    #[serde(default)]
    pub loader: Option<Address>,
    pub upgrade_control: Fact<Controller>,
    pub code_transparency: Fact<CodeTransparency>,
}

/// `core` programs hold or move user funds and decide the Stage (REG-5).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRole {
    Core,
    Periphery,
}

/// Does the deployed binary match public source? (OtterSec, COL-13)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CodeTransparency {
    Verified {
        repo: String,
        commit: String,
        on_chain_hash: String,
        executable_hash: String,
        /// Set when the verified repo is outside the registry's GitHub org (COL-13).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        warning: Option<String>,
    },
    PublicSourceUnverified {
        repo: String,
        /// Full 40-character commit, when the registry records one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        commit: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        on_chain_hash: Option<String>,
    },
    NoPublicSource,
    Unavailable {
        reason: String,
    },
}

/// What an admin key can do outside an upgrade.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AdminPower {
    Parameters,
    Pause,
    Funds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdminFact {
    pub account: Address,
    pub field: String,
    pub power: AdminPower,
    /// The key in `field`, resolved to its effective controller. `Immutable`
    /// means the field holds no key (renounced).
    pub controller: Fact<Controller>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum OracleRole {
    Primary,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OracleFact {
    pub market: String,
    pub role: OracleRole,
    /// The protocol rejects a primary price outside a threshold of this
    /// feed, per the registry's source (v0.3 guarded-primary rule).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bounds_primary: bool,
    /// For a primary: the oracle networks its publisher documents as
    /// bounding it. Each must be registered as a bounding fallback before the
    /// guard counts (v0.3, D3); a documented guard that is not is a gap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guarded_by: Vec<String>,
    /// Who publishes the price, when the owner program alone does not say
    /// (e.g. "Edge by Chaos Labs" on a Doves account).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub source: Fact<OracleSource>,
    /// The feeds a protocol-run aggregator combines for this market, where
    /// the registry says how to read them (edge case 11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlying: Option<Fact<UnderlyingFeeds>>,
    /// The same, per further market reading this account (Kamino: every
    /// reserve of the market), keyed by market name (F4).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub underlying_markets: BTreeMap<String, Fact<UnderlyingFeeds>>,
}

/// What an aggregator (Kamino Scope) reads for one market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnderlyingFeeds {
    /// Leaf sources, sorted by type then address.
    Listed {
        feeds: Vec<UnderlyingFeed>,
    },
    Unavailable {
        reason: String,
    },
}

/// One source an aggregator combines.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct UnderlyingFeed {
    /// The aggregator's own name for the source type (Scope: `Chainlink`,
    /// `PythLazer`, `OrcaWhirlpoolAtoB`, …), from its vendored IDL.
    pub source_type: String,
    /// The address the aggregator maps the entry to: an oracle or pool
    /// account, or a feed ID for push-based sources.
    pub address: Address,
}

/// Who publishes the price, from the oracle account's owner program (COL-12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OracleSource {
    /// An oracle network named in `registry/oracles.yaml` (`pyth`, …).
    Network {
        name: String,
    },
    /// A feed run by the protocol itself or by a single signer.
    ProtocolRun,
    /// An owner program `oracles.yaml` does not know.
    Other {
        owner_program: Address,
    },
    Unavailable {
        reason: String,
    },
}

/// Direct callees of the core programs, from a transaction sample (COL-14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Dependencies {
    Sampled {
        /// Keyed by callee program address.
        callees: BTreeMap<Address, DependencyFact>,
        /// True when the latest window had no transactions and this list is
        /// carried over from an earlier run; its evidence keeps the old slot.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        stale: bool,
        /// True when the list comes from the registry instead of a
        /// transaction sample (HyperEVM in wave 2, HCOL-10).
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        declared: bool,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DependencyFact {
    pub upgrade_control: Fact<Controller>,
}

/// One mint's control facts (TOK-3 to TOK-5). Every authority is resolved to
/// its effective controller; `Immutable` means the authority is not set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenFacts {
    pub symbol: String,
    pub program: TokenProgram,
    /// Supply in base units at the read slot (TOK-9).
    #[serde(default)]
    pub supply: u64,
    #[serde(default)]
    pub decimals: u8,
    pub mint_authority: Fact<Controller>,
    pub freeze_authority: Fact<Controller>,
    /// Token-2022 extensions; empty for classic SPL Token mints.
    pub extensions: Fact<TokenExtensions>,
    /// From `registry/tokens.yaml`; the evidence is the mint account.
    pub provenance: Fact<TokenProvenance>,
    /// Type, issuer and source from `registry/tokens.yaml` (TOK-1), for the
    /// token page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<TokenIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenIdentity {
    /// `stablecoin`, `liquid-staking`, `bridged`, `governance` or `other`.
    pub kind: String,
    pub issuer: String,
    pub issuer_url: String,
    /// Confirms the mint, symbol and type.
    pub source: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TokenProgram {
    SplToken,
    Token2022,
}

/// Token-2022 powers. `None` means the extension is absent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenExtensions {
    /// Can move tokens out of any wallet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permanent_delegate: Option<Controller>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_hook: Option<TransferHook>,
    /// Who can change the transfer fee; set when the extension is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_fee_authority: Option<Controller>,
    /// Who can pause all transfers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_authority: Option<Controller>,
    /// New token accounts start frozen (an allowlist).
    #[serde(default)]
    pub default_frozen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint_close_authority: Option<Controller>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_update_authority: Option<Controller>,
    /// Decoded extensions that carry no control power Celebtir rates
    /// (confidential transfers, interest-bearing, groups, …), by name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub other: Vec<String>,
    /// Extension type numbers Celebtir does not decode (TOK-4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransferHook {
    /// None when the extension is present but no hook program is set yet.
    pub program: Option<Address>,
    /// Who can point the hook at another program.
    pub authority: Controller,
    /// Upgrade control of the hook program itself.
    pub program_controller: Option<Controller>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TokenProvenance {
    Native,
    Bridged {
        bridge: String,
        backing_chain: String,
        backing_address: String,
    },
}

/// How a protocol uses a token (TOK-2).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TokenRole {
    Collateral,
    Borrowable,
    Deposit,
    Lp,
}

/// Version of the `run.json` format.
pub const RUN_SCHEMA_VERSION: &str = "0.1.0";

/// `data/snapshots/<date>/run.json`: how the run went. The only file with
/// wall-clock timestamps (SNAP-2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunFile {
    pub schema_version: String,
    /// Tool version and, when built in CI, the commit.
    pub tool_version: String,
    /// RFC 3339, UTC.
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    /// The RPC host only, never the URL (SNAP-5).
    pub rpc_host: String,
    /// Requests per RPC method, retries included (COL-5).
    pub calls: BTreeMap<String, u64>,
    pub retries: u64,
    pub budget: u64,
    pub budget_exceeded: bool,
    /// Protocols collected successfully.
    pub protocols: Vec<String>,
    pub failures: Vec<RunFailure>,
    pub review_needed: Vec<RunNote>,
    /// The Hyperliquid half of the run (wave 2, HOPS-1); None when it was
    /// not collected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperliquid: Option<RunChain>,
}

/// Calls and coverage for one extra chain in a run (HOPS-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunChain {
    /// The HyperEVM RPC host only, never the URL (SNAP-5).
    pub rpc_host: String,
    /// The pinned HyperEVM block; None when the RPC did not answer.
    pub block: Option<u64>,
    /// JSON-RPC requests per method, retries included.
    pub calls: BTreeMap<String, u64>,
    pub retries: u64,
    pub budget: u64,
    /// Info API requests (`perpDexs`, `delegatorSummary`).
    pub info_calls: u64,
    /// HyperEVM protocols collected.
    pub protocols: Vec<String>,
    /// HIP-3 DEXs listed; 0 when the info API failed.
    pub hip3_dexs: u64,
    /// Failures in this chain only; also listed in the run's `failures`.
    pub failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunFailure {
    /// A protocol ID, `token:<mint>` or `tokens`.
    pub scope: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunNote {
    pub protocol: String,
    pub note: String,
}

/// A number from an external source, or why it is missing. Missing numbers
/// show as "data unavailable" and never fail a run (VAL-7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Metric {
    /// Whole US dollars.
    Usd {
        value: u64,
    },
    Unavailable {
        reason: String,
    },
}

impl Metric {
    pub fn usd(&self) -> Option<u64> {
        match self {
            Metric::Usd { value } => Some(*value),
            Metric::Unavailable { .. } => None,
        }
    }
}

/// One protocol's context metrics for the day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtocolMetrics {
    pub defillama_slug: Option<String>,
    pub tvl: Metric,
    /// TVL split by asset group (VAL-3).
    #[serde(default)]
    pub asset_groups: AssetGroups,
    /// Daily fees, every category (VAL-6).
    #[serde(default)]
    pub fees: Activity,
    /// Daily volume, DEXs only; others show fees only (VAL-6).
    #[serde(default)]
    pub volume: Activity,
}

/// TVL by asset group in whole US dollars; the four groups sum to the TVL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetGroups {
    Usd {
        stablecoins: u64,
        sol: u64,
        btc: u64,
        other: u64,
    },
    Unavailable {
        reason: String,
    },
}

impl Default for AssetGroups {
    fn default() -> Self {
        AssetGroups::Unavailable {
            reason: "not collected".into(),
        }
    }
}

/// A daily USD series ending at the last complete day (VAL-6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Activity {
    Usd {
        /// Up to 30 complete days, oldest first.
        days: Vec<DailyUsd>,
        /// The newest complete day.
        latest: u64,
        /// The 7-day total against the previous 7 days, in basis points
        /// (1 % = 100). None when either window has a missing or
        /// implausible day (see `gap_days`) or the earlier total is zero.
        change_7d_bp: Option<i64>,
        /// Days in the two comparison windows that are absent or below 5 %
        /// of the series median, treated as missing (methodology v0.3, D5).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        gap_days: Vec<String>,
    },
    Unavailable {
        reason: String,
    },
}

impl Default for Activity {
    fn default() -> Self {
        Activity::Unavailable {
            reason: "not collected".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DailyUsd {
    /// `YYYY-MM-DD`, UTC.
    pub date: String,
    pub usd: u64,
}

/// One change to a program's ProgramData account (VAL-4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpgradeEvent {
    pub signature: String,
    pub slot: u64,
    /// `YYYY-MM-DD` from the block time; None when the chain has no block
    /// time for the slot (show the slot only, never a guessed date).
    pub date: Option<String>,
    pub kind: UpgradeKind,
    /// For `set_authority`: the new upgrade authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_authority: Option<Address>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeKind {
    Deploy,
    Upgrade,
    SetAuthority,
    /// The authority was removed: the program became immutable.
    RemoveAuthority,
    Extend,
    Close,
}

/// A program's upgrade history as far as it has been read. Busy ProgramData
/// accounts appear in many unrelated transactions, so the history fills over
/// several runs within the daily call budget.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgramHistory {
    /// Oldest first.
    pub events: Vec<UpgradeEvent>,
    /// Transactions on the account examined so far.
    pub scanned: u64,
    /// Transactions on the account found so far.
    pub total: u64,
    /// True once every transaction has been examined.
    pub complete: bool,
}

/// A token's price, in millionths of a US dollar so it stays an exact integer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TokenMetrics {
    Price { usd_micros: u64 },
    Unavailable { reason: String },
}
