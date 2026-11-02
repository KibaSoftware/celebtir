"use client";

import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import type { ReactNode } from "react";
import { CompareToggle } from "./compare/CompareToggle";
import { FactorStrip, FactorStripHeader, ProtocolLogo, StageBadge, StageSwatch } from "./ProtocolBits";
import { RatingBadge } from "./RatingBadge";
import { Tooltip } from "./Tooltip";
import { formatChange, formatChangeBp, formatUsd } from "@/lib/format";
import { CONTROL_COLORS, CONTROL_LABELS, FACTOR_LABELS, STAGE_LABELS } from "@/lib/labels";
import { applyView, CATEGORY_LABELS, categoryCounts, onChain, type ProtocolRow } from "@/lib/summary";
import type { Rating } from "@/lib/tokens";
import { CONTROL_CLASSES, FACTORS, type Factor, type Stage } from "@/lib/types";
import { useViewState } from "@/lib/useViewState";
import { DEFAULT_VIEW, TABS, type Tab, type ViewState } from "@/lib/viewState";
import { ScopeEmpty } from "./scope/ScopeEmpty";
import styles from "./ProtocolTable.module.css";

type Update = (next: (v: ViewState) => ViewState) => void;

interface Column {
  key: string;
  label: string;
  /** The direction a first click sorts in. */
  first: "asc" | "desc";
  numeric?: boolean;
  /** Always shown: the column picker cannot hide it. */
  fixed?: boolean;
}

export const COLUMNS: Column[] = [
  { key: "rank", label: "#", first: "asc", numeric: true },
  { key: "name", label: "Protocol", first: "asc", fixed: true },
  { key: "category", label: "Category", first: "asc" },
  { key: "risk", label: "Risk", first: "desc" },
  { key: "stage", label: "Stage", first: "desc" },
  { key: "controller", label: "Upgrade controller", first: "asc" },
  { key: "tvl", label: "TVL · 7d", first: "desc", numeric: true },
  { key: "activity", label: "Latest day volume or fees · 7-day total vs previous 7 days", first: "desc", numeric: true },
  { key: "last_change", label: "Last on-chain change", first: "desc" },
];

const STAGES: Stage[] = ["unrated", "stage_0", "stage_1", "stage_2"];
const RATINGS: Rating[] = ["low", "medium", "high", "unknown", "not_applicable"];

const toggle = <T,>(xs: T[], x: T) => (xs.includes(x) ? xs.filter((y) => y !== x) : [...xs, x]);

function Change({ text, value }: { text: string; value: number }) {
  return (
    <span className={styles.change} data-direction={value > 0 ? "up" : value < 0 ? "down" : "flat"}>
      {text}
    </span>
  );
}

function Cell({ col, row, logo }: { col: string; row: ProtocolRow; logo: string | null }) {
  switch (col) {
    case "rank":
      return <span className={styles.rank}>{row.rank}</span>;
    case "name":
      return (
        <span className={styles.name}>
          <CompareToggle id={row.id} name={row.name} />
          <ProtocolLogo name={row.name} logo={logo} size={30} />
          <Link href={`/protocols/${row.id}/`} className={styles.protocolLink}>
            {row.name}
          </Link>
        </span>
      );
    case "category":
      return <span className={styles.category}>{CATEGORY_LABELS[row.category as Tab]?.label ?? row.category}</span>;
    case "risk":
      if (!row.ratings || !row.reasons) return <Link href="/hyperevm/">Unassessed · native evidence</Link>;
      return <FactorStrip ratings={row.ratings} reasons={row.reasons} protocolId={row.id} />;
    case "stage":
      if (!row.stage) return <span>Unassessed</span>;
      return <StageBadge stage={row.stage} next={row.nextStage} missing={row.missing} inspection={row.inspection} />;
    case "controller":
      return <span className={styles.controller}>{row.controller}</span>;
    case "tvl":
      return row.tvl === null ? (
        <span className={styles.muted}>Unavailable</span>
      ) : (
        <span className={styles.money}>
          <span className={styles.tvl}>{formatUsd(row.tvl)}</span>
          {row.tvlChange7d !== null ? <Change text={formatChange(row.tvlChange7d)} value={row.tvlChange7d} /> : null}
        </span>
      );
    case "activity":
      return row.activity === null ? (
        <span className={styles.muted}>Unavailable</span>
      ) : (
        <span className={styles.money}>
          {formatUsd(row.activity.usd)}
          <span className={styles.kind}>{row.activity.kind === "volume" ? "volume" : "fees"}{row.activity.date ? ` · ${row.activity.date}` : ""}</span>
          {row.activity.changeBp !== null ? (
            <Change text={formatChangeBp(row.activity.changeBp)} value={row.activity.changeBp} />
          ) : null}
        </span>
      );
    case "last_change":
      return row.lastChange === null ? (
        <span className={styles.muted}>None recorded</span>
      ) : (
        <Tooltip label={`Last on-chain change ${row.lastChange.date}`} trigger={<span className={styles.date}>{row.lastChange.date}</span>}>
          {row.lastChange.summary}
        </Tooltip>
      );
    default:
      return null;
  }
}

function Checks<T extends string>({
  legend,
  options,
  labels,
  chosen,
  onToggle,
  swatch,
}: {
  legend: string;
  options: readonly T[];
  labels: (o: T) => ReactNode;
  chosen: readonly string[];
  onToggle: (o: T) => void;
  swatch?: (o: T) => ReactNode;
}) {
  return (
    <fieldset className={styles.fieldset}>
      <legend>{legend}</legend>
      {options.map((o) => (
        <label key={o} className={styles.check}>
          <input type="checkbox" checked={chosen.includes(o)} onChange={() => onToggle(o)} />
          {swatch?.(o)}
          {labels(o)}
        </label>
      ))}
    </fieldset>
  );
}

function activeFilters(v: ViewState): number {
  return (
    v.filters.stage.length +
    v.filters.controller.length +
    Object.values(v.filters.rating).reduce((n, xs) => n + (xs?.length ?? 0), 0)
  );
}

/** UI-9 to UI-11, driven by an explicit view state (pre-rendered with the default). */
export function ProtocolTableView({
  rows,
  counts,
  logos = {},
  view,
  update,
  live = false,
}: {
  rows: ProtocolRow[];
  counts: Record<Tab, number>;
  /** True once bound to the URL in the browser: the static fallback is inert. */
  live?: boolean;
  /** Logo path per protocol id, or null for the monogram placeholder. */
  logos?: Record<string, string | null>;
  view: ViewState;
  update: Update;
}) {
  const shown = applyView(rows, view);
  const inChain = onChain(rows, view.chain);
  // HUI-1: tab counts follow the chain filter.
  const tabCounts = view.chain === "all" ? counts : categoryCounts(onChain(rows, view.chain));
  const columns = COLUMNS.filter((c) => c.fixed || !view.hidden.includes(c.key));
  const filters = activeFilters(view);
  const setRating = (f: Factor, r: Rating) =>
    update((v) => ({
      ...v,
      filters: { ...v.filters, rating: { ...v.filters.rating, [f]: toggle(v.filters.rating[f] ?? [], r) } },
    }));

  return (
    <div className={styles.section} data-view={live ? "live" : "static"}>
      <div className={styles.toolbar}>
      <div className={styles.tabs} role="group" aria-label="Category">
        {TABS.map((t) => (
          <button
            key={t}
            type="button"
            className={styles.tab}
            aria-pressed={view.tab === t}
            onClick={() => update((v) => ({ ...v, tab: t }))}
          >
            {CATEGORY_LABELS[t].label} <span className={styles.count}>{tabCounts[t]}</span>
          </button>
        ))}
      </div>

      <div className={styles.controls}>
        <label className={styles.inline}>
          <span className="sr-only">Search</span>
          <input
            type="search"
            data-shortcut="search"
            className={styles.search}
            value={view.q}
            placeholder="Filter protocols"
            onChange={(e) => {
              const q = e.target.value;
              update((v) => ({ ...v, q }));
            }}
          />
        </label>
        <details className={styles.menu} data-shortcut="filters">
          <summary>Filters{filters ? ` (${filters})` : ""}</summary>
          <div className={styles.panel}>
            <Checks
              legend="Stage"
              options={STAGES}
              labels={(s) => STAGE_LABELS[s]}
              swatch={(s) => <StageSwatch stage={s} />}
              chosen={view.filters.stage}
              onToggle={(s) => update((v) => ({ ...v, filters: { ...v.filters, stage: toggle(v.filters.stage, s) } }))}
            />
            <Checks
              legend="Upgrade controller"
              options={CONTROL_CLASSES}
              labels={(c) => CONTROL_LABELS[c]}
              swatch={(c) => <span className={styles.swatch} style={{ background: CONTROL_COLORS[c] }} aria-hidden="true" />}
              chosen={view.filters.controller}
              onToggle={(c) =>
                update((v) => ({ ...v, filters: { ...v.filters, controller: toggle(v.filters.controller, c) } }))
              }
            />
            {FACTORS.map((f) => (
              <Checks
                key={f}
                legend={FACTOR_LABELS[f]}
                options={RATINGS}
                labels={(r) => <RatingBadge rating={r} />}
                chosen={view.filters.rating[f] ?? []}
                onToggle={(r) => setRating(f, r)}
              />
            ))}
            {filters ? (
              <button
                type="button"
                className={styles.clear}
                onClick={() => update((v) => ({ ...v, filters: DEFAULT_VIEW.filters }))}
              >
                Clear filters
              </button>
            ) : null}
          </div>
        </details>
        <details className={styles.menu}>
          <summary>Columns</summary>
          <div className={styles.panel}>
            <fieldset className={styles.fieldset}>
              <legend>Show columns</legend>
              {COLUMNS.filter((c) => !c.fixed).map((c) => (
                <label key={c.key} className={styles.check}>
                  <input
                    type="checkbox"
                    checked={!view.hidden.includes(c.key)}
                    onChange={() => update((v) => ({ ...v, hidden: toggle(v.hidden, c.key) }))}
                  />
                  {c.key === "rank" ? "Rank" : c.label}
                </label>
              ))}
            </fieldset>
          </div>
        </details>
      </div>
      </div>
      <p className={styles.definition}>{CATEGORY_LABELS[view.tab].definition}</p>

      <div className={styles.scroll} role="region" aria-label="Protocol leaderboard" tabIndex={0}>
        <table className={styles.table}>
          <caption className={styles.srOnly}>
            {shown.length} of {inChain.length} protocols, sorted by {COLUMNS.find((c) => c.key === view.sort.key)?.label ?? view.sort.key}{" "}
            {view.sort.dir === "asc" ? "ascending" : "descending"}
          </caption>
          <thead>
            <tr>
              {columns.map((c) => {
                const active = view.sort.key === c.key;
                return (
                  <th
                    key={c.key}
                    scope="col"
                    data-label={c.label}
                    data-col={c.key}
                    className={c.numeric ? styles.numeric : undefined}
                    aria-sort={active ? (view.sort.dir === "asc" ? "ascending" : "descending") : undefined}
                  >
                    <button
                      type="button"
                      className={styles.sort}
                      onClick={() =>
                        update((v) => ({
                          ...v,
                          sort: {
                            key: c.key,
                            dir: active ? (v.sort.dir === "asc" ? "desc" : "asc") : c.first,
                          },
                        }))
                      }
                    >
                      {c.key === "rank" ? <span aria-label="Rank">#</span> : c.label}
                      <span className={styles.arrow} aria-hidden="true">
                        {active ? (view.sort.dir === "asc" ? "▲" : "▼") : "↕"}
                      </span>
                    </button>
                    {c.key === "risk" ? <FactorStripHeader /> : null}
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {shown.map((r) => (
              <tr key={r.id} data-protocol={r.id}>
                {columns.map((c) =>
                  c.key === "name" ? (
                    <th key={c.key} scope="row">
                      <Cell col={c.key} row={r} logo={logos[r.id] ?? null} />
                    </th>
                  ) : (
                    <td key={c.key} data-col={c.key} className={c.numeric ? styles.numeric : undefined}>
                      <Cell col={c.key} row={r} logo={logos[r.id] ?? null} />
                    </td>
                  ),
                )}
              </tr>
            ))}
            {shown.length === 0 ? (
              <tr>
                <td colSpan={columns.length} className={styles.empty}>
                  {inChain.length === 0 ? (
                    <ScopeEmpty chain={view.chain} kind="protocols" />
                  ) : (
                    <>
                      No protocols match these filters.{" "}
                      {/* Resets the filters, keeps the chain (HUI-1). */}
                      <button
                        type="button"
                        className={styles.clear}
                        onClick={() => update(() => ({ ...DEFAULT_VIEW, chain: view.chain, sort: view.sort, hidden: view.hidden, range: view.range }))}
                      >
                        Show all
                      </button>
                    </>
                  )}
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
      <p className={styles.legend}>
        <span>Ratings:</span>
        {RATINGS.map((r) => (
          <span key={r} className={styles.legendItem}>
            <RatingBadge rating={r} />
          </span>
        ))}
        <span className={styles.legendCodes}>
          Factors: UC upgrade control · UD upgrade delay · AP admin powers · CT code transparency · OD oracle dependency · DP dependencies
        </span>
      </p>
    </div>
  );
}

/** The table bound to the URL (UI-1). Render inside <Suspense>. */
export function ProtocolTable(props: { rows: ProtocolRow[]; counts: Record<Tab, number>; logos?: Record<string, string | null> }) {
  const [view, update] = useViewState();
  return <ProtocolTableView {...props} view={view} update={update} live />;
}

/** The default view, pre-rendered into the static HTML as the Suspense fallback. */
export function ProtocolTableDefault(props: { rows: ProtocolRow[]; counts: Record<Tab, number>; logos?: Record<string, string | null> }) {
  return <ProtocolTableView {...props} view={DEFAULT_VIEW} update={() => {}} />;
}
