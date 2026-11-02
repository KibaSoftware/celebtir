import { formatUsd } from "@/lib/format";
import { sparkPath, type SparkPoint } from "@/lib/sparkline";
import styles from "./Charts.module.css";
import { ScopedLink } from "@/components/scope/ScopedLink";

/**
 * Server-rendered compositional charts (no JavaScript): a horizontal stacked
 * bar with its unit named, and a sparkline. Colors come from src/lib/labels.ts.
 */

export interface Segment {
  key: string;
  label: string;
  value: number;
  color: string;
  href?: string;
  /** Optional text pattern for segments that must read without colour. */
  pattern?: "dashed";
}

/**
 * One horizontal stacked bar. `unit` names what the bar measures ("% of TVL",
 * "protocols"); the legend gives each segment's exact value and share, so
 * nothing depends on colour or on reading segment widths.
 */
export function StackedBar({
  segments,
  unit,
  label,
  format = (n) => String(n),
  compact = false,
}: {
  segments: Segment[];
  unit: "share" | "count";
  label: string;
  format?: (n: number) => string;
  compact?: boolean;
}) {
  const total = segments.reduce((n, s) => n + s.value, 0);
  const pct = (v: number) => (total > 0 ? (v / total) * 100 : 0);
  const share = (v: number) => (total > 0 ? `${pct(v).toFixed(pct(v) < 10 && pct(v) > 0 ? 1 : 0)}%` : "–");
  const summary = segments.map((s) => `${s.label} ${format(s.value)}${unit === "share" ? ` (${share(s.value)})` : ""}`).join(", ");
  return (
    <figure className={styles.stack} data-compact={compact || undefined}>
      <div className={styles.bar} role={segments.some(s => s.href) ? "group" : "img"} aria-label={`${label}: ${summary}`}>
        {total === 0 ? <span className={styles.emptyBar} /> : null}
        {segments
          .filter((s) => s.value > 0)
          .map((s) => (
            s.href ? <ScopedLink key={s.key} href={s.href} className={styles.segment}
              style={{ flexGrow: s.value, background: s.color }}
              aria-label={`${s.label}: ${format(s.value)}, ${share(s.value)}. Explore protocols.`} /> :
            <span
              key={s.key}
              className={styles.segment}
              data-pattern={s.pattern}
              style={{ flexGrow: s.value, background: s.color }}
              title={`${s.label}: ${format(s.value)}${unit === "share" ? ` · ${share(s.value)}` : ""}`}
            />
          ))}
      </div>
      {compact ? null : (
        <ul className={styles.legend}>
          {segments.map((s) => (
            <li key={s.key} data-zero={s.value === 0 || undefined}>
              <span className={styles.key} data-pattern={s.pattern} style={{ background: s.color }} />
              <span className={styles.legendLabel}>{s.href ? <ScopedLink href={s.href}>{s.label}</ScopedLink> : s.label}</span>
              <span className={styles.legendValue}>
                {unit === "share" ? share(s.value) : format(s.value)}
                {unit === "share" ? <span className={styles.legendSub}> {format(s.value)}</span> : null}
              </span>
            </li>
          ))}
        </ul>
      )}
    </figure>
  );
}

/**
 * A small line of daily values. A missing day breaks the line; fewer than
 * two days shows a dash instead of a trend.
 */
export function Sparkline({ days, label, width = 96, height = 26 }: { days: SparkPoint[]; label: string; width?: number; height?: number }) {
  if (days.length < 2) {
    return (
      <span className={styles.noSpark} title="Fewer than two days reported">
        –
      </span>
    );
  }
  const sorted = [...days].sort((a, b) => a.date.localeCompare(b.date));
  const first = sorted[0]!;
  const last = sorted[sorted.length - 1]!;
  return (
    <svg
      className={styles.spark}
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label={`${label}, ${first.date} to ${last.date}: ${formatUsd(first.usd)} to ${formatUsd(last.usd)}`}
    >
      <path d={sparkPath(sorted, width, height)} fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

/** A thin share bar for table cells: the part of a total a row holds. */
export function ShareBar({ fraction, label }: { fraction: number; label: string }) {
  const pct = Math.max(0, Math.min(1, fraction)) * 100;
  return (
    <span className={styles.share} role="img" aria-label={`${label}: ${pct.toFixed(1)}%`}>
      <span className={styles.shareFill} style={{ width: `${pct}%` }} />
    </span>
  );
}
