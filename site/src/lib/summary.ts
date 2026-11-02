// Rows and series for the Summary page, built at build time from the
// published files. Pure functions: tested without a browser.

import type { Rating } from "./tokens";
import { stageInspection, type StageInspection } from './scoreReview';
import {
  FACTORS,
  type ChangeEvent,
  type ControlClass,
  type Factor,
  type ProtocolHistoryFile,
  type Snapshot,
  type Stage,
} from "./types";
import { RANGE_DAYS, RANGES, TABS, type Range, type Tab, type ViewState } from "./viewState";
import { isAssessed } from "./assessment";
import { describeController } from "./controller";
import { changeDate, latestOnChain } from "./changes";
import type { Assessment } from "./types";
import { weekOverWeek } from "./activity";

export interface ProtocolRow {
  inspection?: StageInspection;
  /** Position by TVL among all rows, 1 = largest; unknown TVL ranks last. */
  rank: number;
  id: string;
  /** Which chain filter the protocol belongs to (HUI-1). */
  chain: "solana" | "hyperliquid";
  name: string;
  category: string;
  website: string | null;
  /** `unassessed`: listed with facts, never with a Stage or ratings (F5). */
  assessment: Assessment;
  /** Null when unassessed. */
  stage: Stage | null;
  nextStage: Stage | null;
  /** Descriptions of what the next Stage still needs. */
  missing: string[];
  /** Null when unassessed. */
  ratings: Record<Factor, Rating> | null;
  reasons: Record<Factor, string> | null;
  controller: string;
  /** Null when unassessed. */
  controlClass: ControlClass | null;
  tvl: number | null;
  /** Fractional change over 7 days, e.g. 0.05 for +5 %. */
  tvlChange7d: number | null;
  /** The latest complete day (its date) and the 7-day total against the
   * previous 7 days; the change is null over a gap (F8). */
  activity: { kind: "volume" | "fees"; usd: number; date: string | null; changeBp: number | null } | null;
  lastChange: { date: string; summary: string } | null;
}

export const CATEGORY_LABELS: Record<Tab, { label: string; definition: string }> = {
  all: { label: "All", definition: "Every rated protocol." },
  lending: { label: "Lending", definition: "Deposit assets to earn interest or borrow against collateral." },
  dex: { label: "Spot DEX", definition: "Swap tokens through pools or order books." },
  perps: { label: "Perps", definition: "Trade perpetual futures with leverage." },
  "liquid-staking": { label: "Liquid staking", definition: "Stake SOL and receive a tradable staking token." },
};

function tvlChange(history: ProtocolHistoryFile | undefined): number | null {
  const days = history?.days.filter((d) => d.tvl_usd !== null) ?? [];
  const last = days[days.length - 1];
  if (!last) return null;
  const cutoff = Date.parse(last.date) - 7 * 86_400_000;
  const before = [...days].reverse().find((d) => Date.parse(d.date) <= cutoff);
  if (!before || !before.tvl_usd) return null;
  return (last.tvl_usd! - before.tvl_usd) / before.tvl_usd;
}

export function buildRows(
  s: Snapshot,
  histories: Record<string, ProtocolHistoryFile | undefined>,
  events: ChangeEvent[],
): ProtocolRow[] {
  const rows = Object.entries(s.facts.protocols).map(([id, facts]) => {
    const score = isAssessed(s, id) ? s.scores.protocols[id] : undefined;
    const m = s.facts.metrics[id];
    const activity =
      m?.volume.kind === "usd"
        ? { kind: "volume" as const, usd: m.volume.latest, date: m.volume.days.at(-1)?.date ?? null, changeBp: weekOverWeek(m.volume) }
        : m?.fees.kind === "usd"
          ? { kind: "fees" as const, usd: m.fees.latest, date: m.fees.days.at(-1)?.date ?? null, changeBp: weekOverWeek(m.fees) }
          : null;
    const change = latestOnChain(events, id);
    return {
      rank: 0,
      id,
      chain: chainOf(facts?.chain),
      name: facts?.name ?? id,
      category: facts?.category ?? "uncategorized",
      website: facts?.website ?? null,
      assessment: (score ? "assessed" : "unassessed") as Assessment,
      stage: score?.stage.stage ?? null,
      inspection: score && facts.chain === 'solana' ? stageInspection(id, facts, score, s.scores.methodology_version, s.snapshotDate ?? s.run.finished_at.slice(0,10)) : undefined,
      nextStage: score?.stage.next ?? null,
      missing: score?.stage.missing.map((r) => r.description) ?? [],
      ratings: score ? (Object.fromEntries(FACTORS.map((f) => [f, score.factors[f].rating])) as Record<Factor, Rating>) : null,
      reasons: score ? (Object.fromEntries(FACTORS.map((f) => [f, score.factors[f].reason])) as Record<Factor, string>) : null,
      controller: score?.controller_summary ?? factualController(facts),
      controlClass: score?.control_class ?? null,
      tvl: m?.tvl.kind === "usd" ? m.tvl.value : null,
      tvlChange7d: tvlChange(histories[id]),
      activity,
      lastChange: change ? { date: changeDate(change), summary: change.summary } : null,
    };
  });
  return withRanks(rows);
}

/** The weakest-looking core controller in words, for unassessed rows: a fact, not a rating. */
function factualController(p: Snapshot["facts"]["protocols"][string]): string {
  const core = Object.values(p.programs).filter(f => f.role === "core").map(f => f.upgrade_control.value);
  return core.length ? core.map(describeController).sort()[0]! : "No core programs";
}

/** A row that carries ratings (assessed). */
export type RatedRow = ProtocolRow & { stage: Stage; ratings: Record<Factor, Rating>; reasons: Record<Factor, string>; controlClass: ControlClass };

/** Rows that carry ratings (assessed), for every rating aggregate. */
export function assessedRows(rows: ProtocolRow[]): RatedRow[] {
  return rows.filter((r): r is RatedRow => r.assessment === "assessed" && r.stage !== null && r.ratings !== null && r.reasons !== null && r.controlClass !== null);
}

/** Ranks rows by TVL, largest first; ties and unknown TVL fall back to name. */
export function withRanks(rows: ProtocolRow[]): ProtocolRow[] {
  const order = [...rows].sort((a, b) => {
    if (a.tvl === null || b.tvl === null) return a.tvl === b.tvl ? a.name.localeCompare(b.name) : a.tvl === null ? 1 : -1;
    return b.tvl - a.tvl || a.name.localeCompare(b.name);
  });
  const rank = new Map(order.map((r, i) => [r.id, i + 1]));
  return rows.map((r) => ({ ...r, rank: rank.get(r.id)! }));
}

/** Sum of the six ratings by risk, for sorting the rosette column. */
export function riskScore(r: ProtocolRow): number | null {
  const ratings = r.ratings;
  if (!ratings) return null;
  return FACTORS.reduce((sum, f) => sum + Math.max(0, RISK[ratings[f]]), 0);
}

export function categoryCounts(rows: ProtocolRow[]): Record<Tab, number> {
  const counts = Object.fromEntries(TABS.map((t) => [t, 0])) as Record<Tab, number>;
  for (const r of rows) {
    counts.all += 1;
    if ((TABS as readonly string[]).includes(r.category)) counts[r.category as Tab] += 1;
  }
  return counts;
}

const RISK: Record<Rating, number> = { low: 0, medium: 1, unknown: 2, high: 3, not_applicable: -1 };
const CONTROL_ORDER: ControlClass[] = ["single_key", "unrecognized", "multisig_without_delay", "multisig_short_delay", "multisig_with_delay", "immutable"];
const STAGE: Record<Stage, number> = { unrated: -1, stage_0: 0, stage_1: 1, stage_2: 2 };

function sortValue(r: ProtocolRow, key: string): number | string | null {
  if ((FACTORS as readonly string[]).includes(key)) return r.ratings ? RISK[r.ratings[key as Factor]] : null;
  switch (key) {
    case "rank":
      return r.rank;
    case "risk":
      return riskScore(r);
    case "name":
      return r.name.toLowerCase();
    case "category":
      return r.category;
    case "stage":
      return r.stage === null ? null : STAGE[r.stage];
    case "controller":
      return r.controlClass === null ? null : CONTROL_ORDER.indexOf(r.controlClass);
    case "activity":
      return r.activity?.usd ?? null;
    case "last_change":
      return r.lastChange?.date ?? null;
    default:
      return r.tvl;
  }
}

/** `hyper_evm` and `hyper_core` are both Hyperliquid (HUI-1). */
export function chainOf(chain: string | undefined): ProtocolRow["chain"] {
  return chain === "hyper_evm" || chain === "hyper_core" ? "hyperliquid" : "solana";
}

/** Rows on the chosen chain (HUI-1). */
export function onChain<R extends ProtocolRow>(rows: R[], chain: ViewState["chain"]): R[] {
  return chain === "all" ? rows : rows.filter((r) => r.chain === chain);
}

/** Tab, filters, search and sort from the URL view state (UI-1, UI-9, UI-11). */
export function applyView<R extends ProtocolRow>(rows: R[], v: ViewState): R[] {
  const q = v.q.trim().toLowerCase();
  const kept = onChain(rows, v.chain).filter((r) => {
    if (v.tab !== "all" && r.category !== v.tab) return false;
    // Rating filters only ever match assessed rows (F5).
    if (v.filters.stage.length && (r.stage === null || !v.filters.stage.includes(r.stage))) return false;
    if (v.filters.controller.length && (r.controlClass === null || !v.filters.controller.includes(r.controlClass))) return false;
    for (const [f, allowed] of Object.entries(v.filters.rating)) {
      if (allowed?.length && (!r.ratings || !allowed.includes(r.ratings[f as Factor]))) return false;
    }
    return !q || r.name.toLowerCase().includes(q) || r.id.includes(q);
  });
  const dir = v.sort.dir === "asc" ? 1 : -1;
  return kept.sort((a, b) => {
    const x = sortValue(a, v.sort.key);
    const y = sortValue(b, v.sort.key);
    // Missing values sort last in both directions.
    if (x === null && y === null) return a.name.localeCompare(b.name);
    if (x === null) return 1;
    if (y === null) return -1;
    if (x < y) return -dir;
    if (x > y) return dir;
    return a.name.localeCompare(b.name);
  });
}

export function rangeDays<T>(days: T[], range: Range): T[] {
  const n = RANGE_DAYS[range];
  return n === null ? days : days.slice(-n);
}

/**
 * Ranges the history can fill: 7 days and all time always, a longer range
 * only once there are at least that many daily snapshots (UI-8). A range the
 * data cannot fill would show the same days as a shorter one.
 */
export function availableRanges(count: number): Range[] {
  return RANGES.filter((r) => r === "7d" || r === "all" || count >= RANGE_DAYS[r]!);
}
