import { describe, expect, it } from "vitest";
import { DEFAULT_VIEW, detailOf, parseView, serializeView, type ViewState } from "./viewState";

describe("ui_1 view state lives in the URL", () => {
  it("ui_1 the default view has an empty query", () => {
    expect(serializeView(DEFAULT_VIEW)).toBe("");
    expect(parseView(new URLSearchParams(""))).toEqual(DEFAULT_VIEW);
  });

  it("ui_1 a copied URL reproduces the exact view", () => {
    const view: ViewState = {
      chain: "hyperliquid",
      tab: "lending",
      sort: { key: "tvl", dir: "asc" },
      hidden: ["activity", "last_change"],
      filters: { stage: ["stage_0", "unrated"], controller: ["single_key"], rating: { upgrade_delay: ["high"] } },
      compare: ["kamino-lend", "marginfi"],
      diff: true,
      q: "kam",
      range: "7d",
      factor: "admin_powers",
      highlight: ["high", "unknown"],
      highlightFactor: "upgrade_control",
      detail: "kamino-lend.upgrade_delay",
      tokenKind: ["stablecoin"],
      tokenRating: ["high"],
    };
    const url = serializeView(view);
    expect(parseView(new URLSearchParams(url))).toEqual(view);
  });

  it("ui_1 serialization is stable: same view, same URL, whatever the order of choices", () => {
    const a = serializeView({ ...DEFAULT_VIEW, hidden: ["b", "a"], filters: { stage: ["stage_1", "stage_0"], controller: [], rating: {} } });
    const b = serializeView({ ...DEFAULT_VIEW, hidden: ["a", "b"], filters: { stage: ["stage_0", "stage_1"], controller: [], rating: {} } });
    expect(a).toBe(b);
  });

  it("ui_1 the compare selection keeps its order and holds at most 4", () => {
    const v = parseView(new URLSearchParams("p=c,a,b,d,e"));
    expect(v.compare).toEqual(["c", "a", "b", "d"]);
  });

  it("ui_1 unknown or malformed parameters fall back to defaults", () => {
    const v = parseView(new URLSearchParams("tab=casino&sort=tvl:sideways&stage=stage_9,stage_1&rating.nope=high&rating.upgrade_delay=extreme,low"));
    expect(v.tab).toBe("all");
    expect(v.sort).toEqual(DEFAULT_VIEW.sort);
    expect(v.filters.stage).toEqual(["stage_1"]);
    expect(v.filters.rating).toEqual({ upgrade_delay: ["low"] });
  });

  it("ui_1 the chart range is part of the view", () => {
    expect(parseView(new URLSearchParams("range=7d")).range).toBe("7d");
    expect(parseView(new URLSearchParams("range=decade")).range).toBe(DEFAULT_VIEW.range);
    expect(serializeView({ ...DEFAULT_VIEW, range: "all" })).toBe("range=all");
  });

  it("ui_1 the Risk page's factor sub-tab is part of the view", () => {
    expect(parseView(new URLSearchParams("factor=upgrade_delay")).factor).toBe("upgrade_delay");
    expect(parseView(new URLSearchParams("factor=vibes")).factor).toBeNull();
    expect(serializeView({ ...DEFAULT_VIEW, factor: "dependencies" })).toBe("factor=dependencies");
  });

  it("ui_1 the risk matrix's highlight and open detail panel are part of the view", () => {
    const v = parseView(new URLSearchParams("hl=high,bogus,low&hlf=admin_powers&detail=kamino-lend.upgrade_control"));
    expect(v.highlight).toEqual(["high", "low"]);
    expect(v.highlightFactor).toBe("admin_powers");
    expect(detailOf(v)).toEqual({ id: "kamino-lend", factor: "upgrade_control" });
    expect(serializeView(v)).toBe("hl=high%2Clow&hlf=admin_powers&detail=kamino-lend.upgrade_control");
  });

  it("ui_1 a malformed detail parameter opens nothing", () => {
    for (const bad of ["kamino-lend", "kamino-lend.vibes", "../x.upgrade_control", ".upgrade_control"]) {
      expect(parseView(new URLSearchParams({ detail: bad })).detail, bad).toBeNull();
    }
  });

  it("ui_8 the 90-day and 1-year ranges are part of the view", () => {
    expect(parseView(new URLSearchParams("range=90d")).range).toBe("90d");
    expect(parseView(new URLSearchParams("range=1y")).range).toBe("1y");
  });
});
