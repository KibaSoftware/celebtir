// @vitest-environment node
import { describe, expect, it } from "vitest";
import { dataDir, freshness, loadHistory, loadSnapshot } from "./data";

describe("ui_2 build-time data", () => {
  it("ui_2 falls back to the demo dataset and labels it", () => {
    delete process.env.CELEBTIR_DATA;
    const { demo } = dataDir();
    const s = loadSnapshot();
    const f = freshness(s);
    expect(f.demo).toBe(demo);
    expect(f.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(f.methodology).toBe("0.1");
    expect(f.slot).toBeGreaterThan(0);
    expect(Object.keys(s.scores.protocols)).toContain("kamino-lend");
  });

  it("ui_2 the date shown is the newest snapshot's, not the day the run finished", () => {
    const s = loadSnapshot();
    const days = loadHistory<{ days: { date: string }[] }>("summary")!.days;
    const f = freshness(s);
    expect(f.date).toBe(days[days.length - 1]!.date);
    // The demo fixture is rebuilt on other days: the run time stays available.
    expect(f.finishedAt).toBe(s.run.finished_at);
  });

  it("ui_2 history files load by name", () => {
    const summary = loadHistory<{ days: unknown[] }>("summary");
    expect(summary?.days.length).toBeGreaterThan(0);
    expect(loadHistory("no-such-protocol")).toBeUndefined();
  });
});
