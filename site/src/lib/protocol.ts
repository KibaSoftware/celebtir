// The protocol page's data (UI-12), derived at build time from the published
// files. Every section reads from this; nothing is fetched in the browser.

import { assessmentOf, unassessedReason } from "./assessment";
import { stageInspection, type StageInspection } from './scoreReview';
import { describeController } from "./controller";
import { buildGraph, type ControlGraph } from "./graph";
import { actorsFor, type Actor, type Grant } from "./permissions";
import type {
  Activity,
  ChangeEvent,
  CodeTransparency,
  Controller,
  Dependencies,
  Fact,
  Factor,
  FactorScore,
  Metric,
  NodeRating,
  OracleSource,
  ProtocolHistoryFile,
  ProtocolMetrics,
  RegistrySource,
  Requirement,
  Snapshot,
  Stage,
  UnderlyingFeeds,
  UpgradeEvent,
} from "./types";
import type { Rating } from "./tokens";

export interface ProgramRow {
  address: string;
  name: string | null;
  role: "core" | "periphery";
  loader: string | null;
  controller: Fact<Controller>;
  controllerText: string;
  transparency: Fact<CodeTransparency>;
  /** Null when the program's history has not been read. */
  history: { lastChange: UpgradeEvent | null; complete: boolean } | null;
}

export interface DatedUpgrade extends UpgradeEvent {
  program: string;
  programName: string | null;
}

export interface ProtocolPage {
  inspection?: StageInspection;
  id: string;
  name: string;
  chain: string;
  category: string | null;
  website: string | null;
  squadsControlled: boolean;
  allCoreVerified: boolean;
  /** `unassessed`: facts only, no Stage, factors or risk sentences (F5). */
  assessment: "assessed" | "unassessed";
  /** Why an unassessed protocol is not rated. */
  unassessedReason: string | null;
  /** Null when unassessed. */
  stage: Stage | null;
  nextStage: Stage | null;
  requirements: Requirement[];
  missing: Requirement[];
  tvl: Metric;
  tvlDays: { date: string; usd: number }[];
  assetGroups: ProtocolMetrics["asset_groups"];
  fees: Activity;
  volume: Activity;
  tokens: { mint: string; symbol: string; roles: string[]; worst: Rating; warning: string | null }[];
  risks: { severity: "high" | "medium"; category: string; factor: Factor; text: string }[];
  /** Null when unassessed. */
  factors: Record<Factor, FactorScore> | null;
  actors: Actor[];
  renounced: Grant[];
  graph: ControlGraph;
  programs: ProgramRow[];
  upgrades: ProtocolHistoryFile["upgrades"] | null;
  upgradeEvents: DatedUpgrade[];
  historyScanned: { scanned: number; total: number } | null;
  events: ChangeEvent[];
  dependencies: Fact<Dependencies>;
  /** Feeds behind protocol-run aggregators, per market (edge case 11). */
  aggregatorFeeds: { oracle: string; market: string; underlying: Fact<UnderlyingFeeds> }[];
  /** Each sampled callee's own rating, from the scores. */
  calleeRatings: Record<string, NodeRating>;
  /** Every registered price feed (F4). */
  oracles: { account: string; market: string; role: string; provider: string | null; boundsPrimary: boolean; guardedBy: string[]; source: OracleSource }[];
  sources: RegistrySource[];
  /** `reviewedBy` is null until a human review (H2). */
  review: { lastReviewed: string | null; reviewedBy: string | null; due: boolean; awaitingHumanReview: boolean };
}

const UNAVAILABLE = (reason: string) => ({ kind: "unavailable" as const, reason });
const NO_METRICS = "DefiLlama has no listing for this protocol";
const UPGRADE_KINDS = new Set(["deploy", "upgrade", "remove_authority", "set_authority", "close"]);

/** Every protocol with facts, assessed or not: each has a page. */
export function protocolIds(s: Snapshot): string[] {
  return Object.keys(s.facts.protocols).sort();
}

export function buildProtocolPage(
  s: Snapshot,
  id: string,
  history: ProtocolHistoryFile | undefined,
  events: ChangeEvent[],
): ProtocolPage {
  const facts = s.facts.protocols[id];
  if (!facts) throw new Error(`unknown protocol ${id}`);
  const assessment = assessmentOf(s, id);
  const score = assessment === "assessed" ? s.scores.protocols[id] : undefined;
  const metrics = s.facts.metrics[id];
  const names = new Map(facts.sources.filter((x) => x.kind === "program").map((x) => [x.address, x.label]));
  const core = Object.values(facts.programs).filter((p) => p.role === "core");
  const recorded = new Map(Object.keys(facts.programs).map(address => [address, [...(s.facts.program_history[address]?.events ?? [])]]));
  const kinds: Partial<Record<string, UpgradeEvent["kind"]>> = {program_upgraded:"upgrade", upgrade_authority_set:"set_authority", upgrade_authority_removed:"remove_authority"};
  for (const event of events) {
    const kind = kinds[event.kind];
    if (event.protocol !== id || event.source !== "on_chain" || !kind || !event.signature || event.slot == null) continue;
    const evidence = event.evidence?.find(e => recorded.has(e.address) && e.slot === event.slot && e.explorer_url?.endsWith(`/tx/${event.signature}`));
    if (!evidence) continue;
    const known = recorded.get(evidence.address)!;
    if (!known.some(e => e.signature === event.signature && e.slot === event.slot && e.kind === kind)) {
      known.push({kind,signature:event.signature,slot:event.slot,date:event.occurred_on ?? null});
    }
  }
  for (const known of recorded.values()) known.sort((a,b)=>a.slot-b.slot);

  const programs: ProgramRow[] = Object.entries(facts.programs).map(([address, p]) => {
    const h = s.facts.program_history[address];
    const changes = recorded.get(address)!.filter((e) => UPGRADE_KINDS.has(e.kind));
    return {
      address,
      name: names.get(address) ?? null,
      role: p.role,
      loader: p.loader,
      controller: p.upgrade_control,
      controllerText: describeController(p.upgrade_control.value),
      transparency: p.code_transparency,
      history: h || changes.length ? { lastChange: changes[changes.length - 1] ?? null, complete: h?.complete ?? false } : null,
    };
  });
  programs.sort((a, b) => (a.role === b.role ? a.address.localeCompare(b.address) : a.role === "core" ? -1 : 1));

  const upgradeEvents: DatedUpgrade[] = programs
    .flatMap((p) =>
      recorded.get(p.address)!.map((e) => ({ ...e, program: p.address, programName: p.name })),
    )
    .sort((a, b) => b.slot - a.slot);
  const histories = programs.map((p) => s.facts.program_history[p.address]).filter((h) => h !== undefined);

  const { actors, renounced } = actorsFor(facts, s.facts.tokens);

  return {
    id,
    name: facts.name ?? id,
    chain: facts.chain,
    category: facts.category,
    website: facts.website,
    squadsControlled: core.some((p) => {
      const c = p.upgrade_control.value;
      return c.kind === "multisig" && c.multisig_kind !== "safe";
    }),
    allCoreVerified: core.length > 0 && core.every((p) => p.code_transparency.value.kind === "verified"),
    assessment,
    unassessedReason: score ? null : unassessedReason(s, id),
    stage: score?.stage.stage ?? null,
    inspection: score && facts.chain === 'solana' ? stageInspection(id,facts,score,s.scores.methodology_version,s.snapshotDate ?? s.run.finished_at.slice(0,10)) : undefined,
    nextStage: score?.stage.next ?? null,
    requirements: score?.stage.requirements ?? [],
    missing: score?.stage.missing ?? [],
    tvl: metrics?.tvl ?? UNAVAILABLE(NO_METRICS),
    tvlDays: (history?.days ?? []).flatMap((d) => (d.tvl_usd === null ? [] : [{ date: d.date, usd: d.tvl_usd }])),
    assetGroups: metrics?.asset_groups ?? UNAVAILABLE(NO_METRICS),
    fees: metrics?.fees ?? UNAVAILABLE(NO_METRICS),
    volume: metrics?.volume ?? UNAVAILABLE(NO_METRICS),
    tokens: Object.entries(score?.tokens ?? {})
      .map(([mint, t]) => ({ mint, symbol: t.symbol, roles: t.roles, worst: t.worst, warning: t.warning ?? Object.values(s.scores.tokens[mint]?.factors ?? {}).filter(f => f.rating === t.worst).map(f => f.reason).join("; ") }))
      .sort((a, b) => a.symbol.localeCompare(b.symbol)),
    risks: [...(score?.risks ?? [])].sort((a, b) => (a.severity === b.severity ? 0 : a.severity === "high" ? -1 : 1)),
    factors: score?.factors ?? null,
    actors,
    renounced,
    graph: buildGraph(s, id),
    programs,
    upgrades: history?.upgrades ?? null,
    upgradeEvents,
    historyScanned: histories.length
      ? { scanned: histories.reduce((n, h) => n + h.scanned, 0), total: histories.reduce((n, h) => n + h.total, 0) }
      : null,
    events: events.filter((e) => e.protocol === id),
    dependencies: facts.dependencies,
    aggregatorFeeds: Object.entries(facts.oracles)
      .flatMap(([oracle, o]) => [
        ...(o.underlying ? [{ oracle, market: o.market, underlying: o.underlying }] : []),
        ...Object.entries(o.underlying_markets ?? {}).map(([market, underlying]) => ({ oracle, market, underlying })),
      ])
      .sort((a, b) => a.market.localeCompare(b.market)),
    calleeRatings: score?.callees ?? {},
    oracles: Object.entries(facts.oracles)
      .map(([account, o]) => ({ account, market: o.market, role: o.role, provider: o.provider ?? null, boundsPrimary: o.bounds_primary ?? false, guardedBy: o.guarded_by ?? [], source: o.source.value }))
      .sort((a, b) => a.market.localeCompare(b.market) || (a.role === b.role ? a.account.localeCompare(b.account) : a.role === "primary" ? -1 : 1)),
    sources: facts.sources,
    review: {
      lastReviewed: facts.last_reviewed,
      reviewedBy: facts.reviewed_by,
      due: history?.review.due ?? facts.last_reviewed === null,
      awaitingHumanReview: history?.review.awaiting_human_review ?? facts.reviewed_by === null,
    },
  };
}
