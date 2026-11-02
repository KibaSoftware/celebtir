import { SortableTable } from "@/components/research/SortableTable";
import { HistoryCoverage } from "@/components/HistoryCoverage";
import type { Metadata } from "next";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { Suspense } from "react";
import { RangedCharts, RangedChartsDefault, type ChartSpec } from "@/components/charts/RangedCharts";
import { CompareToggle } from "@/components/compare/CompareToggle";
import styles from "@/components/page/Page.module.css";
import { ProtocolLogo } from "@/components/ProtocolBits";
import { Tooltip } from "@/components/Tooltip";
import { Sparkline } from "@/components/ui/Charts";
import { EmptyState, MetricCard, PageHeader, SectionHeader } from "@/components/ui/Ui";
import { loadSnapshot } from "@/lib/data";
import { formatChangeBp, formatUsd } from "@/lib/format";
import { SERIES_COLOR } from "@/lib/labels";
import { logoMap } from "@/lib/logos";
import { activityRows, activitySeries, type DisplayActivity } from "@/lib/pages";
import local from "./activity.module.css";
import { ChainScoped } from "@/components/scope/ChainScoped";
import { ScopeEmpty } from "@/components/scope/ScopeEmpty";
import { scope, SCOPES } from "@/lib/scope";
import type { ChainFilter } from "@/lib/viewState";

export const metadata: Metadata = {
  title: "Activity",
  description: "Daily volume and fees per covered protocol, with the 7-day total against the previous 7 days.",
};

/** One metric cell: value, the 7-day total against the previous 7 days (unavailable over a gap day, D5), and the daily line. Unavailable is never shown as zero. */
function Cell({ a, label }: { a: DisplayActivity; label: string }) {
  if (a.kind === "not_applicable") return <span title={a.reason}>Not applicable</span>;
  if (a.kind === "unavailable") {
    return (
      <Tooltip label="Unavailable" trigger={<span className={local.unavailable}>Unavailable</span>}>
        {a.reason}
      </Tooltip>
    );
  }
  return (
    <span className={local.metric}>
      <span className={local.value}>{formatUsd(a.latest)}</span>
      <span className={local.change} title={a.change_7d_bp === null && a.gap_days?.length ? `Unavailable: missing or near-zero days ${a.gap_days.join(", ")}` : undefined} data-direction={a.change_7d_bp === null ? "none" : a.change_7d_bp > 0 ? "up" : a.change_7d_bp < 0 ? "down" : "flat"}>
        {a.change_7d_bp !== null ? formatChangeBp(a.change_7d_bp) : "–"}
      </span>
      <Sparkline days={a.days} label={label} />
    </span>
  );
}

function ActivityPageBody({ chain }: { chain: ChainFilter }) {
  const s = scope(loadSnapshot(), chain);
  const rows = activityRows(s);
  const logos = logoMap(rows.map((r) => r.id));
  const volume = activitySeries(s, "volume");
  const fees = activitySeries(s, "fees");
  const reportingFees = rows.filter((r) => r.fees.kind === "usd").length;
  const reportingVolume = rows.filter((r) => r.volume.kind === "usd").length;
  const lastFees = fees[fees.length - 1];
  const lastVolume = volume[volume.length - 1];
  const charts: ChartSpec[] = [
    ...(volume.length >= 2
      ? [
          {
            title: "Volume",
            description: `Daily volume summed over the ${reportingVolume} of ${rows.length} protocols reporting volume, from DefiLlama.`,
            series: [{ key: "usd", label: "Volume", color: SERIES_COLOR }],
            points: volume.map((p) => ({ date: p.date, values: { usd: p.usd } })),
            unit: "usd" as const,
          },
        ]
      : []),
    ...(fees.length >= 2
      ? [
          {
            title: "Fees",
            description: `Daily fees summed over the ${reportingFees} of ${rows.length} protocols reporting fees, from DefiLlama.`,
            series: [{ key: "usd", label: "Fees", color: SERIES_COLOR }],
            points: fees.map((p) => ({ date: p.date, values: { usd: p.usd } })),
            unit: "usd" as const,
          },
        ]
      : []),
  ];

  return (
    <>
      {rows.length === 0 ? <ScopeEmpty chain={chain} kind="activity" /> : null}
      <PageHeader art="activity"
        eyebrow="Market activity"
        title="Activity"
        subtitle="Daily volume and fees from DefiLlama. Perpetuals volume needs a paid plan, so perps show fees only. Activity is context: it never changes a rating."
      />

      <HistoryCoverage snapshot={s} chain={chain} />
      <div className={styles.kpis}>
        <MetricCard
          title="Daily volume"
          value={lastVolume ? formatUsd(lastVolume.usd) : "–"}
          context={lastVolume ? `${lastVolume.date}, summed over ${reportingVolume} of ${rows.length} protocols reporting volume` : "No covered protocol reports volume"}
          help="Not an ecosystem total: only protocols whose volume DefiLlama reports on the free plan are summed."
        />
        <MetricCard
          title="Daily fees"
          value={lastFees ? formatUsd(lastFees.usd) : "–"}
          context={lastFees ? `${lastFees.date}, summed over ${reportingFees} of ${rows.length} protocols reporting fees` : "No covered protocol reports fees"}
          help="Not an ecosystem total: only protocols whose fees DefiLlama reports are summed."
        />
        <MetricCard
          title="Volume coverage"
          value={
            <>
              {reportingVolume} <span className={local.of}>of {rows.length}</span>
            </>
          }
          context="Protocols reporting volume on the displayed completed day"
          help="Coverage counts observations for the displayed date. Missing data is excluded; liquid-staking volume is not applicable."
        />
        <MetricCard
          title="Fee coverage"
          value={
            <>
              {reportingFees} <span className={local.of}>of {rows.length}</span>
            </>
          }
          context="Protocols reporting fees on the displayed completed day"
          help="Coverage counts protocols with a fee observation for the displayed date; missing observations are not zero."
        />
      </div>

      <section className={styles.section} aria-labelledby="charts-h">
        <SectionHeader id="charts-h" title="Daily activity" note="Unit: US dollars per day. A missing day is a gap, not a zero." />
        {charts.length ? (
          <Suspense fallback={<RangedChartsDefault charts={charts} label="Daily activity" />}>
            <RangedCharts charts={charts} label="Daily activity" />
          </Suspense>
        ) : null}
        {volume.length < 2 ? (
          <div className={local.gap}>
            <EmptyState kind={volume.length === 0 ? "coverage" : "history"} title={volume.length === 0 ? "No volume reported" : undefined}>
              <p>{volume.length === 0 ? "No rated protocol reports volume to DefiLlama on the free plan." : "Volume: fewer than two days reported so far."}</p>
            </EmptyState>
          </div>
        ) : null}
        {fees.length < 2 ? (
          <div className={local.gap}>
            <EmptyState kind={fees.length === 0 ? "coverage" : "history"} title={fees.length === 0 ? "No fees reported" : undefined}>
              <p>{fees.length === 0 ? "No rated protocol reports fees to DefiLlama." : "Fees: fewer than two days reported so far."}</p>
            </EmptyState>
          </div>
        ) : null}
      </section>

      <section className={styles.section} aria-labelledby="by-h">
        <SectionHeader id="by-h" title="By protocol" note="The same completed day as each KPI, the 7-day total against the previous 7 days (shown as – when either week has a missing or near-zero day), and the daily line." />
        <div className={styles.scroll}>
          <SortableTable label="Protocol activity" defaultColumn={1} values={Object.fromEntries(rows.map(r=>[r.id,[r.name,r.volume.kind === "usd" ? r.volume.latest : null,r.fees.kind === "usd" ? r.fees.latest : null]]))}><table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Protocol</th>
                <th scope="col">Volume on {lastVolume?.date ?? "the latest day"} · 7-day total vs previous 7 days</th>
                <th scope="col">Fees on {lastFees?.date ?? "the latest day"} · 7-day total vs previous 7 days</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <tr key={r.id} data-protocol={r.id} data-sort-id={r.id}>
                  <th scope="row">
                    <span className={styles.nameCell}>
                      <CompareToggle id={r.id} name={r.name} />
                      <ProtocolLogo name={r.name} logo={logos[r.id] ?? null} size={26} />
                      <Link href={`/protocols/${r.id}/#activity`}>{r.name}</Link>
                    </span>
                  </th>
                  <td>
                    <Cell a={r.volume} label={`${r.name} daily volume`} />
                  </td>
                  <td>
                    <Cell a={r.fees} label={`${r.name} daily fees`} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table></SortableTable>
        </div>
      </section>
    </>
  );
}

// HUI-1: rendered once per chain; the picker chooses the variant.
export default function ActivityPage() {
  const variants = Object.fromEntries(SCOPES.map((c) => [c, <ActivityPageBody key={c} chain={c} />])) as Record<ChainFilter, React.ReactNode>;
  return <ChainScoped variants={variants} />;
}
