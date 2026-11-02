import { describe, expect, it } from "vitest";
import { compareItems, compareRows, overlay, type CompareItem } from "./compare";
import { addTo, removeFrom, TRAY_MAX } from "./compareTray";
import { loadHistory, loadSnapshot } from "./data";
import type { ProtocolHistoryFile } from "./types";
import { peerOfKamino } from "./testSnapshot";

// The protocol compared with Kamino Lend in the snapshot being tested.
const PEER = peerOfKamino(loadSnapshot());


function items() {
  const s = loadSnapshot();
  const ids = Object.keys(s.scores.protocols);
  return compareItems(s, Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])));
}

describe("ui_16 compare rows", () => {
  it("ui_16 rows cover Stage, six factors with reasons, controllers, upgrades, TVL, activity and tokens", () => {
    const all = items();
    const rows = compareRows([all["kamino-lend"]!, all[PEER]!]);
    const labels = rows.map((r) => r.label);
    for (const l of [
      "Stage",
      "Missing for the next Stage",
      "Upgrade control",
      "Upgrade delay",
      "Admin powers",
      "Code transparency",
      "Oracle dependency",
      "Dependencies",
      "Upgrade controller",
      "Upgrades",
      "Last upgrade",
      "TVL",
      "24h volume",
      "24h fees",
      "Tokens used",
    ]) {
      expect(labels).toContain(l);
    }
    const control = rows.find((r) => r.label === "Upgrade control")!;
    expect(control.cells.map((c) => c.rating)).toEqual(["medium", all[PEER]!.factors.upgrade_control.rating]);
    expect(control.cells[0]!.text.length).toBeGreaterThan(10);
    for (const r of rows) expect(r.cells).toHaveLength(2);
  });

  it("ui_16 the same protocol twice matches on every row", () => {
    const k = items()["kamino-lend"]!;
    expect(compareRows([k, { ...k, id: "copy", name: "Copy" }]).every((r) => r.same)).toBe(true);
  });

  it("ui_17 rows differ when any one protocol differs", () => {
    const all = items();
    const rows = compareRows([all["kamino-lend"]!, all[PEER]!]);
    expect(rows.find((r) => r.label === "Upgrade controller")!.same).toBe(false);
  });
});

describe("ui_18 overlay series", () => {
  it("ui_18 one series per protocol, named, over the union of dates, gaps where a protocol has no value", () => {
    const a: Pick<CompareItem, "id" | "name"> & { days: { date: string; usd: number }[] } = {
      id: "a",
      name: "A",
      days: [
        { date: "2026-10-01", usd: 1 },
        { date: "2026-10-02", usd: 2 },
      ],
    };
    const b = { id: "b", name: "B", days: [{ date: "2026-10-02", usd: 5 }] };
    const o = overlay([a, b]);
    expect(o.series.map((s) => s.label)).toEqual(["A", "B"]);
    expect(o.points).toEqual([
      { date: "2026-10-01", values: { a: 1 } },
      { date: "2026-10-02", values: { a: 2, b: 5 } },
    ]);
    expect(new Set(o.series.map((s) => s.color)).size).toBe(2);
    expect(new Set(o.series.map((s) => s.dash)).size).toBe(2);
  });
});

describe("ui_4 compare tray", () => {
  it("ui_4 keeps the order chosen, ignores repeats and holds at most 4", () => {
    let t: string[] = [];
    for (const id of ["a", "b", "a", "c", "d", "e"]) t = addTo(t, id);
    expect(t).toEqual(["a", "b", "c", "d"]);
    expect(TRAY_MAX).toBe(4);
    expect(removeFrom(t, "b")).toEqual(["a", "c", "d"]);
  });
});
