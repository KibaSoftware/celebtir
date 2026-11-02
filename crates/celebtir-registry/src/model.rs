//! The registry file format: `registry/solana/<protocol-id>.yaml` and
//! `registry/hyperliquid/<protocol-id>.yaml` (HyperEVM, wave 2).
//! The JSON Schema published for it is generated from these types (REG-1).

use std::collections::BTreeMap;

use celebtir_core::Address;
use celebtir_core::Chain;
pub use celebtir_core::snapshot::{AdminPower, OracleRole, ProgramRole, TokenRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtocolEntry {
    /// Verified declarations for token, admin or upgrade authority PDAs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authority_pdas: Vec<AuthorityPda>,
    /// Kebab-case, unique, never reused; matches the file name.
    #[schemars(regex(pattern = r"^[a-z0-9]+(-[a-z0-9]+)*$"))]
    pub id: String,
    pub name: String,
    pub chain: Chain,
    pub category: Category,
    pub website: Url,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defillama_slug: Option<String>,
    #[schemars(length(min = 1))]
    pub programs: Vec<ProgramEntry>,
    #[serde(default)]
    pub multisigs: Vec<MultisigEntry>,
    /// HyperEVM: Safe multisigs; members are read on chain (HREG-2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub safes: Vec<SafeEntry>,
    /// HyperEVM: OpenZeppelin `TimelockController`s (HREG-2, ADR 024 D4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timelocks: Vec<TimelockEntry>,
    #[serde(default)]
    pub admin_accounts: Vec<AdminAccountEntry>,
    /// SPL stake pools (and forks) whose manager, staker and deposit or
    /// withdraw authorities are read on chain as admin powers (F3, D2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stake_pools: Vec<StakePoolEntry>,
    /// A sourced review that found no admin accounts. Without it, a protocol
    /// with no admin entries rates Unknown, never Low (v0.3, F3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_reviewed_none: Option<ReviewedNone>,
    /// A sourced review that found no price feeds (v0.3). Not allowed for
    /// lending and perps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracles_reviewed_none: Option<ReviewedNone>,
    /// HyperEVM: who holds admin powers on a contract (HCOL-7).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evm_admins: Vec<EvmAdminEntry>,
    /// HyperEVM: contracts called with user funds, declared here because
    /// wave 2 does not trace transactions (HCOL-10).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<DependencyEntry>,
    #[serde(default)]
    pub oracles: Vec<OracleEntry>,
    /// Tokens the protocol accepts or issues (TOK-2); each must be in `tokens.yaml`.
    #[serde(default)]
    pub tokens: Vec<TokenUse>,
    pub review: Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Lending,
    Dex,
    Perps,
    LiquidStaking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgramEntry {
    pub id: Address,
    pub name: String,
    /// `core` holds or moves user funds; `periphery` is everything else.
    pub role: ProgramRole,
    /// Where this program ID was confirmed (REG-3).
    pub source: Url,
    /// Other protocols that list the same address (REG-4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with: Vec<String>,
    /// HyperEVM contracts only, and required there: how the code can change
    /// (HREG-1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyKind>,
}

/// How a HyperEVM contract's code can change (ADR 024 D3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyKind {
    /// Transparent proxy: the EIP-1967 admin slot holds the upgrader.
    Eip1967,
    /// UUPS: the admin slot is empty; the implementation's `owner()` upgrades.
    Uups,
    /// UUPS whose `_authorizeUpgrade` calls `roleRegistry().onlyProtocolUpgrader`
    /// (the ether.fi RoleRegistry pattern): the role registry's `owner()`
    /// upgrades. The program `source` must show the verified implementation.
    UupsRoleRegistry,
    /// Not a proxy.
    None,
}

/// An admin power on a HyperEVM contract. Without `role`, the holder is
/// read from `owner()`; with `role` (an AccessControl role name, or
/// `DEFAULT_ADMIN_ROLE`), `holder` is confirmed with `hasRole`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvmAdminEntry {
    pub contract: Address,
    pub power: AdminPower,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<Address>,
    pub source: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DependencyEntry {
    pub address: Address,
    pub name: String,
    pub proxy: ProxyKind,
    pub source: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SafeEntry {
    pub address: Address,
    pub source: Url,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimelockEntry {
    pub address: Address,
    /// Confirmed on chain with `hasRole(PROPOSER_ROLE, …)`.
    #[schemars(length(min = 1))]
    pub proposers: Vec<Address>,
    pub source: Url,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MultisigEntry {
    pub address: Address,
    pub program: MultisigProgram,
    /// Squads v4: the vault index. Squads v3: the authority index (the
    /// default vault is 1).
    pub vault_index: u8,
    pub source: Url,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MultisigProgram {
    SquadsV4,
    /// squads-mpl (`SMPLecH…`): no time lock; config changes need a vote.
    SquadsV3,
}

/// A registry statement that something is absent, with its evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewedNone {
    /// The public source (program source, documentation) the review used.
    pub source: Url,
    /// `YYYY-MM-DD`.
    #[schemars(regex(pattern = r"^\d{4}-\d{2}-\d{2}$"))]
    pub reviewed_on: String,
    /// What was checked, in one sentence.
    #[schemars(length(min = 1))]
    pub note: String,
}

/// An SPL stake pool account (Solana Program Library layout, also used by
/// Sanctum's forks), read on chain for its admin authorities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StakePoolEntry {
    pub account: Address,
    /// The pool token mint the account must name; checked when decoding.
    pub pool_mint: Address,
    pub source: Url,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdminAccountEntry {
    pub account: Address,
    /// Path of the vendored IDL, relative to `registry/` (`idl/<file>.json`).
    pub idl: String,
    /// The IDL field holding the admin key.
    pub field: String,
    pub power: AdminPower,
    pub source: Url,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_with: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OracleEntry {
    pub account: Address,
    pub market: String,
    pub role: OracleRole,
    /// The protocol rejects the market's primary price when it differs from
    /// this feed by more than a threshold (v0.3 guarded primary, D3). The
    /// `source` must show it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bounds_primary: bool,
    /// Primary only: networks the publisher documents as bounding this
    /// price; each must also be registered with `bounds_primary` (D3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guarded_by: Vec<String>,
    /// Who publishes the price, when the owner program alone does not say.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// For pull reports that are not accounts (Chainlink Data Streams feed
    /// IDs): the on-chain program that verifies each report. `account` is
    /// then the feed ID, and the feed is classified by this program.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<Address>,
    pub source: Url,
    /// Where the market records which Scope entries it reads, so the
    /// collector can list the feeds behind a Scope price (edge case 11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_chain: Option<ScopeChain>,
}

/// The account and IDL fields that name a market's Scope price account and
/// its chain of Scope entries (Kamino: the reserve's `scope_configuration`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeChain {
    pub account: Address,
    /// Path of the vendored IDL for `account`, relative to `registry/`.
    pub idl: String,
    /// The field holding the Scope price account; it must be this oracle.
    pub feed_field: String,
    /// The field holding the `[u16; N]` chain of Scope entries.
    pub chain_field: String,
    /// Further markets (Kamino reserves) that read the same Scope price
    /// account, each with its own chain in the same fields (F4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserves: Vec<ScopeReserve>,
    /// IDL fields of direct Pyth or Switchboard overrides on each reserve,
    /// read alongside the Scope chain; unset fields are ignored.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub direct_feed_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeReserve {
    /// `"<symbol> reserve"`.
    pub market: String,
    pub account: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub reviewed_by: String,
    /// `YYYY-MM-DD`.
    #[schemars(regex(pattern = r"^\d{4}-\d{2}-\d{2}$"))]
    pub last_reviewed: String,
}

/// `registry/idl/origins.yaml`: where each vendored IDL came from (REG-8).
/// Keyed by file name inside `registry/idl/`.
pub type IdlOrigins = BTreeMap<String, IdlOrigin>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdlOrigin {
    pub source: Url,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<u64>,
}

/// A token a protocol uses, and how (TOK-2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenUse {
    pub mint: Address,
    #[schemars(length(min = 1))]
    pub roles: Vec<TokenRole>,
}

/// `registry/archived.yaml` (ADR 017): protocols that shut down or left
/// coverage. Optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArchivedFile {
    pub archived: Vec<ArchivedEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArchivedEntry {
    /// The protocol's former registry ID; never reused.
    #[schemars(regex(pattern = r"^[a-z0-9]+(-[a-z0-9]+)*$"))]
    pub id: String,
    pub name: String,
    /// `YYYY-MM-DD`.
    #[schemars(regex(pattern = r"^\d{4}-\d{2}-\d{2}$"))]
    pub archived_on: String,
    /// Why it left coverage, as a configuration fact.
    #[schemars(length(min = 1))]
    pub reason: String,
    /// Public page that confirms it.
    pub source: Url,
}

/// `registry/tokens.yaml` (TOK-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokensFile {
    pub tokens: Vec<TokenEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenEntry {
    /// Verified declarations for token, admin or upgrade authority PDAs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authority_pdas: Vec<AuthorityPda>,
    pub mint: Address,
    pub symbol: String,
    #[serde(rename = "type")]
    pub kind: TokenKind,
    /// Confirms the mint, symbol and type.
    pub source: Url,
    pub issuer: Issuer,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TokenKind {
    Stablecoin,
    LiquidStaking,
    Bridged,
    Governance,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Issuer {
    pub name: String,
    pub source: Url,
}

/// Native to Solana, or bridged from another chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Provenance {
    Native {
        source: Url,
    },
    Bridged {
        bridge: String,
        backing_chain: String,
        /// The backing token's address on `backing_chain`.
        backing_address: String,
        source: Url,
    },
}

impl Category {
    /// The kebab-case name used in registry files.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Lending => "lending",
            Category::Dex => "dex",
            Category::Perps => "perps",
            Category::LiquidStaking => "liquid-staking",
        }
    }
}

/// The EVM sections of `controllers.yaml` and `oracles.yaml` (HREG-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EvmControllerKind {
    /// A Safe singleton (master copy) that proxies delegate to.
    SafeSingleton,
    /// A `TimelockController` implementation, matched by code hash or address.
    Timelock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvmController {
    pub address: Address,
    pub kind: EvmControllerKind,
    pub name: String,
    pub source: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvmOracle {
    pub address: Address,
    /// `network` or `protocol-run`, as for Solana oracle programs.
    pub class: String,
    pub name: String,
    pub source: Url,
}

/// `registry/hyperliquid/hip3.yaml` (HREG-4, HIP-4): the required deployer
/// stake with its source, and optional display data per DEX name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hip3File {
    /// HYPE, as a decimal string.
    pub required_stake: String,
    pub required_stake_source: Url,
    #[serde(default)]
    pub dexs: BTreeMap<String, Hip3DexEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hip3DexEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<Url>,
    /// Published contract spec and oracle methodology.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_url: Option<Url>,
}

/// `registry/programs.yaml`: source-code facts keyed by program ID (F1), so
/// a program shared by several protocols carries one fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgramsFile {
    pub programs: Vec<ProgramSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgramSource {
    pub id: Address,
    pub name: String,
    pub source_code: SourceCode,
}

/// Where a program's source is published, or a sourced finding that it is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceCode {
    Public {
        /// A repository URL: `https://github.com/<owner>/<repo>` (or GitLab).
        repo: Url,
        /// The full 40-character commit the program ID is declared at.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        commit: Option<String>,
        /// Path inside the repository, when the repo holds several programs.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        /// The page that ties this program ID to that source (a
        /// `declare_id!`, the publisher's program list).
        evidence: Url,
    },
    NoneFound {
        /// Where the search ended: the publisher's program list.
        evidence: Url,
    },
}

/// Every seed has an explicit encoding; address strings are decoded to 32 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum PdaSeed {
    Literal(String),
    Address(Address),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPda {
    pub key: Address,
    pub program: Address,
    pub seeds: Vec<PdaSeed>,
    pub source: Url,
}
