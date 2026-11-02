// Build-time models for the Risk, Value, Activity, Tokens and token pages.

import { change7d } from "./activity";
import { describeController } from "./controller";
import type { Rating } from "./tokens";
import {
  TOKEN_FACTORS,
  type Activity,
  type Controller,
  type Evidence,
  type Factor,
  type FactorScore,
  type Snapshot,
  type TokenFactor,
  type TokenFacts,
  type TokenHistoryFile,
  type TokenScore,
} from "./types";

export const TOKEN_FACTOR_LABELS: Record<TokenFactor, string> = {
  supply_control: "Supply control",
  freeze_and_seize: "Freeze and seize",
  transfer_restrictions: "Transfer restrictions",
  provenance: "Provenance",
};

export const TOKEN_KIND_LABELS: Record<string, string> = {
  stablecoin: "Stablecoin",
  "liquid-staking": "Liquid staking token",
  bridged: "Bridged asset",
  governance: "Governance",
  other: "Other",
};

const nameOf = (s: Snapshot, id: string) => s.facts.protocols[id]?.name ?? id;

/** Risk page sub-tab: one factor for every protocol. */
export function factorRows(s: Snapshot, f: Factor): { id: string; name: string; score: FactorScore }[] {
  return Object.entries(s.scores.protocols)
    .map(([id, p]) => ({ id, name: nameOf(s, id), score: p.factors[f] }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

/** Asset groups summed over protocols whose groups are known. */
export function assetTotals(s: Snapshot) {
  const t = { stablecoins: 0, sol: 0, btc: 0, other: 0, total: 0, missing: [] as string[] };
  for (const [id, m] of Object.entries(s.facts.metrics)) {
    const g = m.asset_groups;
    if (g.kind !== "usd") {
      t.missing.push(nameOf(s, id));
      continue;
    }
    t.stablecoins += g.stablecoins;
    t.sol += g.sol;
    t.btc += g.btc;
    t.other += g.other;
  }
  t.total = t.stablecoins + t.sol + t.btc + t.other;
  return t;
}

const NO_LISTING: Activity = { kind: "unavailable", reason: "DefiLlama has no listing for this protocol" };

/** Last observed completed UTC day, anchored to collection rather than the browser clock. */
export function activitySeries(s: Snapshot, kind: "fees" | "volume"): { date: string; usd: number }[] {
  const by = new Map<string, number>();
  const cutoff = s.snapshotDate ?? s.run.started_at.slice(0, 10);
  for (const id of Object.keys(s.facts.protocols)) {
    if (kind === "volume" && s.facts.protocols[id]?.category === "liquid-staking") continue;
    const a = s.facts.metrics[id]?.[kind];
    if (a?.kind !== "usd") continue;
    for (const d of a.days) if (d.date < cutoff) by.set(d.date, (by.get(d.date) ?? 0) + d.usd);
  }
  return [...by].sort(([a], [b]) => a.localeCompare(b)).map(([date, usd]) => ({ date, usd }));
}

export type DisplayActivity = Activity | { kind: "not_applicable"; reason: string };

export function activityRows(s: Snapshot): { id: string; name: string; volume: DisplayActivity; fees: DisplayActivity }[] {
  const dates = { volume: activitySeries(s, "volume").at(-1)?.date, fees: activitySeries(s, "fees").at(-1)?.date };
  const metric = (id: string, kind: "volume" | "fees"): DisplayActivity => {
    if (kind === "volume" && s.facts.protocols[id]?.category === "liquid-staking") return { kind: "not_applicable", reason: "Trading volume does not apply to liquid staking." };
    const a = s.facts.metrics[id]?.[kind] ?? NO_LISTING;
    if (a.kind !== "usd") return a;
    const date = dates[kind];
    const days = a.days.filter(d => date && d.date <= date);
    const latest = days.find(d => d.date === date);
    if (!date || !latest) return { kind: "unavailable", reason: date ? `No observation for ${date}` : "No completed day reported" };
    // F8: the one 7-day rule, aligned to the page's completed day.
    const { changeBp, gapDays } = change7d(days);
    return { kind: "usd", days, latest: latest.usd, change_7d_bp: changeBp, gap_days: gapDays };
  };
  return Object.keys(s.facts.protocols).map(id => ({ id, name: nameOf(s, id), volume: metric(id, "volume"), fees: metric(id, "fees") })).sort((a,b) => a.name.localeCompare(b.name));
}

export interface Exposure {
  id: string;
  name: string;
  roles: string[];
}

function exposure(s: Snapshot, mint: string): Exposure[] {
  return Object.entries(s.facts.protocols)
    .filter(([, p]) => p.tokens[mint])
    .map(([id, p]) => ({ id, name: p.name ?? id, roles: p.tokens[mint]! }))
    .sort((a, b) => a.id.localeCompare(b.id));
}

const RISK: Record<Rating, number> = { not_applicable: 0, low: 1, medium: 2, unknown: 3, high: 4 };

export function tokenRows(s: Snapshot) {
  return Object.entries(s.facts.tokens)
    .map(([mint, t]) => {
      const score = s.scores.tokens[mint];
      const ratings = Object.fromEntries(
        TOKEN_FACTORS.map((f) => [f, score?.factors[f]?.rating ?? "unknown"]),
      ) as Record<TokenFactor, Rating>;
      return {
        mint,
        symbol: t.symbol,
        kind: t.identity?.kind ?? null,
        issuer: t.identity?.issuer ?? null,
        ratings,
        worst: TOKEN_FACTORS.map((f) => ratings[f]).reduce((w, r) => (RISK[r] > RISK[w] ? r : w), "not_applicable" as Rating),
        exposure: exposure(s, mint),
      };
    })
    .sort((a, b) => a.symbol.localeCompare(b.symbol));
}

export interface ControlRow {
  label: string;
  text: string;
  controller: Controller;
  evidence: Evidence[];
  /** What this power can do, in plain words. */
  meaning: string;
}

function extensionRows(t: TokenFacts): ControlRow[] {
  const ext = t.extensions.value;
  const ev = t.extensions.evidence;
  const rows: ControlRow[] = [];
  const add = (label: string, c: Controller | undefined, meaning: string) => {
    if (c) rows.push({ label, text: describeController(c), controller: c, evidence: ev, meaning });
  };
  add("Permanent delegate", ext.permanent_delegate, "can move tokens out of any wallet");
  if (ext.transfer_hook) {
    add("Transfer hook authority", ext.transfer_hook.authority, "can point transfers at a program that can block them");
    if (ext.transfer_hook.program_controller) {
      add("Transfer hook program", ext.transfer_hook.program_controller, "upgrade control of the hook program");
    }
  }
  add("Transfer fee authority", ext.transfer_fee_authority, "can change the fee taken on every transfer");
  add("Pause authority", ext.pause_authority, "can pause all transfers");
  add("Mint close authority", ext.mint_close_authority, "can close the mint");
  add("Metadata update authority", ext.metadata_update_authority, "can change the name, symbol and metadata");
  return rows;
}

export function tokenPage(s: Snapshot, mint: string) {
  const t = s.facts.tokens[mint];
  if (!t) throw new Error(`unknown token ${mint}`);
  const score: TokenScore | undefined = s.scores.tokens[mint];
  return {
    mint,
    symbol: t.symbol,
    program: t.program,
    supply: t.supply,
    decimals: t.decimals,
    identity: t.identity ?? null,
    provenance: t.provenance,
    controls: [
      {
        label: "Mint authority",
        text: describeController(t.mint_authority.value),
        controller: t.mint_authority.value,
        evidence: t.mint_authority.evidence,
        meaning: "can create new supply",
      },
      {
        label: "Freeze authority",
        text: t.freeze_authority.value.kind === "immutable" ? "Renounced" : describeController(t.freeze_authority.value),
        controller: t.freeze_authority.value,
        evidence: t.freeze_authority.evidence,
        meaning: "can freeze any holder's balance",
      },
    ] satisfies ControlRow[],
    extensions: extensionRows(t),
    defaultFrozen: t.extensions.value.default_frozen,
    otherExtensions: t.extensions.value.other ?? [],
    unknownExtensions: t.extensions.value.unknown ?? [],
    factors: Object.fromEntries(
      TOKEN_FACTORS.map((f) => [
        f,
        score?.factors[f] ?? { rating: "unknown" as Rating, reason: "not scored", next_step: "rated once Celebtir can read the missing data", evidence: [] },
      ]),
    ) as Record<TokenFactor, FactorScore>,
    risks: score?.risks ?? [],
    exposure: exposure(s, mint),
  };
}

/** TOK-9: the last 30 days of supply in whole tokens, once 30 snapshots exist. */
export function supplySeries(h: TokenHistoryFile | undefined): { date: string; supply: number }[] | null {
  if (!h || h.days.length < 30) return null;
  return h.days.slice(-30).map((d) => ({ date: d.date, supply: d.supply / 10 ** d.decimals }));
}
