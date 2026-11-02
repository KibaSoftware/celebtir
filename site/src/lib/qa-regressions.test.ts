import { describe, expect, it } from "vitest";
import { loadSnapshot } from "./data";
import { activityRows, activitySeries } from "./pages";
import { withChain } from "./scope";

describe("qa completed-day and navigation regressions", () => {
  it("val_7 ui_9 cards equal rows and exclude the partial current day", () => {
    const s = structuredClone(loadSnapshot());
    s.snapshotDate = "2026-10-21";
    s.run.started_at = "2026-10-21T12:00:00Z";
    s.run.finished_at = s.run.started_at;
    for (const [id, m] of Object.entries(s.facts.metrics)) {
      s.facts.protocols[id]!.category = "dex";
      m.volume = { kind: "usd", latest: 999, change_7d_bp: null, days: [{ date: "2026-10-20", usd: 10 }, { date: "2026-10-21", usd: 1 }] };
    }
    const series = activitySeries(s, "volume");
    const rows = activityRows(s);
    expect(series.at(-1)?.date).toBe("2026-10-20");
    expect(series.at(-1)?.usd).toBe(rows.reduce((sum, r) => sum + (r.volume.kind === "usd" ? r.volume.latest : 0), 0));
    expect(series.at(-1)?.usd).toBe(Object.keys(s.facts.metrics).length * 10);
  });
  it("hui_1 scoped links replace chain and preserve query and hash", () => {
    expect(withChain("/risk/?chain=solana&factor=upgrade_control#table", "hyperliquid")).toBe("/risk/?chain=hyperliquid&factor=upgrade_control#table");
    expect(withChain("/risk/?chain=solana#table", "all")).toBe("/risk/#table");
  });
});

it("val_6 weekly change uses complete calendar weeks, not two daily samples", () => {
  const s = structuredClone(loadSnapshot()); s.snapshotDate = "2026-10-21";
  const id = Object.keys(s.scores.protocols)[0]!;
  s.facts.protocols[id]!.category = "spot-dex";
  const days = Array.from({ length: 14 }, (_, i) => ({ date: `2026-10-${String(i+7).padStart(2, "0")}`, usd: i < 7 ? 10 : 20 }));
  s.facts.metrics[id]!.volume = { kind: "usd", latest: 999, change_7d_bp: 999, days };
  const a = activityRows(s).find(r => r.id === id)!.volume;
  expect(a.kind === "usd" && a.change_7d_bp).toBe(10000);
  days.splice(3, 1);
  const missing = activityRows(s).find(r => r.id === id)!.volume;
  expect(missing.kind === "usd" && missing.change_7d_bp).toBeNull();
});
