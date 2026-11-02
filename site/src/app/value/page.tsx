import { SortableTable } from "@/components/research/SortableTable";
import { loadMethodology } from "@/lib/methodology";
import { formatDuration } from "@/lib/format";
import { HistoryCoverage } from "@/components/HistoryCoverage";
import type { Metadata } from "next";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { Suspense } from "react";
import { RangedCharts, RangedChartsDefault, type ChartSpec } from "@/components/charts/RangedCharts";
import { CompareToggle } from "@/components/compare/CompareToggle";
import styles from "@/components/page/Page.module.css";
import { ProtocolLogo } from "@/components/ProtocolBits";
import { Termed } from "@/components/Term";
import { ShareBar, StackedBar } from "@/components/ui/Charts";
import { Card, EmptyState, MetricCard, PageHeader, SectionHeader } from "@/components/ui/Ui";
import { loadHistory, loadSnapshot } from "@/lib/data";
import { formatChange, formatUsd } from "@/lib/format";
import { CONTROL_COLORS, CONTROL_DEFINITIONS, CONTROL_LABELS, SERIES_COLOR, SERIES_COLORS } from "@/lib/labels";
import { logoMap } from "@/lib/logos";
import { controlShares } from "@/lib/overview";
import { assetTotals } from "@/lib/pages";
import { buildRows, CATEGORY_LABELS } from "@/lib/summary";
import { CONTROL_CLASSES, type ChangeEvent, type ProtocolHistoryFile, type SummaryDay } from "@/lib/types";
import type { Tab } from "@/lib/viewState";
import local from "./value.module.css";
import { ChainScoped } from "@/components/scope/ChainScoped";
import { ScopeEmpty } from "@/components/scope/ScopeEmpty";
import { scope, scopeDays, scopeEvents, SCOPES } from "@/lib/scope";
import type { ChainFilter } from "@/lib/viewState";

export const metadata: Metadata = {
  title: "Value",
  description: "Where the money sits: TVL per protocol, value by upgrade control, and asset groups.",
};

const share = (part: number, total: number) => (total > 0 ? `${((part / total) * 100).toFixed(1)}%` : "–");

function ValuePageBody({ chain }: { chain: ChainFilter }) {
  const s = scope(loadSnapshot(), chain);
  const ids = Object.keys(s.facts.protocols);
  const rows = buildRows(
    s,
    Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])),
    scopeEvents(loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [], loadSnapshot(), chain),
  ).sort((a, b) => a.rank - b.rank);
  const days = scopeDays(loadHistory<{ days: SummaryDay[] }>("summary")?.days ?? [], chain);
  const vbc = s.scores.value_by_control;
  const total = vbc.all.total;
  const assets = assetTotals(s);
  const logos = logoMap(ids);
  const largest = rows.find((r) => r.tvl !== null) ?? null;
  const delayed = vbc.all.classes.multisig_with_delay ?? 0;
  const short = vbc.all.classes.multisig_short_delay ?? 0;
  const undelayed = vbc.all.classes.multisig_without_delay ?? 0;
  // UI-6: glossary terms link at their first use on the page.
  const seen = new Set<string>();
  const charts: ChartSpec[] = [
    {
      title: "Value under analyzed protocol control",
      description: "Total TVL of every assessed protocol, from DefiLlama, one point per daily snapshot.",
      series: [{ key: "tvl", label: "Total TVL", color: SERIES_COLOR }],
      points: days.map((d) => ({ date: d.date, values: { tvl: d.total_tvl_usd } })),
      unit: "usd",
    },
    {
      title: "Value by upgrade control",
      description: "TVL grouped by the weakest control over each protocol's core programs.",
      series: CONTROL_CLASSES.map((c) => ({ key: c, label: CONTROL_LABELS[c], color: CONTROL_COLORS[c] })),
      points: days.map((d) => ({ date: d.date, values: d.value_by_control.classes })),
      unit: "usd",
    },
  ];
  const categories = [["all", vbc.all] as const, ...Object.entries(vbc.by_category)];
  const tvlTable = (
    <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">#</th>
                <th scope="col">Protocol</th>
                <th scope="col" className={styles.num}>
                  TVL
                </th>
                <th scope="col">Share</th>
                <th scope="col" className={styles.num}>
                  7d
                </th>
                <th scope="col">Upgrade control</th>
                <th scope="col">Controller</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <tr key={r.id} data-sort-id={r.id}>
                  <td className={`${styles.num} ${styles.muted}`}>{r.rank}</td>
                  <th scope="row">
                    <span className={styles.nameCell}>
                      <CompareToggle id={r.id} name={r.name} />
                      <ProtocolLogo name={r.name} logo={logos[r.id] ?? null} size={26} />
                      <Link href={`/protocols/${r.id}/#value`}>{r.name}</Link>
                    </span>
                  </th>
                  <td className={styles.num}>
                    {r.tvl === null ? <span className={styles.muted}>Unavailable</span> : <strong className={local.tvl}>{formatUsd(r.tvl)}</strong>}
                  </td>
                  <td className={local.shareCell}>
                    {r.tvl === null || total === 0 ? (
                      <span className={styles.muted}>–</span>
                    ) : (
                      <span className={local.share}>
                        <ShareBar fraction={r.tvl / total} label={`${r.name}, share of analyzed TVL`} />
                        <span className={styles.num}>{share(r.tvl, total)}</span>
                      </span>
                    )}
                  </td>
                  <td className={styles.num}>{r.tvlChange7d === null ? <span className={styles.muted}>–</span> : formatChange(r.tvlChange7d)}</td>
                  <td>
                    {r.controlClass ? (
                      <span className={local.control}>
                        <span className={local.swatch} style={{ background: CONTROL_COLORS[r.controlClass] }} aria-hidden="true" />
                        {CONTROL_LABELS[r.controlClass]}
                      </span>
                    ) : (
                      <span className={styles.muted}>Unassessed</span>
                    )}
                  </td>
                  <td className={styles.muted}>{r.controller}</td>
                </tr>
              ))}
            </tbody>
          </table>
  );

  return (
    <>
      {rows.length === 0 ? <ScopeEmpty chain={chain} kind="value" /> : null}
      <PageHeader art="value"
        eyebrow="Economic exposure"
        title="Value"
        subtitle={
          <Termed
            text="Total value locked in each rated protocol, from DefiLlama, grouped by who can upgrade the code. TVL is context: it never changes a rating."
            seen={seen}
          />
        }
      />

      <HistoryCoverage snapshot={s} chain={chain} />
      <div className={styles.kpis}>
        <MetricCard
          title="Total analyzed TVL"
          value={formatUsd(total)}
          context={vbc.excluded.length ? `${vbc.excluded.length} protocol${vbc.excluded.length === 1 ? "" : "s"} without DefiLlama TVL left out` : "Every rated protocol reports TVL"}
          help="Sum of DefiLlama TVL over the rated protocols."
        />
        <MetricCard
          title="Largest protocol by TVL"
          help="The largest reported protocol TVL in this chain scope; missing TVL is excluded."
          value={largest ? formatUsd(largest.tvl!) : "–"}
          context={largest ? `${largest.name} · ${share(largest.tvl!, total)} of analyzed TVL` : "No protocol reports TVL"}
          href={largest ? `/protocols/${largest.id}/#value` : undefined}
          hrefLabel={largest ? largest.name : undefined}
        />
        <MetricCard
          title="TVL under a 24 h+ delay"
          value={formatUsd(delayed)}
          context={`${share(delayed, total)} of analyzed TVL: upgrades wait at least ${formatDuration(loadMethodology(s.scores.methodology_version).upgrade_delay.medium_min_secs)}`}
          help={s.scores.methodology_version === "0.1" ? "v0.1: a positive time lock counts as delayed." : CONTROL_DEFINITIONS.multisig_with_delay}
        />
        <MetricCard
          title="TVL under a delay below 24 h"
          value={formatUsd(short)}
          context={`${share(short, total)} of analyzed TVL: a time lock above zero but under 24 hours`}
          help={CONTROL_DEFINITIONS.multisig_short_delay}
        />
        <MetricCard
          title="TVL under no-delay multisigs"
          value={formatUsd(undelayed)}
          context={`${share(undelayed, total)} of analyzed TVL: upgrades can apply as soon as signed`}
          help={CONTROL_DEFINITIONS.multisig_without_delay}
        />
      </div>

      <section className={styles.section} aria-labelledby="history-h">
        <SectionHeader id="history-h" title="Value over time" note="Unit: US dollars of TVL. Source: DefiLlama, read with each daily snapshot." />
        {days.length >= 2 ? (
          <Suspense fallback={<RangedChartsDefault charts={charts} label="Value over time" />}>
            <RangedCharts charts={charts} label="Value over time" />
          </Suspense>
        ) : (
          <EmptyState kind="history">
            <p>Charts appear once two daily snapshots exist. The figures on this page are from the current snapshot.</p>
          </EmptyState>
        )}
      </section>

      <section className={styles.section} aria-labelledby="exposure-h">
        <SectionHeader id="exposure-h" title="Upgrade-control exposure" note="Unit: share of each category's analyzed TVL. Weakest control first." />
        <Card>
          <ul className={local.bars}>
            {categories.map(([cat, row]) => (
              <li key={cat}>
                <div className={local.barHead}>
                  <span className={local.barName}>{CATEGORY_LABELS[cat as Tab]?.label ?? cat}</span>
                  <span className={local.barTotal}>{formatUsd(row.total)}</span>
                </div>
                <StackedBar
                  compact={cat !== "all"}
                  unit="share"
                  format={formatUsd}
                  label={`${CATEGORY_LABELS[cat as Tab]?.label ?? cat}: TVL by upgrade control`}
                  segments={controlShares(row).map(({ cls, usd }) => ({ key: cls, label: CONTROL_LABELS[cls], value: usd, color: CONTROL_COLORS[cls] }))}
                />
              </li>
            ))}
          </ul>
        </Card>
        <dl className={local.definitions} aria-label="Upgrade-control classes">
          {CONTROL_CLASSES.map(c => <div key={c}><dt>{CONTROL_LABELS[c]}</dt><dd>{CONTROL_DEFINITIONS[c]}</dd></div>)}
        </dl>
        <h3 className={local.tableTitle}>Value by upgrade control, per category</h3>
        <div className={styles.scroll}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Category</th>
                {CONTROL_CLASSES.map((c) => (
                  <th key={c} scope="col" className={styles.num}>
                    {CONTROL_LABELS[c]}
                  </th>
                ))}
                <th scope="col" className={styles.num}>
                  Total
                </th>
              </tr>
            </thead>
            <tbody>
              {categories.map(([cat, row]) => (
                <tr key={cat}>
                  <th scope="row">{CATEGORY_LABELS[cat as Tab]?.label ?? cat}</th>
                  {CONTROL_CLASSES.map((c) => (
                    <td key={c} className={styles.num}>
                      {formatUsd(row.classes[c] ?? 0)} <span className={styles.muted}>{share(row.classes[c] ?? 0, row.total)}</span>
                    </td>
                  ))}
                  <td className={styles.num}>{formatUsd(row.total)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {vbc.excluded.length ? (
          <p className={styles.hint}>
            Left out because DefiLlama reported no TVL: {vbc.excluded.map((id) => s.facts.protocols[id]?.name ?? id).join(", ")}.
          </p>
        ) : null}
      </section>

      <section className={styles.section} aria-labelledby="tvl-h">
        <SectionHeader id="tvl-h" title="TVL by protocol" note="Ranked by TVL. Share is of analyzed TVL; the 7-day change appears once a week of history exists." />
        <div className={styles.scroll}>
          <SortableTable label="Protocol value" defaultColumn={2} values={Object.fromEntries(rows.map(r=>[r.id,[r.rank,r.name,r.tvl,r.tvl,r.tvlChange7d,r.controlClass ? CONTROL_LABELS[r.controlClass] : "Unassessed",r.controller]]))}>{tvlTable}</SortableTable>
        </div>
      </section>

      <section className={styles.section} aria-labelledby="assets-h">
        <SectionHeader id="assets-h" title="Asset groups" note="Unit: US dollars and share of the protocols whose asset groups DefiLlama reports." />
        {assets.total > 0 ? (
          <>
            <Card>
              <StackedBar
                unit="share"
                format={formatUsd}
                label="Asset groups, share of reported value"
                segments={[
                  { key: "stablecoins", label: "Stablecoins", value: assets.stablecoins, color: SERIES_COLORS[0] },
                  { key: "sol", label: "SOL and staked SOL", value: assets.sol, color: SERIES_COLORS[1] },
                  { key: "btc", label: "BTC", value: assets.btc, color: SERIES_COLORS[2] },
                  { key: "other", label: "Other", value: assets.other, color: SERIES_COLORS[3] },
                ]}
              />
            </Card>
            <div className={styles.scroll} style={{ marginTop: 16 }}>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th scope="col">Group</th>
                    <th scope="col" className={styles.num}>
                      Value
                    </th>
                    <th scope="col" className={styles.num}>
                      Share
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {(
                    [
                      ["Stablecoins", assets.stablecoins],
                      ["SOL and staked SOL", assets.sol],
                      ["BTC", assets.btc],
                      ["Other", assets.other],
                    ] as const
                  ).map(([label, usd]) => (
                    <tr key={label}>
                      <th scope="row">{label}</th>
                      <td className={styles.num}>{formatUsd(usd)}</td>
                      <td className={styles.num}>{share(usd, assets.total)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        ) : (
          <EmptyState kind="coverage">
            <p>DefiLlama returned no asset groups for any rated protocol.</p>
          </EmptyState>
        )}
        {assets.missing.length ? <p className={styles.hint}>No asset groups for: {assets.missing.join(", ")}.</p> : null}
      </section>
    </>
  );
}

// HUI-1: rendered once per chain; the picker chooses the variant.
export default function ValuePage() {
  const variants = Object.fromEntries(SCOPES.map((c) => [c, <ValuePageBody key={c} chain={c} />])) as Record<ChainFilter, React.ReactNode>;
  return <ChainScoped variants={variants} />;
}
