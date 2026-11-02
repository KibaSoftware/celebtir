import { describe, expect, it } from "vitest";
import { applyView, categoryCounts, rangeDays, riskScore, withRanks, type ProtocolRow, type RatedRow } from "./summary";
import { DEFAULT_VIEW, type ViewState } from "./viewState";

function row(id: string, over: Partial<RatedRow> = {}): RatedRow {
  return {
    rank: 0,
    id,
    chain: "solana",
    name: id.toUpperCase(),
    category: "lending",
    website: null,
    assessment: "assessed",
    stage: "stage_0",
    nextStage: "stage_1",
    missing: [],
    ratings: {
      upgrade_control: "medium",
      upgrade_delay: "high",
      admin_powers: "low",
      code_transparency: "low",
      oracle_dependency: "not_applicable",
      dependencies: "low",
    },
    reasons: {} as RatedRow["reasons"],
    controller: "Squads 5 of 10 · 24 h delay",
    controlClass: "multisig_with_delay",
    tvl: 100,
    tvlChange7d: null,
    activity: null,
    lastChange: null,
    ...over,
  };
}

const rows = withRanks([
  row("alpha", { tvl: 300, category: "dex", stage: "stage_1", controlClass: "immutable" }),
  row("beta", { tvl: 100, controlClass: "single_key", ratings: { ...row("x").ratings, upgrade_delay: "low" } }),
  row("gamma", { tvl: null, category: "perps", stage: "unrated" }),
]);

const view = (over: Partial<ViewState>): ViewState => ({ ...DEFAULT_VIEW, ...over });

describe("ui_9 category tabs", () => {
  it("ui_9 counts match the rows per category, with All", () => {
    expect(categoryCounts(rows)).toEqual({ all: 3, lending: 1, dex: 1, perps: 1, "liquid-staking": 0 });
  });
});

describe("ui_10 sorting", () => {
  it("ui_10 sorts by TVL descending by default, unknown TVL last", () => {
    expect(applyView(rows, DEFAULT_VIEW).map((r) => r.id)).toEqual(["alpha", "beta", "gamma"]);
  });

  it("ui_10 every column sorts both ways", () => {
    for (const key of ["rank", "risk", "name", "category", "stage", "tvl", "activity", "last_change", "controller", "upgrade_control", "upgrade_delay"]) {
      const asc = applyView(rows, view({ sort: { key, dir: "asc" } })).map((r) => r.id);
      const desc = applyView(rows, view({ sort: { key, dir: "desc" } })).map((r) => r.id);
      expect(asc).toHaveLength(3);
      expect(desc).toHaveLength(3);
    }
    expect(applyView(rows, view({ sort: { key: "name", dir: "asc" } })).map((r) => r.id)).toEqual(["alpha", "beta", "gamma"]);
    expect(applyView(rows, view({ sort: { key: "name", dir: "desc" } })).map((r) => r.id)).toEqual(["gamma", "beta", "alpha"]);
  });

  it("ui_10 rank follows TVL, unknown TVL last", () => {
    expect(rows.map((r) => [r.id, r.rank])).toEqual([["alpha", 1], ["beta", 2], ["gamma", 3]]);
    expect(applyView(rows, view({ sort: { key: "rank", dir: "asc" } })).map((r) => r.id)).toEqual(["alpha", "beta", "gamma"]);
  });

  it("ui_10 the rosette sorts by its combined risk", () => {
    expect(riskScore(rows[1])!).toBeLessThan(riskScore(rows[0])!);
    expect(applyView(rows, view({ sort: { key: "risk", dir: "asc" } }))[0].id).toBe("beta");
  });

  it("ui_10 ratings sort by risk, not alphabetically", () => {
    const ids = applyView(rows, view({ sort: { key: "upgrade_delay", dir: "desc" } })).map((r) => r.id);
    expect(ids[ids.length - 1]).toBe("beta");
  });
});

describe("ui_11 filters", () => {
  it("ui_11 filter by tab, stage, controller class, factor rating and search", () => {
    expect(applyView(rows, view({ tab: "dex" })).map((r) => r.id)).toEqual(["alpha"]);
    expect(applyView(rows, view({ filters: { stage: ["unrated"], controller: [], rating: {} } })).map((r) => r.id)).toEqual(["gamma"]);
    expect(applyView(rows, view({ filters: { stage: [], controller: ["single_key"], rating: {} } })).map((r) => r.id)).toEqual(["beta"]);
    expect(applyView(rows, view({ filters: { stage: [], controller: [], rating: { upgrade_delay: ["low"] } } })).map((r) => r.id)).toEqual(["beta"]);
    expect(applyView(rows, view({ q: "gam" })).map((r) => r.id)).toEqual(["gamma"]);
  });
});

describe("ui_8 chart ranges", () => {
  const days = Array.from({ length: 40 }, (_, i) => ({ date: `d${i}` }));
  it("ui_8 7-day, 30-day and all-time ranges", () => {
    expect(rangeDays(days, "7d")).toHaveLength(7);
    expect(rangeDays(days, "30d")).toHaveLength(30);
    expect(rangeDays(days, "all")).toHaveLength(40);
    expect(rangeDays(days.slice(0, 3), "30d")).toHaveLength(3);
  });
});

describe("aud_5 unassessed rows", () => {
  const unassessed: ProtocolRow = { ...row("evm", { tvl: 50 }), chain: "hyperliquid", assessment: "unassessed", stage: null, ratings: null, reasons: null, controlClass: null, nextStage: null };
  it("aud_5 rating filters never match an unassessed row", () => {
    const all = withRanks([...rows, unassessed]);
    expect(applyView(all, view({ filters: { ...DEFAULT_VIEW.filters, stage: ["stage_0"] } })).map(r => r.id)).not.toContain("evm");
    expect(applyView(all, view({ filters: { ...DEFAULT_VIEW.filters, controller: ["multisig_with_delay"] } })).map(r => r.id)).not.toContain("evm");
    expect(riskScore(unassessed)).toBeNull();
  });
  it("aud_5 sorting by Stage puts unassessed rows last", () => {
    const all = withRanks([...rows, unassessed]);
    expect(applyView(all, view({ sort: { key: "stage", dir: "desc" } })).at(-1)!.id).toBe("evm");
  });
});
