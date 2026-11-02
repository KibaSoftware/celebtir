"use client";
import { Finding } from "@/components/research/Finding";
import { FACTOR_QUESTIONS } from "@/lib/presentation";

import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { CompareToggle } from "@/components/compare/CompareToggle";
import { DATA_STATES } from "@/components/DataState";
import { FactorDistribution } from "@/components/research/FactorDistribution";
import { ControllerAuthorityTree } from "@/components/research/ControllerAuthorityTree";
import { EvidenceList } from "@/components/Evidence";
import { ProtocolLogo, StageChip } from "@/components/ProtocolBits";
import { StageInspector } from '@/components/research/StageInspector';
import { ProtectionsBrowser } from '@/components/research/ProtectionsBrowser';
import { PROTECTION_LABELS, type ProtectionRow, type ProtectionKey } from '@/lib/protections';
import { RatingBadge } from "@/components/RatingBadge";
import { DetailPanel, PanelSection } from "@/components/ui/DetailPanel";
import { FACTOR_LABELS } from "@/lib/labels";
import { applyView, assessedRows, CATEGORY_LABELS, onChain, type ProtocolRow, type RatedRow } from "@/lib/summary";
import { ScopeEmpty } from "@/components/scope/ScopeEmpty";
import { ratingStyle, type Rating } from "@/lib/tokens";
import { FACTORS, type Factor, type FactorScore, type Controller } from "@/lib/types";
import { useViewState } from "@/lib/useViewState";
import { DEFAULT_VIEW, detailOf, TABS, type Tab, type ViewState } from "@/lib/viewState";
import styles from "./Page.module.css";

type Detail = { id: string; name: string; score: FactorScore; implications?: string[]; authorities?: {name: string; controller: Controller}[] };
export type Details = Record<Factor, Detail[]>;
type Update = (next: (v: ViewState) => ViewState) => void;

/** The methodology's criteria for a factor, with its thresholds written in. */
export interface FactorCriteria {
  question: string;
  low: string;
  medium: string;
  high: string;
}

export interface RiskMeta {
  date: string;
  methodology: string;
}

const RATINGS: Rating[] = ["low", "medium", "high", "unknown", "not_applicable"];
const toggle = <T,>(xs: T[], x: T) => (xs.includes(x) ? xs.filter((y) => y !== x) : [...xs, x]);

/** One cell's research panel: what was observed, kept apart from how it is assessed. */
function RiskDetail({
  row,
  factor,
  score,
  finding,
  protection,
  criteria,
  meta,
  onClose,
}: {
  row: RatedRow;
  factor: Factor;
  score: FactorScore | undefined;
  finding?: Detail;
  protection?: ProtectionRow;
  criteria?: FactorCriteria;
  meta?: RiskMeta;
  onClose: () => void;
}) {
  const rating = row.ratings[factor];
  const criterion = criteria && (rating === "low" || rating === "medium" || rating === "high") ? criteria[rating] : null;
  return (
    <DetailPanel art="risk" eyebrow={row.name} title={FACTOR_LABELS[factor]} onClose={onClose}>
      <div>
        <RatingBadge rating={rating} tone="pill" />
      </div>
      <PanelSection title="Observed configuration" tone="observed">
        <p>{FACTOR_QUESTIONS[factor]}</p><Finding text={score?.reason ?? row.reasons[factor]} />
        {factor === "upgrade_control" || factor === "upgrade_delay" ? (
          <p>
            Upgrade controller: <strong>{row.controller}</strong>
          </p>
        ) : null}
      </PanelSection>
      <PanelSection title="Recorded authority paths" tone="observed">
        {finding?.authorities?.length ? finding.authorities.map((a) => <details key={a.name}><summary>{a.name}</summary><ControllerAuthorityTree controller={a.controller} /></details>) : <p>No separate authority path is recorded for this factor. Consult the observed configuration and source fields.</p>}
      </PanelSection>
      <PanelSection title="Why it matters" tone="observed">
        {finding?.implications?.length ? finding.implications.map(text => <p key={text}>{text}</p>) : <p>No elevated-risk implication is recorded for this factor. The rating describes the observed configuration, not code safety.</p>}
      </PanelSection>
      <PanelSection title="Verified protections" tone="observed">
        {protection && Object.entries(protection.observations).some(([key, observation]) => observation.factor === factor && observation.value === "TRUE" && ["timelock", "multi_actor", "transparency"].includes(key)) ?
          <ul>{Object.entries(protection.observations).filter(([key, observation]) => observation.factor === factor && observation.value === "TRUE" && ["timelock", "multi_actor", "transparency"].includes(key)).map(([key, observation]) => <li key={key}>{PROTECTION_LABELS[key as ProtectionKey]}<details><summary>Supporting evidence</summary><EvidenceList evidence={observation.evidence} /></details></li>)}</ul>
          : <p>No separate protection is established for this factor by the available published observations.</p>}
        <p>Multiple approval keys do not establish that their owners are independent.</p>
      </PanelSection>
      <PanelSection title="Methodology criterion" tone="method">
        <p><Link href={`/methodology/#factor-${factor}`}>Read the {FACTOR_LABELS[factor]} rule{meta ? ` · v${meta.methodology}` : ""}</Link></p>
        {criteria ? <p>{criteria.question}</p> : null}
        {criterion ? (
          <p>
            <strong>{ratingStyle[rating].label}:</strong> {criterion}
          </p>
        ) : rating === "unknown" ? (
          <p>{DATA_STATES.unknown.explain}</p>
        ) : rating === "not_applicable" ? (
          <p>{DATA_STATES.not_applicable.explain}</p>
        ) : null}
        {criteria ? (
          <details>
            <summary>All criteria for this factor</summary>
            <dl className={styles.criteria}>
              {(["low", "medium", "high"] as const).map((r) => (
                <div key={r}>
                  <dt>
                    <RatingBadge rating={r} />
                  </dt>
                  <dd>{criteria[r]}</dd>
                </div>
              ))}
            </dl>
          </details>
        ) : null}
      </PanelSection>
      <PanelSection title="What would improve the rating" tone="method">
        <p>{score?.next_step ?? "No step recorded."}</p>
      </PanelSection>
      <PanelSection title="Snapshot">
        <p className={styles.muted}>
          {meta ? `Snapshot ${meta.date} · methodology v${meta.methodology}` : null}
        </p>
        <p>
          <Link href={`/protocols/${row.id}/#factor-${factor}`}>Open {row.name}&apos;s full assessment</Link>
        </p>
      </PanelSection>
      <PanelSection title="Evidence: accounts read" tone="observed">
        <EvidenceList evidence={score?.evidence ?? []} />
      </PanelSection>
      <PanelSection title="Assessment boundary">
        <p>This assessment describes the recorded configuration, not code safety. Evidence above identifies each observation&apos;s chain and anchor. Unavailable reads remain Unknown; dependency observations are sampled.</p>
      </PanelSection>
    </DetailPanel>
  );
}

/** `/risk`: protocols × six factors, or one factor's detail (state in the URL, UI-1). */
export function RiskViewBody({
  protections = [],
  rows,
  details,
  criteria,
  meta,
  logos = {},
  view,
  update,
}: {
  rows: ProtocolRow[];
  protections?: ProtectionRow[];
  details: Details;
  criteria?: Record<Factor, FactorCriteria>;
  meta?: RiskMeta;
  logos?: Record<string, string | null>;
  view: ViewState;
  update: Update;
}) {
  const sortBy = (key: string) =>
    update((v) => ({ ...v, sort: { key, dir: v.sort.key === key && v.sort.dir === "desc" ? "asc" : "desc" } }));
  const ariaSort = (key: string) =>
    view.sort.key === key ? (view.sort.dir === "asc" ? ("ascending" as const) : ("descending" as const)) : undefined;
  const factor = view.factor;
  // F5: ratings are shown for assessed protocols only.
  const rated = assessedRows(rows);
  const nativeRows = onChain(rows, view.chain).filter(r => r.assessment !== "assessed");
  const shown = applyView(rated, view);
  const highlighting = view.highlight.length > 0;
  const matches = (f: Factor, r: Rating) =>
    highlighting && view.highlight.includes(r) && (view.highlightFactor === null || view.highlightFactor === f);
  const matchingRows = shown.filter((row) => FACTORS.some((f) => matches(f, row.ratings[f]))).length;
  const open = detailOf(view);
  const openRow = open ? rated.find((r) => r.id === open.id) : undefined;
  // HUI-1: chips count the chosen chain's protocols.
  const inChain = onChain(rated, view.chain);
  const inChainIds = new Set(inChain.map((r) => r.id));
  const counts = Object.fromEntries(TABS.map((t) => [t, t === "all" ? inChain.length : inChain.filter((r) => r.category === t).length])) as Record<Tab, number>;

  return (
    <>
      {nativeRows.length ? <p>Native HyperEVM assessment is Unassessed here: this methodology version does not rate HyperEVM protocols, so they carry no ratings or Stage. <Link href="/hyperevm/">Inspect {nativeRows.length} protocols through their contract authorities</Link>.</p> : null}
      <FactorDistribution rows={shown} art="risk" />
      <div className={styles.toolbar}>
        <div className={styles.segmented} role="group" aria-label="View" data-shortcut="filters">
          <button type="button" className={styles.tab} aria-pressed={factor === null} onClick={() => update((v) => ({ ...v, factor: null }))}>
            All factors
          </button>
          {FACTORS.map((f) => (
            <button key={f} type="button" className={styles.tab} aria-pressed={factor === f} onClick={() => update((v) => ({ ...v, factor: f }))}>
              {FACTOR_LABELS[f]}
            </button>
          ))}
        </div>
      </div>

      {factor === null ? (
        <>
          <div className={styles.filterBar}>
            <div className={styles.filterGroup} role="group" aria-label="Category">
              <span className={styles.filterLabel}>Category</span>
              {TABS.map((t) => (
                <button
                  key={t}
                  type="button"
                  className={styles.chip}
                  aria-pressed={view.tab === t}
                  disabled={t !== "all" && counts[t] === 0}
                  onClick={() => update((v) => ({ ...v, tab: t }))}
                >
                  {CATEGORY_LABELS[t].label} <span className={styles.chipCount}>{counts[t]}</span>
                </button>
              ))}
            </div>
            <div className={styles.filterGroup} role="group" aria-label="Highlight ratings">
              <span className={styles.filterLabel}>Highlight</span>
              {RATINGS.map((r) => (
                <button
                  key={r}
                  type="button"
                  className={styles.chip}
                  aria-pressed={view.highlight.includes(r)}
                  onClick={() => update((v) => ({ ...v, highlight: toggle(v.highlight, r) }))}
                >
                  <RatingBadge rating={r} />
                </button>
              ))}
              <label className={styles.inlineSelect}>
                <span>in</span>
                <select
                  value={view.highlightFactor ?? ""}
                  onChange={(e) => {
                    const value = e.target.value as Factor | "";
                    update((v) => ({ ...v, highlightFactor: value === "" ? null : value }));
                  }}
                >
                  <option value="">all six factors</option>
                  {FACTORS.map((f) => (
                    <option key={f} value={f}>
                      {FACTOR_LABELS[f]}
                    </option>
                  ))}
                </select>
              </label>
              {highlighting ? (
                <span className={styles.matchCount} role="status">
                  {matchingRows} of {shown.length} protocols match
                </span>
              ) : null}
            </div>
          </div>

          <div className={styles.matrixScroll} role="region" aria-label="Risk factor matrix" tabIndex={0}>
            <table className={styles.matrix} data-highlighting={highlighting || undefined}>
              <caption className="sr-only">Every protocol rated on the six factors. Select a rating to open its evidence.</caption>
              <thead>
                <tr>
                  <th scope="col" aria-sort={ariaSort("name")} className={styles.stickyCol}>
                    <button type="button" className={styles.sort} onClick={() => sortBy("name")}>
                      Protocol
                    </button>
                  </th>
                  <th scope="col" aria-sort={ariaSort("stage")}>
                    <button type="button" className={styles.sort} onClick={() => sortBy("stage")}>
                      Stage
                    </button>
                  </th>
                  {FACTORS.map((f) => (
                    <th key={f} scope="col" aria-sort={ariaSort(f)} data-factor-col={view.highlightFactor === f || undefined}>
                      <button type="button" className={styles.sort} onClick={() => sortBy(f)}>
                        {FACTOR_LABELS[f]}
                      </button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {shown.map((r) => (
                  <tr key={r.id} data-protocol={r.id} data-selected={openRow?.id === r.id || undefined}>
                    <th scope="row" className={styles.stickyCol}>
                      <span className={styles.nameCell}>
                        <CompareToggle id={r.id} name={r.name} />
                        <ProtocolLogo name={r.name} logo={logos[r.id] ?? null} size={26} />
                        <Link href={`/protocols/${r.id}/`} className={styles.protocolLink}>
                          {r.name}
                        </Link>
                      </span>
                    </th>
                    <td>
                      {r.inspection ? <StageInspector inspection={r.inspection} /> : <StageChip stage={r.stage} />}
                    </td>
                    {FACTORS.map((f) => (
                      <td key={f} data-match={matches(f, r.ratings[f]) || undefined}>
                        <button
                          type="button"
                          className={styles.cellButton}
                          title={r.reasons[f]}
                          aria-label={`${r.name}, ${FACTOR_LABELS[f]}: ${ratingStyle[r.ratings[f]].label}. Open the evidence`}
                          aria-haspopup="dialog"
                          onClick={() => update((v) => ({ ...v, detail: `${r.id}.${f}` }))}
                        >
                          <RatingBadge rating={r.ratings[f]} tone="pill" />
                        </button>
                      </td>
                    ))}
                  </tr>
                ))}
                {shown.length === 0 ? (
                  <tr>
                    <td colSpan={8} className={styles.muted}>
                      {inChain.length === 0 ? <ScopeEmpty chain={view.chain} kind="protocols" /> : "No protocol in this category."}
                    </td>
                  </tr>
                ) : null}
              </tbody>
            </table>
          </div>
          <p className={styles.hint}>Select any rating to open what was observed, the methodology criterion and the accounts it was read from.</p>
        </>
      ) : (
        <div className={styles.scroll}>
          <table className={styles.table}>
            <caption className="sr-only">{FACTOR_LABELS[factor]} for every protocol</caption>
            <thead>
              <tr>
                <th scope="col">Protocol</th>
                <th scope="col">Rating</th>
                <th scope="col">Why</th>
                <th scope="col">To improve</th>
                <th scope="col">Evidence</th>
              </tr>
            </thead>
            <tbody>
              {details[factor].filter((d) => inChainIds.has(d.id)).map((d) => (
                <tr key={d.id} data-protocol={d.id}>
                  <th scope="row">
                    <Link href={`/protocols/${d.id}/#risk-analysis`}>{d.name}</Link>
                  </th>
                  <td>
                    <RatingBadge rating={d.score.rating} tone="pill" />
                  </td>
                  <td className={styles.prose}>{d.score.reason}</td>
                  <td className={`${styles.muted} ${styles.prose}`}>{d.score.next_step}</td>
                  <td>
                    <EvidenceList evidence={d.score.evidence} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {open && openRow ? (
        <RiskDetail
          row={openRow}
          factor={open.factor}
          score={details[open.factor].find((d) => d.id === open.id)?.score}
          finding={details[open.factor].find((d) => d.id === open.id)}
          protection={protections.find(p => p.id === open.id)}
          criteria={criteria?.[open.factor]}
          meta={meta}
          onClose={() => update((v) => ({ ...v, detail: null }))}
        />
      ) : null}
      {protections.length ? <details><summary>Compare verified protections</summary><ProtectionsBrowser rows={protections} view={view} update={update}/></details> : null}
    </>
  );
}

type RiskProps = {
  protections?: ProtectionRow[];
  rows: ProtocolRow[];
  details: Details;
  criteria?: Record<Factor, FactorCriteria>;
  meta?: RiskMeta;
  logos?: Record<string, string | null>;
};

export function RiskView(props: RiskProps) {
  const [view, update] = useViewState();
  return <RiskViewBody {...props} view={view} update={update} />;
}

export function RiskViewDefault(props: RiskProps) {
  return <RiskViewBody {...props} view={DEFAULT_VIEW} update={() => {}} />;
}
