import { describe, expect, it } from "vitest";
import { change7d } from "./activity";

const series = (values: number[]) =>
  values.map((usd, i) => ({ date: new Date(Date.UTC(2024, 0, 1) + i * 86_400_000).toISOString().slice(0, 10), usd }));

// The same cases as celebtir_core::activity, so both sides agree (F8).
describe("aud_8 7-day change", () => {
  it("compares two complete weeks", () => {
    expect(change7d(series([10, 10, 10, 10, 10, 10, 10, 20, 20, 20, 20, 20, 20, 20]))).toEqual({ changeBp: 10_000, gapDays: [] });
  });
  it("is null over a near-zero day and names it", () => {
    const s = series(Array(14).fill(100));
    s[3]!.usd = 1;
    expect(change7d(s)).toEqual({ changeBp: null, gapDays: [s[3]!.date] });
  });
  it("is null when a date is absent", () => {
    const s = series(Array(15).fill(100));
    s.splice(5, 1);
    expect(change7d(s).changeBp).toBeNull();
    expect(change7d(s).gapDays).toHaveLength(1);
  });
  it("needs fourteen days", () => {
    expect(change7d(series(Array(10).fill(5))).changeBp).toBeNull();
  });
});
