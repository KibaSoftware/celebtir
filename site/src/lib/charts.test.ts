import { describe, expect, it } from "vitest";
import { calendarDays } from "./calendar";
import { segments, sparkPath } from "./sparkline";

describe("ui_8 charts never fill in a missing day", () => {
  it("ui_8 the calendar axis keeps a slot for each missing day, marked missing", () => {
    const days = calendarDays([
      { date: "2026-10-01", values: { tvl: 1 } },
      { date: "2026-10-04", values: { tvl: 2 } },
    ]);
    expect(days.map((d) => d.date)).toEqual(["2026-10-01", "2026-10-02", "2026-10-03", "2026-10-04"]);
    expect(days.map((d) => Boolean(d.missing))).toEqual([false, true, true, false]);
    expect(days[1]!.values).toEqual({});
  });

  it("ui_8 contiguous days stay as they are, in date order", () => {
    const days = calendarDays([
      { date: "2026-10-02", values: { tvl: 2 } },
      { date: "2026-10-01", values: { tvl: 1 } },
    ]);
    expect(days.map((d) => d.date)).toEqual(["2026-10-01", "2026-10-02"]);
    expect(days.some((d) => d.missing)).toBe(false);
  });

  it("ui_8 a sparkline breaks at a missing day instead of joining across it", () => {
    const joined = sparkPath(
      [
        { date: "2026-10-01", usd: 1 },
        { date: "2026-10-02", usd: 2 },
        { date: "2026-10-03", usd: 3 },
      ],
      96,
      26,
    );
    expect(segments(joined)).toBe(1);
    const gap = sparkPath(
      [
        { date: "2026-10-01", usd: 1 },
        { date: "2026-10-02", usd: 2 },
        { date: "2026-10-05", usd: 3 },
        { date: "2026-10-06", usd: 1 },
      ],
      96,
      26,
    );
    expect(segments(gap)).toBe(2);
  });

  it("ui_8 an empty series draws nothing", () => {
    expect(sparkPath([], 96, 26)).toBe("");
  });
});
