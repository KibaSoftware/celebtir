// Sparkline geometry. Pure: tested without a browser. A missing day breaks
// the line, so a gap in the data never reads as a measurement.

const DAY = 86_400_000;

export interface SparkPoint {
  date: string;
  usd: number;
}

/**
 * SVG path data for `points` in a `width` × `height` box. Days are placed on
 * a calendar axis; consecutive days are joined, and a missing day starts a
 * new segment. A single isolated day is drawn as a dot-sized segment.
 */
export function sparkPath(points: SparkPoint[], width: number, height: number, pad = 2): string {
  const days = [...points].sort((a, b) => a.date.localeCompare(b.date));
  if (days.length === 0) return "";
  const t = days.map((d) => Date.parse(d.date));
  const t0 = t[0]!;
  const span = Math.max(DAY, t[t.length - 1]! - t0);
  const max = Math.max(...days.map((d) => d.usd));
  const min = Math.min(...days.map((d) => d.usd));
  const range = max - min || 1;
  const x = (i: number) => pad + ((t[i]! - t0) / span) * (width - 2 * pad);
  const y = (v: number) => (max === min ? height / 2 : pad + (1 - (v - min) / range) * (height - 2 * pad));
  const parts: string[] = [];
  days.forEach((d, i) => {
    const joined = i > 0 && t[i]! - t[i - 1]! <= DAY;
    const px = x(i).toFixed(1);
    const py = y(d.usd).toFixed(1);
    if (joined) {
      parts.push(`L${px} ${py}`);
    } else {
      const next = days[i + 1];
      const alone = !next || Date.parse(next.date) - t[i]! > DAY;
      parts.push(alone ? `M${px} ${py}h0.01` : `M${px} ${py}`);
    }
  });
  return parts.join("");
}

/** How many segments a path has: one per run of consecutive days. */
export function segments(path: string): number {
  return (path.match(/M/g) ?? []).length;
}
