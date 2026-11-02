// F8, methodology v0.3 D5: the 7-day change of a daily DefiLlama series,
// computed exactly as the pipeline does (celebtir_core::activity): two
// calendar windows of seven days ending at the newest day; a window with an
// absent date, or a day below 5 % of the series median, has no total and the
// change is unavailable. One definition, so the leaderboard, the protocol
// page and the Activity page never disagree.

import type { Activity } from "./types";

export const GAP_PCT_OF_MEDIAN = 5;
const DAY = 86_400_000;

export interface DailyUsd {
  date: string;
  usd: number;
}

/** The middle sorted value (upper median), as the published checker computes it. */
export function median(series: DailyUsd[]): number {
  const v = series.map(d => d.usd).sort((a, b) => a - b);
  return v[Math.floor(v.length / 2)] ?? 0;
}

/** `{ changeBp, gapDays }`; the change is null over any gap or a zero prior week. */
export function change7d(series: DailyUsd[]): { changeBp: number | null; gapDays: string[] } {
  const last = series.at(-1);
  if (!last) return { changeBp: null, gapDays: [] };
  const m = median(series);
  const byDay = new Map(series.map(d => [d.date, d.usd]));
  const end = Date.parse(`${last.date}T00:00:00Z`);
  const gapDays: string[] = [];
  const totals = [0, 0];
  for (let w = 0; w < 2; w++) {
    for (let i = 0; i < 7; i++) {
      const day = new Date(end - (w * 7 + i) * DAY).toISOString().slice(0, 10);
      const usd = byDay.get(day);
      if (usd === undefined || usd * 100 < m * GAP_PCT_OF_MEDIAN) gapDays.push(day);
      else totals[w]! += usd;
    }
  }
  gapDays.sort();
  const [cur, prev] = totals as [number, number];
  return { changeBp: gapDays.length === 0 && prev > 0 ? Math.trunc(((cur - prev) * 10_000) / prev) : null, gapDays };
}

/** The published change, re-derived so older snapshots follow the same rule. */
export function weekOverWeek(a: Activity): number | null {
  return a.kind === "usd" ? change7d(a.days).changeBp : null;
}

export const CHANGE_LABEL = "7-day total vs previous 7 days";
