"use client";

import { TimeChart, type Point } from "./TimeChart";
import { CONTROL_COLORS, CONTROL_LABELS, SERIES_COLOR, STAGE_COLORS, STAGE_LABELS } from "@/lib/labels";
import { availableRanges, rangeDays } from "@/lib/summary";
import { CONTROL_CLASSES, type SummaryDay, type Stage } from "@/lib/types";
import { useViewState } from "@/lib/useViewState";
import { DEFAULT_VIEW, RANGES, type Range, type ViewState } from "@/lib/viewState";
import { EmptyState } from "@/components/ui/Ui";
import styles from "./SummaryCharts.module.css";

const STAGES: Stage[] = ["unrated", "stage_0", "stage_1", "stage_2"];
export const RANGE_LABELS: Record<Range, string> = { "7d": "7 days", "30d": "30 days", "90d": "90 days", "1y": "1 year", all: "All time" };

/** UI-8: three charts over the same range; switching it does not reload. */
export function SummaryChartsView({
  days,
  view,
  setRange,
}: {
  days: SummaryDay[];
  view: ViewState;
  setRange: (r: Range) => void;
}) {
  if (days.length < 2) {
    return (
      <div className={styles.waiting}>
        <EmptyState kind="history">
          Charts appear once two daily snapshots exist. {days.length === 1 ? `The first is from ${days[0]!.date}; the figures above are from it.` : ""}
        </EmptyState>
      </div>
    );
  }
  const offered = availableRanges(days.length);
  // A range the history cannot fill shows everything there is.
  const range = offered.includes(view.range) ? view.range : "all";
  const shown = rangeDays(days, range);
  const tvl: Point[] = shown.map((d) => ({ date: d.date, values: { tvl: d.total_tvl_usd } }));
  const value: Point[] = shown.map((d) => ({ date: d.date, values: d.value_by_control.classes }));
  const stages: Point[] = shown.map((d) => ({ date: d.date, values: d.stages }));

  return (
    <section aria-labelledby="charts-heading" className={styles.section}>
      <div className={styles.head}>
        <div>
          <h3 id="charts-heading" className={styles.heading}>
            Snapshot history
          </h3>
          <p className={styles.headingNote}>
            {days.length} daily snapshots, {days[0]!.date} to {days[days.length - 1]!.date}. Days without a snapshot are left as gaps.
          </p>
        </div>
        <div className={styles.ranges} role="group" aria-label="Time range">
          {RANGES.map((r) => (
            <button
              key={r}
              type="button"
              aria-pressed={range === r}
              disabled={!offered.includes(r)}
              title={offered.includes(r) ? undefined : `Needs ${RANGE_LABELS[r]} of snapshots; ${days.length} so far`}
              className={styles.range}
              onClick={() => setRange(r)}
            >
              {RANGE_LABELS[r]}
            </button>
          ))}
        </div>
      </div>
      <div className={styles.grid}>
        <TimeChart
          title="Total value rated"
          description="TVL of every rated protocol, from DefiLlama."
          series={[{ key: "tvl", label: "Total TVL", color: SERIES_COLOR }]}
          points={tvl}
          unit="usd"
        />
        <TimeChart
          title="Value by upgrade control"
          description="TVL grouped by the weakest control over each protocol's core programs."
          series={CONTROL_CLASSES.map((c) => ({ key: c, label: CONTROL_LABELS[c], color: CONTROL_COLORS[c] }))}
          points={value}
          unit="usd"
        />
        <TimeChart
          title="Stage distribution"
          description="Number of protocols at each Stage."
          series={STAGES.map((s) => ({ key: s, label: STAGE_LABELS[s], color: STAGE_COLORS[s] }))}
          points={stages}
          unit="count"
        />
      </div>
    </section>
  );
}

/** The charts bound to the URL's `range` (UI-1). Render inside <Suspense>. */
export function SummaryCharts({ days }: { days: SummaryDay[] }) {
  const [view, update] = useViewState();
  return <SummaryChartsView days={days} view={view} setRange={(range) => update((v) => ({ ...v, range }))} />;
}

/** The default range, pre-rendered into the static HTML as the Suspense fallback. */
export function SummaryChartsDefault({ days }: { days: SummaryDay[] }) {
  return <SummaryChartsView days={days} view={DEFAULT_VIEW} setRange={() => {}} />;
}
