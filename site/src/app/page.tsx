import { FactorDistribution } from "@/components/research/FactorDistribution";
import { Finding } from "@/components/research/Finding";
import { HistoryCoverage } from "@/components/HistoryCoverage";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { Suspense } from "react";
import { SummaryCharts, SummaryChartsDefault } from "@/components/charts/SummaryCharts";
import { RatingBadge } from "@/components/RatingBadge";
import { ProtocolTable, ProtocolTableDefault } from "@/components/ProtocolTable";
import { ChainScoped } from "@/components/scope/ChainScoped";
import { ScopeEmpty } from "@/components/scope/ScopeEmpty";
import { StackedBar } from "@/components/ui/Charts";
import { SectionHeader } from "@/components/ui/Ui";
import { freshness, loadHistory, loadSnapshot } from "@/lib/data";
import { formatUsd } from "@/lib/format";
import { CONTROL_COLORS, CONTROL_LABELS, STAGE_COLORS, STAGE_LABELS, STAGE_NOTE } from "@/lib/labels";
import { logoMap } from "@/lib/logos";
import { controlShares, insights, kpis, STAGES, stageDistribution } from "@/lib/overview";
import { CHAIN_LABELS, scope, scopeDays, scopeEvents, SCOPES, withChain } from "@/lib/scope";
import { assessedRows as ratedRows, buildRows, categoryCounts } from "@/lib/summary";
import { type ChangeEvent, type ProtocolHistoryFile, type Snapshot, type SummaryDay } from "@/lib/types";
import type { ChainFilter } from "@/lib/viewState";
import styles from "./page.module.css";

// UI-8 to UI-11. Everything is computed at build time; the client components
// only re-filter and re-sort. The Suspense fallbacks are the default view, so
// the static HTML carries the full table and charts without JavaScript.
interface Inputs {
  snapshot: Snapshot;
  histories: Record<string, ProtocolHistoryFile | undefined>;
  events: ChangeEvent[];
  days: SummaryDay[];
  date: string;
}

/** HUI-1: one chain's view of the snapshot, for every aggregator below. */
function scoped(i: Inputs, chain: ChainFilter) {
  const snapshot = scope(i.snapshot, chain);
  const events = scopeEvents(i.events, i.snapshot, chain);
  const days = scopeDays(i.days, chain);
  const rows = buildRows(snapshot, i.histories, events);
  return { snapshot, events, days, rows };
}

function Overview({ inputs, chain }: { inputs: Inputs; chain: ChainFilter }) {
  const { snapshot, events, days, rows } = scoped(inputs, chain);
  const k = kpis(rows, snapshot.scores.value_by_control, days, events, inputs.date);
  const observations = days.filter(d => d.protocols > 0);
  // F5: every rating figure below counts assessed protocols only.
  const assessedRows = ratedRows(rows);
  const selected = assessedRows[0];
  if (!rows.length) return <><h1>{CHAIN_LABELS[chain]}</h1><ScopeEmpty chain={chain} kind="protocols" /></>;
  return <>
    <h1 className={styles.pageTitle}>Summary</h1>
    <p className={styles.coverage}>{CHAIN_LABELS[chain]} · {k.assessed} assessed protocols{k.unassessed ? ` · ${k.unassessed} listed unassessed` : ""} · {inputs.date} · methodology v{snapshot.scores.methodology_version}</p>
    <div className={styles.overviewTop}>
      <section className={styles.metricPanel} aria-label="Recent observations">
        <h2>Recent observations</h2>
        <p className={styles.figure}>{observations.length < 2 && !k.recentChanges ? "Insufficient history" : k.recentChanges}</p>
        <p>Recorded on-chain changes · {k.changeWindowDays}-day window ending {inputs.date}.</p>
        <p className={styles.cardNote}>{observations.length} Celebtir snapshots. Collection gaps do not establish an absence of changes.</p>
        <Link href="/changelog/">Inspect observation coverage ↗</Link>
      </section>
      <section className={styles.metricPanel} aria-label="Reported TVL and control">
        <h2>Reported TVL and control</h2>
        <div className={styles.figures}><span className={styles.figure}>{formatUsd(k.totalTvl)}</span><span className={styles.share}>{k.undelayedShare === null ? "Unavailable" : (k.undelayedShare * 100).toFixed(1) + "%"}</span></div>
        <p>of assessed TVL without a 24 h+ upgrade delay or immutability</p>
        <StackedBar compact label="Value by upgrade control, share of analyzed TVL" unit="share" format={formatUsd} segments={controlShares(snapshot.scores.value_by_control.all).map(({cls, usd}) => ({key:cls,label:CONTROL_LABELS[cls],value:usd,color:CONTROL_COLORS[cls],href:withChain('/?controller='+cls+'#protocols-heading',chain)}))} />
        <p className={styles.cardNote}>DefiLlama · {k.assessed-k.tvlExcluded}/{k.assessed} reporting. Deposits may overlap; TVL is not unique economic value. Unknown control is not proof of no delay.</p>
        <Link href="/value/">Explore value by control ↗</Link>
      </section>
      <section className={styles.metricPanel} aria-label="Governance overview">
        <h2>Governance overview</h2>
        <p className={styles.figure}>{assessedRows.filter(r=>r.ratings.upgrade_delay === "high").length} / {assessedRows.length}</p>
        <p>High upgrade-delay ratings · assessed protocols</p><p className={styles.cardNote}>Six independent factors; no composite security score. HyperEVM protocols are listed unassessed, with their contract facts on the HyperEVM page.</p>
        <Link href="/risk/">Explore risk assessments ↗</Link>
      </section>
    </div>
    <div className={styles.researchRow}>
      <FactorDistribution rows={rows} />
      {selected ? <section className={styles.selected}>
        <h2>Selected assessment</h2><h3>{selected.name} / Upgrade delay</h3>
        <Link href={'/risk/?detail='+selected.id+'.upgrade_delay'}><RatingBadge rating={selected.ratings.upgrade_delay} /></Link>
        <div><Finding text={selected.reasons.upgrade_delay} /></div>
        <p>Effective controller: {selected.controller}</p>
        <p>To improve: {snapshot.scores.protocols[selected.id].factors.upgrade_delay.next_step}</p>
        <Link href={'/risk/?detail='+selected.id+'.upgrade_delay'}>Criteria, evidence and record slot ↗</Link>
      </section> : null}
    </div>
    <details className={styles.secondary}><summary>Assessment history and Stage distribution</summary>
      <HistoryCoverage snapshot={snapshot} chain={chain} />
      <StackedBar label="Assessed protocols by Stage, number of protocols" unit="count" segments={STAGES.map(stage=>({key:stage,label:STAGE_LABELS[stage],value:stageDistribution(assessedRows)[stage],color:STAGE_COLORS[stage]}))} />
      <p>{STAGE_NOTE}</p>
      <p>Stage distribution reflects the selected methodology and available evidence; inspect each protocol&apos;s requirements for context.</p>
      <Suspense fallback={<SummaryChartsDefault days={days} />}><SummaryCharts days={days} /></Suspense>
    </details>
  </>;
}

function Insights({ inputs, chain }: { inputs: Inputs; chain: ChainFilter }) {
  const { events, rows } = scoped(inputs, chain);
  const names = Object.fromEntries(rows.map((r) => [r.id, r.name]));
  const allNotes = insights(rows, events, inputs.date, names);
  const notes = allNotes.filter(n => n.items.length > 0).slice(0, 3);
  return (
    <div className={styles.insights}>
      {!allNotes.find(n => n.key === "changes")?.items.length ? <p className={styles.quiet}>No on-chain change records available for the 30-day window ending {inputs.date}. <Link href="/changelog/">Observation history</Link></p> : null}
      {notes.map((n) => (
        <article key={n.key} className={styles.insight} aria-labelledby={`insight-${chain}-${n.key}`}>
          <h3 id={`insight-${chain}-${n.key}`} className={styles.insightTitle}>
            {n.title}
            <span className={styles.insightCount}>{n.items.length}</span>
          </h3>
          <p className={styles.insightRule}>{n.rule}</p>

          {n.items.length ? (
            <ul className={styles.insightList}>
              {n.items.slice(0, 3).map((i) => (
                <li key={i.id + i.detail}>
                  <Link href={i.href}>{i.name}</Link>
                  <div className={styles.insightDetail}><Finding text={i.detail} /></div>
                </li>
              ))}
            </ul>
          ) : (
            <p className={styles.insightNone}>No protocol matches in this snapshot.</p>
          )}
          {n.items.length > 3 ? <details><summary>See all {n.items.length} findings</summary><ul>{n.items.slice(3).map(i => <li key={i.id + i.detail}><Link href={i.href}>{i.name}</Link><Finding text={i.detail} /></li>)}</ul></details> : null}
        </article>
      ))}
    </div>
  );
}

const variants = (render: (chain: ChainFilter) => React.ReactNode) =>
  Object.fromEntries(SCOPES.map((c) => [c, render(c)])) as Record<ChainFilter, React.ReactNode>;

export default function SummaryPage() {
  const snapshot = loadSnapshot();
  const ids = Object.keys(snapshot.facts.protocols);
  const histories = Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)]));
  const events = loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [];
  const days = loadHistory<{ days: SummaryDay[] }>("summary")?.days ?? [];
  const date = freshness(snapshot).date;
  const inputs: Inputs = { snapshot, histories, events, days, date };
  // The leaderboard filters by chain on the client (applyView); its counts follow.
  const rows = buildRows(snapshot, histories, events);
  const counts = categoryCounts(rows);
  const logos = logoMap(ids);

  return (
    <>
      <ChainScoped variants={variants((c) => <Overview inputs={inputs} chain={c} />)} />

      <section aria-labelledby="protocols-heading" className={styles.section}>
        <SectionHeader
          id="protocols-heading"
          title="Protocol leaderboard"
          note="Ranked by TVL. TVL is context and never changes a rating. Focus any factor cell for its rating and reason."
        />
        <Suspense fallback={<ProtocolTableDefault rows={rows} counts={counts} logos={logos} />}>
          <ProtocolTable rows={rows} counts={counts} logos={logos} />
        </Suspense>
      </section>

      <section aria-labelledby="insights-heading" className={styles.section}>
        <SectionHeader
          id="insights-heading"
          title="Ecosystem insights"
          note="Rules over the published ratings and events, not editorial judgement. Each rule is stated; follow an item to its evidence."
        />
        <ChainScoped variants={variants((c) => <Insights inputs={inputs} chain={c} />)} />
      </section>
    </>
  );
}
