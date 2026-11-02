import { describe, expect, it } from "vitest";
import { loadHistory, loadSnapshot } from "./data";
import { controlShares, factorDistribution, insights, kpis, recentOnChain, stageDistribution, tvlDelta } from "./overview";
import { assessedRows, availableRanges, buildRows, rangeDays } from "./summary";
import { FACTORS, type ChangeEvent, type ProtocolHistoryFile, type SummaryDay } from "./types";

const s = loadSnapshot();
const ids = Object.keys(s.scores.protocols);
const allRows = buildRows(s, Object.fromEntries(Object.keys(s.facts.protocols).map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])), []);
// F5: rating aggregates run over assessed protocols only.
const rows = assessedRows(allRows);
const day = (date: string, tvl: number): SummaryDay => ({
  date,
  protocols: 2,
  total_tvl_usd: tvl,
  value_by_control: { classes: {}, total: tvl },
  stages: { unrated: 0, stage_0: 2, stage_1: 0, stage_2: 0 },
  methodology_version: "0.1",
});
const event = (id: string, date: string, source: ChangeEvent["source"] = "on_chain"): ChangeEvent => ({
  id,
  date,
  detected_within_days: 1,
  protocol: "kamino-lend",
  subject: "x",
  kind: "controller_changed",
  severity: "warning",
  source,
  summary: `summary ${id}`,
  signature: null,
});

describe("ui_8 summary KPIs come only from published fields", () => {
  it("ui_8 total TVL, assessed count and High upgrade control match the snapshot", () => {
    const k = kpis(allRows, s.scores.value_by_control, [], [], "2026-10-17");
    expect(k.totalTvl).toBe(s.scores.value_by_control.all.total);
    expect(k.assessed).toBe(ids.length);
    expect(k.unassessed).toBe(allRows.length - ids.length);
    expect(k.tvlExcluded).toBe(s.scores.value_by_control.excluded.length);
    expect(k.highUpgradeControl).toBe(ids.filter((id) => s.scores.protocols[id]!.factors.upgrade_control.rating === "high").length);
    const undelayed = ["single_key", "unrecognized", "multisig_without_delay", "multisig_short_delay"].reduce((n, c) => n + (s.scores.value_by_control.all.classes[c] ?? 0), 0);
    expect(k.undelayedTvl).toBe(undelayed);
  });

  it("ui_8 no change is shown when history does not reach back 7 days", () => {
    expect(tvlDelta([])).toBeNull();
    expect(tvlDelta([day("2026-10-15", 100), day("2026-10-17", 110)])).toBeNull();
    const d = tvlDelta([day("2026-10-01", 100), day("2026-10-09", 110)]);
    expect(d).toEqual({ fraction: 0.1, since: "2026-10-01" });
  });

  it("ui_8 recent changes count only on-chain events inside the window, up to the snapshot date", () => {
    const events = [event("a", "2026-10-16"), event("b", "2026-09-01"), event("c", "2026-10-10", "registry_edit"), event("d", "2026-10-20")];
    expect(recentOnChain(events, "2026-10-17").map((e) => e.id)).toEqual(["a"]);
  });

  it("ui_8 factor distribution keeps Unknown and Not applicable apart and counts every protocol once", () => {
    const d = factorDistribution(rows);
    for (const f of FACTORS) expect(Object.values(d[f]).reduce((a, b) => a + b, 0)).toBe(rows.length);
    expect(d.dependencies.unknown).toBe(rows.filter((r) => r.ratings.dependencies === "unknown").length);
    expect(d.oracle_dependency.not_applicable).toBe(rows.filter((r) => r.ratings.oracle_dependency === "not_applicable").length);
  });

  it("ui_8 Stage distribution and control shares add up", () => {
    expect(Object.values(stageDistribution(rows)).reduce((a, b) => a + b, 0)).toBe(rows.length);
    const shares = controlShares(s.scores.value_by_control.all);
    expect(shares.map((x) => x.cls)).toEqual(["single_key", "unrecognized", "multisig_without_delay", "multisig_short_delay", "multisig_with_delay", "immutable"]);
    expect(shares.reduce((n, x) => n + x.usd, 0)).toBe(s.scores.value_by_control.all.total);
  });

  it("ui_8 insights are rules over the ratings, each item linking to its factor", () => {
    const notes = insights(rows, [], "2026-10-17", {});
    const delay = notes.find((n) => n.key === "delay")!;
    expect(delay.items.map((i) => i.id)).toEqual(rows.filter((r) => r.ratings.upgrade_delay === "high").map((r) => r.id).sort());
    for (const i of delay.items) expect(i.href).toBe(`/protocols/${i.id}/#factor-upgrade_delay`);
    const controller = notes.find((n) => n.key === "controller")!;
    expect(controller.items.every((i) => ["single_key", "unrecognized"].includes(rows.find((r) => r.id === i.id)!.controlClass))).toBe(true);
    expect(notes.find((n) => n.key === "changes")!.items).toEqual([]);
  });

  it("lch_3 insight wording stays factual", () => {
    const text = JSON.stringify(insights(rows, [event("a", "2026-10-16")], "2026-10-17", { "kamino-lend": "Kamino Lend" }));
    for (const word of ["unsafe", "scam", "rug"]) expect(text.toLowerCase()).not.toContain(word);
  });
});

describe("ui_8 chart ranges", () => {
  it("ui_8 a range is offered only once the history can fill it", () => {
    expect(availableRanges(3)).toEqual(["7d", "all"]);
    expect(availableRanges(30)).toEqual(["7d", "30d", "all"]);
    expect(availableRanges(400)).toEqual(["7d", "30d", "90d", "1y", "all"]);
  });

  it("ui_8 90-day and 1-year ranges slice the last days", () => {
    const days = Array.from({ length: 400 }, (_, i) => i);
    expect(rangeDays(days, "90d")).toHaveLength(90);
    expect(rangeDays(days, "1y")).toHaveLength(365);
    expect(rangeDays(days, "all")).toHaveLength(400);
  });
});

describe("aud_6 change dates", () => {
  it("aud_6 recent on-chain changes window by the block date, not the recording date", () => {
    const backfilled = { ...event("old", "2026-10-17"), occurred_on: "2025-01-01", backfill: true };
    const recent = { ...event("new", "2026-10-17"), occurred_on: "2026-10-10", backfill: true };
    expect(recentOnChain([backfilled, recent], "2026-10-17").map((e) => e.id)).toEqual(["new"]);
  });
});
