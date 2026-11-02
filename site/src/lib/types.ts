// The JSON the pipeline publishes (schemas/*.schema.json), typed for the
// fields the site reads. Rust is the source of truth; these follow it.

import type { Rating } from "./tokens";

export type Stage = "unrated" | "stage_0" | "stage_1" | "stage_2";
export type Factor =
  | "upgrade_control"
  | "upgrade_delay"
  | "admin_powers"
  | "code_transparency"
  | "oracle_dependency"
  | "dependencies";

export const FACTORS: readonly Factor[] = [
  "upgrade_control",
  "upgrade_delay",
  "admin_powers",
  "code_transparency",
  "oracle_dependency",
  "dependencies",
];

export interface Evidence {
  chain: string;
  address: string;
  slot: number;
  field: string;
  explorer_url: string;
}

export interface FactorScore {
  rating: Rating;
  reason: string;
  next_step: string;
  evidence: Evidence[];
  /** Registry sources behind a reviewed-none finding (methodology v0.3). */
  registry_sources?: string[];
}

export interface Requirement {
  id: string;
  stage: Stage;
  description: string;
  met: boolean;
  detail: string;
}

export type ControlClass =
  | "single_key"
  | "unrecognized"
  | "multisig_without_delay"
  | "multisig_short_delay"
  | "multisig_with_delay"
  | "immutable";

/** Weakest first. `multisig_short_delay` exists from methodology v0.3 (D4). */
export const CONTROL_CLASSES: readonly ControlClass[] = [
  "single_key",
  "unrecognized",
  "multisig_without_delay",
  "multisig_short_delay",
  "multisig_with_delay",
  "immutable",
];

export interface ProtocolScore {
  control_class: ControlClass;
  controller_summary: string;
  stage: { stage: Stage; next: Stage | null; missing: Requirement[]; requirements: Requirement[] };
  risks: { severity: "high" | "medium"; category: string; factor: Factor; text: string }[];
  factors: Record<Factor, FactorScore>;
  programs: Record<string, { role: "core" | "periphery"; upgrade_control: FactorScore; upgrade_delay: FactorScore; code_transparency: FactorScore }>;
  tokens: Record<string, { symbol: string; roles: string[]; worst: Rating; warning: string | null }>;
  /** Each admin field's own rating, keyed like the facts' `admin` map. */
  admin: Record<string, NodeRating>;
  /** Each sampled callee's upgrade-control rating. */
  callees: Record<string, NodeRating>;
}

export interface NodeRating {
  rating: Rating;
  reason: string;
}

export interface ValueRow {
  classes: Record<string, number>;
  total: number;
}

export interface ScoresFile {
  schema_version: string;
  methodology_version: string;
  /** Assessed protocols only: the loader withholds the rest (F5). */
  protocols: Record<string, ProtocolScore>;
  /** Protocols listed without ratings (methodology v0.3 coverage). */
  unassessed?: Record<string, { rated: false; reason: string }>;
  tokens: Record<string, TokenScore>;
  value_by_control: { all: ValueRow; by_category: Record<string, ValueRow>; excluded: string[] };
}

export type Metric = { kind: "usd"; value: number } | { kind: "unavailable"; reason: string };
export type Activity =
  | { kind: "usd"; days: { date: string; usd: number }[]; latest: number; change_7d_bp: number | null; gap_days?: string[] }
  | { kind: "unavailable"; reason: string };

export interface ProtocolMetrics {
  defillama_slug: string | null;
  tvl: Metric;
  asset_groups: { kind: "usd"; stablecoins: number; sol: number; btc: number; other: number } | { kind: "unavailable"; reason: string };
  fees: Activity;
  volume: Activity;
}

export type Controller =
  | { kind: "program_controlled"; key: string; program: string; upgrade: Controller }
  | { kind: "immutable" }
  | { kind: "single_key"; key: string }
  | {
      kind: "multisig";
      multisig_kind: "squads_v4" | "squads_v3" | "safe";
      address: string;
      threshold: number;
      voters: number;
      time_lock_secs: number;
      config_authority: Controller | null;
      /** Voting members' keys, sorted; absent in older snapshots. */
      signers?: string[];
      /** Signers that are themselves multisigs or unknown accounts, resolved. */
      nested_signers?: Record<string, Controller>;
    }
  | {
      kind: "timelock";
      address: string;
      min_delay_secs: number;
      proposer: Controller;
      /** An outside DEFAULT_ADMIN_ROLE holder: acts without the delay (ADR 024 D4). */
      admin?: Controller | null;
    }
  | { kind: "cannot_act"; address: string; threshold: number; voters: number }
  | { kind: "unrecognized"; key: string; owner_program: string | null }
  /** Off-curve: no key exists; the controlling program is not resolved (F2). */
  | { kind: "pda"; key: string; owner_program: string | null }
  | { kind: "unavailable"; reason: string };

export interface Fact<T> {
  value: T;
  evidence: Evidence[];
}

export type CodeTransparency =
  | { kind: "verified"; repo: string; commit: string; on_chain_hash: string; executable_hash: string; warning?: string }
  | { kind: "public_source_unverified"; repo: string; commit?: string; on_chain_hash?: string }
  | { kind: "no_public_source" }
  | { kind: "unavailable"; reason: string };

export interface ProgramFacts {
  role: "core" | "periphery";
  loader: string | null;
  upgrade_control: Fact<Controller>;
  code_transparency: Fact<CodeTransparency>;
}

export type AdminPower = "parameters" | "pause" | "funds";

export interface AdminFact {
  account: string;
  field: string;
  power: AdminPower;
  controller: Fact<Controller>;
}

export type OracleSource =
  | { kind: "network"; name: string }
  | { kind: "protocol_run" }
  | { kind: "other"; owner_program: string }
  | { kind: "unavailable"; reason: string };

/** The feeds a protocol-run aggregator combines for one market (edge case 11). */
export type UnderlyingFeeds =
  | { kind: "listed"; feeds: { source_type: string; address: string }[] }
  | { kind: "unavailable"; reason: string };

export interface OracleFact {
  market: string;
  role: "primary" | "fallback";
  /** The protocol rejects the primary when it strays from this feed (D3). */
  bounds_primary?: boolean;
  /** Networks the publisher documents as bounding this primary. */
  guarded_by?: string[];
  /** Who publishes the price, when the owner program does not say. */
  provider?: string;
  source: Fact<OracleSource>;
  underlying?: Fact<UnderlyingFeeds>;
  /** The same per further market reading this account (Kamino reserves). */
  underlying_markets?: Record<string, Fact<UnderlyingFeeds>>;
}

/** A registry statement that a protocol has none of something (F3). */
export interface ReviewedNone {
  source: string;
  reviewed_on: string;
  note: string;
}

export type Dependencies =
  | { kind: "sampled"; callees: Record<string, { upgrade_control: Fact<Controller> }>; stale?: boolean }
  | { kind: "unavailable"; reason: string };

export interface RegistrySource {
  kind: "admin" | "multisig" | "oracle" | "program";
  address: string;
  label: string;
  url: string;
}

export interface ProtocolFacts {
  chain: string;
  category: string | null;
  last_reviewed: string | null;
  name: string | null;
  website: string | null;
  reviewed_by: string | null;
  sources: RegistrySource[];
  programs: Record<string, ProgramFacts>;
  admin: Record<string, AdminFact>;
  oracles: Record<string, OracleFact>;
  admin_review?: ReviewedNone;
  oracle_review?: ReviewedNone;
  dependencies: Fact<Dependencies>;
  tokens: Record<string, string[]>;
  min_slot: number;
  max_slot: number;
}

export type TokenFactor = "supply_control" | "freeze_and_seize" | "transfer_restrictions" | "provenance";

export const TOKEN_FACTORS: readonly TokenFactor[] = ["supply_control", "freeze_and_seize", "transfer_restrictions", "provenance"];

export interface TokenScore {
  symbol: string;
  factors: Record<TokenFactor, FactorScore>;
  /** "Your balance can be frozen if…", most severe first. */
  risks: { severity: "high" | "medium"; factor: TokenFactor; text: string }[];
}

export interface TokenExtensions {
  permanent_delegate?: Controller;
  transfer_hook?: { program: string | null; authority: Controller; program_controller: Controller | null };
  transfer_fee_authority?: Controller;
  pause_authority?: Controller;
  default_frozen: boolean;
  mint_close_authority?: Controller;
  metadata_update_authority?: Controller;
  other?: string[];
  unknown?: number[];
}

export type TokenProvenance =
  | { kind: "native" }
  | { kind: "bridged"; bridge: string; backing_chain: string; backing_address: string };

export interface TokenFacts {
  symbol: string;
  program: "spl_token" | "token2022";
  supply: number;
  decimals: number;
  mint_authority: Fact<Controller>;
  freeze_authority: Fact<Controller>;
  extensions: Fact<TokenExtensions>;
  provenance: Fact<TokenProvenance>;
  identity?: { kind: string; issuer: string; issuer_url: string; source: string };
}

export interface TokenHistoryFile {
  mint: string;
  symbol: string;
  days: { date: string; supply: number; decimals: number; price_usd_micros: number | null }[];
}

export interface ArchivedFile {
  protocols: {
    id: string;
    name: string;
    archived_on: string;
    reason: string;
    source: string;
    last_known: HistoryDay | null;
  }[];
}

export interface UpgradeEvent {
  signature: string;
  slot: number;
  date: string | null;
  kind: "deploy" | "upgrade" | "set_authority" | "remove_authority" | "extend" | "close";
  new_authority?: string;
}

export interface ProgramHistory {
  events: UpgradeEvent[];
  scanned: number;
  total: number;
  complete: boolean;
}

export interface FactsFile {
  schema_version: string;
  protocols: Record<string, ProtocolFacts>;
  tokens: Record<string, TokenFacts>;
  metrics: Record<string, ProtocolMetrics>;
  program_history: Record<string, ProgramHistory>;
  /** HIP-3 perp DEXs (wave 2); absent in Solana-only snapshots. */
  hip3?: Hip3Snapshot;
}

export type Hip3Stake =
  | {
      kind: "read";
      delegated: string;
      undelegated: string;
      pending_withdrawal: string;
      required: string;
      stake_below_required: boolean;
      unstaking: boolean;
    }
  | { kind: "unavailable"; reason: string };

export interface Hip3DexFacts {
  full_name: string;
  deployer: Fact<string>;
  oracle_updater: Fact<string | null>;
  sub_deployers: Fact<Record<string, string[]>>;
  fee_recipient: Fact<string | null>;
  oi_caps: Fact<Record<string, string>>;
  funding_multipliers: Fact<Record<string, string>>;
  stake: Fact<Hip3Stake>;
}

export type Hip3Snapshot =
  | { kind: "listed"; dexs: Record<string, Hip3DexFacts> }
  | { kind: "unavailable"; reason: string };

export interface RunFile {
  tool_version: string;
  started_at: string;
  finished_at: string;
  protocols: string[];
  failures: { scope: string; error: string }[];
}

export type Assessment = "assessed" | "unassessed";

/** `history/coverage.json`: which protocols are rated (F5). */
export interface CoverageFile {
  methodology_version: string;
  assessed_chains: string[];
  protocols: Record<string, { chain: string; assessment: Assessment; reason?: string }>;
}

export interface Snapshot {
  snapshotDate?: string;
  facts: FactsFile;
  /** Published scores: unassessed protocols are not in `protocols`. */
  scores: ScoresFile;
  run: RunFile;
  /** Every protocol in the facts, with its assessment. */
  coverage: CoverageFile;
  /** True when the site is built from the committed demo dataset. */
  demo: boolean;
}

export interface HistoryDay {
  date: string;
  tvl_usd: number | null;
  fees_usd: number | null;
  volume_usd: number | null;
  /** Absent before the coverage record existed (then assessed). */
  assessment?: Assessment;
  /** Null for an unassessed protocol. */
  stage: Stage | null;
  ratings: Record<Factor, Rating> | null;
  methodology_version: string;
}

export interface ProtocolHistoryFile {
  protocol: string;
  days: HistoryDay[];
  upgrades: {
    upgrades: number;
    last_upgrade_date: string | null;
    last_upgrade_slot: number | null;
    average_interval_days: number | null;
    upgrades_last_30_days: number;
    complete: boolean;
  };
  review: { last_reviewed: string | null; due: boolean; awaiting_human_review?: boolean };
}

export interface SummaryDay {
  date: string;
  /** Assessed protocols that day. */
  protocols: number;
  /** Protocols listed without ratings that day (F5). */
  unassessed?: number;
  total_tvl_usd: number;
  value_by_control: ValueRow;
  stages: Record<Stage, number>;
  methodology_version: string;
  /** Per chain filter (HUI-1); absent on days before wave 2 (Solana only).
   * An unassessed chain has no Stages or value classes. */
  by_chain?: Record<string, { protocols: number; total_tvl_usd: number; assessment?: Assessment; value_by_control: ValueRow | null; stages: Record<Stage, number> | null }>;
}

export interface ChangeEvent {
  before?: unknown;
  after?: unknown;
  corrects?: string;
  chain?: string;
  evidence?: Evidence[];
  id: string;
  date: string;
  detected_within_days: number;
  protocol: string | null;
  subject: string;
  kind: string;
  severity: "critical" | "warning" | "positive" | "info";
  source: "on_chain" | "registry_edit" | "methodology";
  summary: string;
  signature: string | null;
  /** Block date of an on-chain transaction; `date` is when it was recorded (F6). */
  occurred_on?: string;
  slot?: number;
  /** Recorded from history older than the comparison window. */
  backfill?: boolean;
}
