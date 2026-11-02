"use client";

import { TimeChart, type Point, type Series, type Unit } from "./TimeChart";
import { availableRanges, rangeDays } from "@/lib/summary";
import { useViewState } from "@/lib/useViewState";
import { DEFAULT_VIEW, RANGES, type Range, type ViewState } from "@/lib/viewState";
import { RANGE_LABELS } from "./SummaryCharts";
import styles from "./SummaryCharts.module.css";

export interface ChartSpec {
  title: string;
  description: string;
  series: Series[];
  points: Point[];
  unit: Unit;
}

/**
 * Charts that share one time range, kept in the URL (UI-1). Only ranges the
 * history can fill are offered; a chart with fewer than two days is left to
 * the caller's empty state.
 */
export function RangedChartsView({ charts, view, setRange, label }: { charts: ChartSpec[]; view: ViewState; setRange: (r: Range) => void; label: string }) {
  const count = Math.max(0, ...charts.map((c) => c.points.length));
  const offered = availableRanges(count);
  const range = offered.includes(view.range) ? view.range : "all";
  return (
    <section className={styles.section} aria-label={label}>
      <div className={styles.head}>
        <p className={styles.headingNote}>
          {count} daily observation{count === 1 ? "" : "s"}. Days without an observation are left as gaps, never filled in.
        </p>
        <div className={styles.ranges} role="group" aria-label="Time range">
          {RANGES.map((r) => (
            <button
              key={r}
              type="button"
              aria-pressed={range === r}
              disabled={!offered.includes(r)}
              title={offered.includes(r) ? undefined : `Needs ${RANGE_LABELS[r]} of snapshots; ${count} so far`}
              className={styles.range}
              onClick={() => setRange(r)}
            >
              {RANGE_LABELS[r]}
            </button>
          ))}
        </div>
      </div>
      <div className={styles.gridFill}>
        {charts.map((c) => (
          <TimeChart key={c.title} title={c.title} description={c.description} series={c.series} points={rangeDays(c.points, range)} unit={c.unit} />
        ))}
      </div>
    </section>
  );
}

export function RangedCharts({ charts, label }: { charts: ChartSpec[]; label: string }) {
  const [view, update] = useViewState();
  return <RangedChartsView charts={charts} label={label} view={view} setRange={(range) => update((v) => ({ ...v, range }))} />;
}

export function RangedChartsDefault({ charts, label }: { charts: ChartSpec[]; label: string }) {
  return <RangedChartsView charts={charts} label={label} view={DEFAULT_VIEW} setRange={() => {}} />;
}
