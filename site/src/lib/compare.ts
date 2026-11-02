// UI-16 to UI-18: the comparison, built from the published files.

import { formatChangeBp, formatUsd } from "./format";
import { FACTOR_LABELS, STAGE_LABELS } from "./labels";
import type { Rating } from "./tokens";
import { FACTORS, type Activity, type ControlClass, type Factor, type FactorScore, type ProtocolHistoryFile, type Snapshot, type Stage } from "./types";
import { chainOfProtocol, type Chain } from "./scope";
import { stageInspection, type StageInspection } from './scoreReview';
import { buildProtocolPage } from './protocol';
import { protocolFinding } from './presentation';

export interface CompareItem {
  /** Display-only summaries; the published factor records are retained separately. */
  findings?: Record<Factor, string>;
  inspection?: StageInspection;
  id: string;
  name: string;
  category?: string;
  /** Chain filter of the protocol, for the search (HUI-1); absent means Solana. */
  chain?: Chain;
  stage: Stage;
  nextStage: Stage | null;
  missing: string[];
  factors: Record<Factor, { rating: Rating; reason: string }>;
  evidenceFactors?: Record<Factor, FactorScore>;
  controller: string;
  controlClass: ControlClass;
  upgrades: { count: number; last: string | null; avgDays: number | null; complete: boolean } | null;
  tvl: number | null;
  volume: Activity;
  fees: Activity;
  tokens: { symbol: string; worst: Rating }[];
  tvlDays: { date: string; usd: number }[];
}

const NO_LISTING: Activity = { kind: "unavailable", reason: "no DefiLlama listing" };

export function compareItems(s: Snapshot, histories: Record<string, ProtocolHistoryFile | undefined>): Record<string, CompareItem> {
  return Object.fromEntries(
    Object.entries(s.scores.protocols).map(([id, p]) => {
      const facts = s.facts.protocols[id];
      const m = s.facts.metrics[id];
      const h = histories[id];
      const presentation = buildProtocolPage(s, id, h, []);
      const item: CompareItem = {
        id,
        name: facts?.name ?? id,
        category: facts?.category ?? undefined,
        stage: p.stage.stage,
        inspection: facts.chain === 'solana' ? stageInspection(id,facts,p,s.scores.methodology_version,s.snapshotDate ?? s.run.finished_at.slice(0,10)) : undefined,
        evidenceFactors: p.factors,
        findings: Object.fromEntries(FACTORS.map(f => [f, protocolFinding(presentation, f)])) as Record<Factor, string>,
        nextStage: p.stage.next,
        missing: p.stage.missing.map((r) => r.description),
        factors: Object.fromEntries(FACTORS.map((f) => [f, { rating: p.factors[f].rating, reason: p.factors[f].reason }])) as CompareItem["factors"],
        chain: chainOfProtocol(s.facts.protocols[id]?.chain),
        controller: p.controller_summary,
        controlClass: p.control_class,
        upgrades: h
          ? {
              count: h.upgrades.upgrades,
              last: h.upgrades.last_upgrade_date ?? (h.upgrades.last_upgrade_slot !== null ? `slot ${h.upgrades.last_upgrade_slot}` : null),
              avgDays: h.upgrades.average_interval_days,
              complete: h.upgrades.complete,
            }
          : null,
        tvl: m?.tvl.kind === "usd" ? m.tvl.value : null,
        volume: m?.volume ?? NO_LISTING,
        fees: m?.fees ?? NO_LISTING,
        tokens: Object.values(p.tokens)
          .map((t) => ({ symbol: t.symbol, worst: t.worst }))
          .sort((a, b) => a.symbol.localeCompare(b.symbol)),
        tvlDays: (h?.days ?? []).flatMap((d) => (d.tvl_usd === null ? [] : [{ date: d.date, usd: d.tvl_usd }])),
      };
      return [id, item];
    }),
  );
}

export interface Cell {
  text: string;
  rating?: Rating;
  stage?: Stage;
}

export interface CompareRow {
  group: string;
  label: string;
  cells: Cell[];
  /** Every protocol shows the same value (UI-17). */
  same: boolean;
}

const activity = (a: Activity): Cell =>
  a.kind === "unavailable"
    ? { text: "Unavailable" }
    : { text: `${formatUsd(a.latest)}${a.change_7d_bp !== null ? ` (${formatChangeBp(a.change_7d_bp)} 7d)` : ""}` };

export function compareRows(items: CompareItem[]): CompareRow[] {
  const row = (group: string, label: string, cell: (i: CompareItem) => Cell): CompareRow => {
    const cells = items.map(cell);
    const key = (c: Cell) => `${c.text}|${c.rating ?? ""}|${c.stage ?? ""}`;
    return { group, label, cells, same: cells.every((c) => key(c) === key(cells[0]!)) };
  };
  return [
    row("Stage", "Stage", (i) => ({ text: STAGE_LABELS[i.stage], stage: i.stage })),
    row("Stage", "Missing for the next Stage", (i) => ({
      text: i.stage === "unrated" ? "Insufficient evidence to assign a Stage" : i.nextStage === null ? "Highest Stage" : i.missing.length ? i.missing.join("; ") : "Nothing recorded",
    })),
    ...FACTORS.map((f) => row("Risk", FACTOR_LABELS[f], (i) => ({ text: i.factors[f].reason, rating: i.factors[f].rating }))),
    row("Control", "Upgrade controller", (i) => ({ text: i.controller })),
    row("Upgrades", "Upgrades", (i) => ({
      text: i.upgrades ? `${i.upgrades.count}${i.upgrades.complete ? "" : " so far (history still filling)"}` : "Not read",
    })),
    row("Upgrades", "Last upgrade", (i) => ({ text: i.upgrades?.last ?? (i.upgrades ? "None" : "Not read") })),
    row("Upgrades", "Average interval", (i) => ({
      text: i.upgrades?.avgDays !== null && i.upgrades?.avgDays !== undefined ? `${i.upgrades.avgDays} days` : "Not enough upgrades",
    })),
    row("Value", "TVL", (i) => ({ text: i.tvl === null ? "Unavailable" : formatUsd(i.tvl) })),
    row("Activity", "24h volume", (i) => activity(i.volume)),
    row("Activity", "24h fees", (i) => activity(i.fees)),
    row("Tokens", "Tokens used", (i) => ({
      text: i.tokens.length ? i.tokens.map((t) => `${t.symbol} (worst ${t.worst.replace("_", " ")})`).join(", ") : "None listed",
    })),
  ];
}

/**
 * Overlay line colours: the Highwatch series order (ADR 023): Celestial Blue,
 * Highwatch Gold, Moon Silver, pale blue. Blue and pale blue can meet when
 * lines cross, so each line also has its own dash pattern and a name label
 * (UI-18: names, not colour alone).
 */
export const OVERLAY = [
  { color: "#6599E8", dash: "" },
  { color: "#D4B46A", dash: "6 4" },
  { color: "#C5CFDD", dash: "2 3" },
  { color: "#94BAF3", dash: "8 3 2 3" },
] as const;

export function overlay(items: { id: string; name: string; days: { date: string; usd: number }[] }[]) {
  const dates = [...new Set(items.flatMap((i) => i.days.map((d) => d.date)))].sort();
  return {
    series: items.map((i, n) => ({ key: i.id, label: i.name, ...OVERLAY[n % OVERLAY.length]! })),
    points: dates.map((date) => ({
      date,
      values: Object.fromEntries(items.flatMap((i) => i.days.filter((d) => d.date === date).map((d) => [i.id, d.usd]))),
    })),
  };
}
