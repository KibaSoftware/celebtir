// Summary-page figures: KPI values, distributions and ecosystem insights.
// Pure functions over the published snapshot, so every figure on the page is
// traceable to a field and tested without a browser. Nothing is estimated:
// a change shows only when history covers its period, and every insight is a
// rule over published ratings, worded as a configuration fact (LCH-3).

import { assessedRows, type ProtocolRow } from "./summary";
import { changeDate, recentOnChain } from "./changes";
import type { Rating } from "./tokens";
import { CONTROL_CLASSES, FACTORS, type ChangeEvent, type ControlClass, type Factor, type ScoresFile, type Stage, type SummaryDay } from "./types";

const DAY = 86_400_000;
export const RATINGS: Rating[] = ["low", "medium", "high", "unknown", "not_applicable"];
export const STAGES: Stage[] = ["stage_2", "stage_1", "stage_0", "unrated"];

/** Control classes without a delay of the protective minimum (24 h+, D4). */
export const UNDELAYED: ControlClass[] = ["single_key", "unrecognized", "multisig_without_delay", "multisig_short_delay"];

export interface Delta {
  fraction: number;
  /** The date compared against. */
  since: string;
}

/** Change in total rated TVL over `days`, or null when history does not reach back that far. */
export function tvlDelta(history: SummaryDay[], days = 7): Delta | null {
  const last = history[history.length - 1];
  if (!last) return null;
  const cutoff = Date.parse(last.date) - days * DAY;
  const before = [...history].reverse().find((d) => Date.parse(d.date) <= cutoff);
  if (!before || !before.total_tvl_usd) return null;
  return { fraction: (last.total_tvl_usd - before.total_tvl_usd) / before.total_tvl_usd, since: before.date };
}

export interface Kpis {
  totalTvl: number;
  /** Protocols left out of the TVL total because DefiLlama reported none. */
  tvlExcluded: number;
  tvlDelta: Delta | null;
  assessed: number;
  /** Protocols listed with facts but not rated (F5). */
  unassessed: number;
  byCategory: Record<string, number>;
  undelayedTvl: number;
  undelayedShare: number | null;
  highUpgradeControl: number;
  recentChanges: number;
  /** The window for `recentChanges`, ending on the snapshot date. */
  changeWindowDays: number;
}

export function kpis(
  rows: ProtocolRow[],
  value: ScoresFile["value_by_control"],
  history: SummaryDay[],
  events: ChangeEvent[],
  snapshotDate: string,
  changeWindowDays = 30,
): Kpis {
  const total = value.all.total;
  const undelayed = UNDELAYED.reduce((n, c) => n + (value.all.classes[c] ?? 0), 0);
  const rated = assessedRows(rows);
  const byCategory: Record<string, number> = {};
  for (const r of rated) byCategory[r.category] = (byCategory[r.category] ?? 0) + 1;
  return {
    totalTvl: total,
    tvlExcluded: value.excluded.length,
    tvlDelta: tvlDelta(history),
    assessed: rated.length,
    unassessed: rows.length - rated.length,
    byCategory,
    undelayedTvl: undelayed,
    undelayedShare: total > 0 ? undelayed / total : null,
    highUpgradeControl: rated.filter((r) => r.ratings.upgrade_control === "high").length,
    recentChanges: recentOnChain(events, snapshotDate, changeWindowDays).length,
    changeWindowDays,
  };
}

export { recentOnChain };

/** Assessed protocols per rating, for each factor. Unknown and Not applicable stay separate. */
export function factorDistribution(rows: ProtocolRow[]): Record<Factor, Record<Rating, number>> {
  const rated = assessedRows(rows);
  return Object.fromEntries(
    FACTORS.map((f) => [f, Object.fromEntries(RATINGS.map((r) => [r, rated.filter((row) => row.ratings[f] === r).length]))]),
  ) as Record<Factor, Record<Rating, number>>;
}

/** Assessed protocols per Stage; unassessed protocols have none (F5). */
export function stageDistribution(rows: ProtocolRow[]): Record<Stage, number> {
  const rated = assessedRows(rows);
  return Object.fromEntries(STAGES.map((s) => [s, rated.filter((r) => r.stage === s).length])) as Record<Stage, number>;
}

/** TVL per control class in the published order, with zero for empty classes. */
export function controlShares(value: ScoresFile["value_by_control"]["all"]): { cls: ControlClass; usd: number }[] {
  return CONTROL_CLASSES.map((cls) => ({ cls, usd: value.classes[cls] ?? 0 }));
}

export interface InsightItem {
  id: string;
  name: string;
  detail: string;
  href: string;
}

export interface Insight {
  key: string;
  title: string;
  /** The rule, stated so a reader can check it. */
  rule: string;
  items: InsightItem[];
}

/**
 * Observations derived only from published ratings and events. Each insight
 * states its rule; an insight with no matching protocol is still listed, with
 * none, so absence is visible too.
 */
export function insights(rows: ProtocolRow[], events: ChangeEvent[], snapshotDate: string, names: Record<string, string>): Insight[] {
  // Rating rules run over assessed protocols only (F5).
  const byName = assessedRows(rows).sort((a, b) => a.name.localeCompare(b.name));
  const item = (r: (typeof byName)[number], f: Factor, detail?: string): InsightItem => ({
    id: r.id,
    name: r.name,
    detail: detail ?? r.reasons[f],
    href: `/protocols/${r.id}/#factor-${f}`,
  });
  return [
    {
      key: "delay",
      title: "Upgrades without a protective delay",
      rule: "Upgrade delay rated High: under the methodology's Medium minimum, or none.",
      items: byName.filter((r) => r.ratings.upgrade_delay === "high").map((r) => item(r, "upgrade_delay")),
    },
    {
      key: "controller",
      title: "Concentrated or unrecognized upgrade control",
      rule: "Core programs upgradeable by a single key or by a controller Celebtir cannot identify.",
      items: byName
        .filter((r) => r.controlClass === "single_key" || r.controlClass === "unrecognized")
        .map((r) => item(r, "upgrade_control", r.controller)),
    },
    {
      key: "dependencies",
      title: "Dependencies rated High or Unknown",
      rule: "A called program rated High on upgrade control, or a dependency that could not be read.",
      items: byName.filter((r) => r.ratings.dependencies === "high" || r.ratings.dependencies === "unknown").map((r) => item(r, "dependencies")),
    },
    {
      key: "changes",
      title: "Recent on-chain control changes",
      rule: "Changes detected on chain in the 30 days before this snapshot.",
      items: recentOnChain(events, snapshotDate).slice(0, 5).map((e) => ({
        id: e.protocol!,
        name: names[e.protocol!] ?? e.protocol!,
        detail: `${changeDate(e)}: ${e.summary}`,
        href: `/changelog/?protocol=${encodeURIComponent(e.protocol!)}`,
      })),
    },
  ];
}
