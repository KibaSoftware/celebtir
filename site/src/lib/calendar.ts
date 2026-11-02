// Calendar-true chart axes. Pure: tested without a browser. Every day between
// the first and last observation gets a slot; a day with no snapshot is marked
// missing, so charts leave a gap instead of joining across it as if measured.

const DAY = 86_400_000;

export interface Dated {
  date: string;
  values: Record<string, number>;
}

export type CalendarDay<T extends Dated> = (T & { missing?: false }) | { date: string; values: Record<string, number>; missing: true };

const iso = (t: number) => new Date(t).toISOString().slice(0, 10);

/** Observations on a daily calendar; at most `limit` days, so a bad date cannot explode the axis. */
export function calendarDays<T extends Dated>(points: T[], limit = 4000): CalendarDay<T>[] {
  const sorted = [...points].sort((a, b) => a.date.localeCompare(b.date));
  if (sorted.length < 2) return sorted;
  const byDate = new Map(sorted.map((p) => [p.date, p]));
  const start = Date.parse(sorted[0]!.date);
  const end = Date.parse(sorted[sorted.length - 1]!.date);
  if (!Number.isFinite(start) || !Number.isFinite(end) || (end - start) / DAY > limit) return sorted;
  const out: CalendarDay<T>[] = [];
  for (let t = start; t <= end; t += DAY) {
    const date = iso(t);
    out.push(byDate.get(date) ?? { date, values: {}, missing: true });
  }
  return out;
}
