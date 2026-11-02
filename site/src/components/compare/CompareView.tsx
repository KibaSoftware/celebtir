"use client";

import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { useState } from "react";
import { Tooltip } from "@/components/Tooltip";
import { readableFinding, nextStageCounts } from "@/lib/presentation";
import { GroupedReason } from "@/components/ReasonText";
import { TimeChart } from "@/components/charts/TimeChart";
import { Empty } from "@/components/Evidence";
import { FactorStrip, StageChip, StageSwatch } from "@/components/ProtocolBits";
import { StageInspector } from '@/components/research/StageInspector';
import { RatingBadge } from "@/components/RatingBadge";
import { EvidenceList } from "@/components/Evidence";
import { DetailPanel, PanelSection } from "@/components/ui/DetailPanel";
import { FACTOR_LABELS } from "@/lib/labels";
import { compareRows, overlay, type CompareItem } from "@/lib/compare";
import { FACTORS, type Factor } from "@/lib/types";
import type { Rating } from "@/lib/tokens";
import { useCompareTray } from "@/lib/compareTray";
import { useNarrow } from "@/lib/useNarrow";
import { useViewState } from "@/lib/useViewState";
import { COMPARE_MAX, DEFAULT_VIEW, type ViewState } from "@/lib/viewState";
import styles from "./Compare.module.css";
import { inScope } from "@/lib/scope";
import { CopyButton } from "@/components/CopyButton";

type Update = (next: (v: ViewState) => ViewState) => void;

/** UI-16 to UI-18, driven by an explicit view (pre-rendered with the default). */
export function CompareBody({
  items,
  view,
  update,
  fromTray,
}: {
  items: Record<string, CompareItem>;
  view: ViewState;
  update: Update;
  fromTray: string[];
}) {
  const [q, setQ] = useState("");
  const [inspectedFactor, setInspectedFactor] = useState<Factor | null>(null);
  const chosen = view.compare.map((id) => items[id]).filter((i) => i !== undefined && inScope(view.chain, i.chain ?? "solana"));
  const add = (id: string) => update((v) => ({ ...v, compare: v.compare.includes(id) ? v.compare : [...v.compare, id].slice(0, COMPARE_MAX) }));
  const remove = (id: string) => update((v) => ({ ...v, compare: v.compare.filter((x) => x !== id) }));
  const matches = q.trim()
    ? Object.values(items)
        .filter((i) => inScope(view.chain, i.chain ?? "solana"))
        .filter((i) => !view.compare.includes(i.id) && (i.name.toLowerCase().includes(q.toLowerCase()) || i.id.includes(q.toLowerCase())))
        .slice(0, 8)
    : [];
  // Rows compare every chosen protocol, so "same" stays true to all of them.
  const rows = chosen.length >= 2 ? compareRows(chosen) : [];
  // UI-25: on a phone, two protocols at a time with a pager.
  const narrow = useNarrow();
  const [offset, setOffset] = useState(0);
  const start = narrow ? Math.min(offset, Math.max(0, chosen.length - 2)) : 0;
  const visible = narrow ? chosen.slice(start, start + 2) : chosen;
  const crossDomain = chosen.some(i => (i.chain ?? "solana") !== "solana");
  const comparable = crossDomain ? rows.filter(r => r.group !== "Risk" && r.group !== "Stage" && r.group !== "Upgrades" && !r.cells.some(c => c.rating || c.stage)) : rows;
  const shown = view.diff ? comparable.filter((r) => !r.same) : comparable;
  const trayUsable = fromTray.filter((id) => items[id]);

  return (
    <>
      {crossDomain ? <p role="status">Cross-domain comparison: native authority models and Stages are not interchangeable. Only shared descriptive and economic fields are shown; inspect each domain&apos;s source evidence separately.</p> : null}
      <div className={styles.picker}>
        <label className={styles.search}>
          <span>Add a protocol</span>
          <input
            type="search"
            data-shortcut="search"
            value={q}
            placeholder="Search by name"
            disabled={view.compare.length >= COMPARE_MAX}
            onChange={(e) => setQ(e.target.value)}
          />
        </label>
        {matches.length ? (
          <ul className={styles.matches} aria-label="Matching protocols">
            {matches.map((m) => (
              <li key={m.id}>
                <button
                  type="button"
                  onClick={() => {
                    add(m.id);
                    setQ("");
                  }}
                >
                  {m.name}
                </button>
              </li>
            ))}
          </ul>
        ) : null}
        {trayUsable.length >= 2 && trayUsable.join(",") !== view.compare.join(",") ? (
          <button type="button" className={styles.clear} onClick={() => update((v) => ({ ...v, compare: trayUsable }))}>
            Use the compare tray ({trayUsable.length})
          </button>
        ) : null}
        <label className={styles.diff} data-on={view.diff || undefined}>
          <input type="checkbox" className={styles.switch} disabled={chosen.length < 2} checked={chosen.length >= 2 && view.diff} onChange={() => update((v) => ({ ...v, diff: !v.diff }))} />
          Show differences only
          {rows.length ? (
            <span className={styles.diffCount} aria-hidden="true">
              {rows.filter((r) => !r.same).length} of {rows.length} rows differ
            </span>
          ) : null}
        </label>
      </div>

      <div className={styles.selection} aria-label="Selected protocols" aria-live="polite">
        <CopyButton value="comparison link" currentPage />
        {chosen.map(c => <button key={c.id} type="button" onClick={() => remove(c.id)} aria-label={`Remove ${c.name}`}>{c.name} <span aria-hidden="true">&times;</span></button>)}
      </div>
      {chosen.length < 2 ? (
        <section className={styles.guide} aria-labelledby="compare-guide-title">
          <p className={styles.guideStep}>Step {chosen.length + 1} of 2</p>
          <h2 id="compare-guide-title">Compare how protocols are controlled.</h2>
          <p>Choose 2 to 4 protocols to inspect each independent factor, controller and evidence side by side.</p>
          <p>{chosen.length ? `Now choose a peer for ${chosen[0]!.name}.` : "Start with a protocol below, or search by name above."} Comparison appears automatically after your second selection.</p>
          <div className={styles.suggestions} aria-label="Suggested comparisons">
            {Object.values(items).filter(i => inScope(view.chain, i.chain ?? "solana") && !chosen.some(c => c.id === i.id) && (!chosen[0] || i.category === chosen[0].category)).slice(0, 3).map(i => <button key={i.id} type="button" onClick={() => add(i.id)}>
              <span className={styles.guideStep}>{i.category}</span><strong>Add {i.name}</strong><span>{i.controller}</span>
            </button>)}
          </div>
          <p className={styles.guideNote}>Stages describe control arrangements. TVL is context; no combined risk score is calculated.</p>
        </section>
      ) : (
        <>
          {narrow && chosen.length > 2 ? (
            <div className={styles.pager}>
              <button type="button" aria-label="Previous two" disabled={start === 0} onClick={() => setOffset(Math.max(0, start - 2))}>
                ←
              </button>
              <span>
                Showing {start + 1}–{start + visible.length} of {chosen.length}
              </span>
              <button
                type="button"
                aria-label="Next two"
                disabled={start + 2 >= chosen.length}
                onClick={() => setOffset(Math.min(chosen.length - 2, start + 2))}
              >
                →
              </button>
            </div>
          ) : null}
          <table className={styles.table} data-cards="off">
            <caption className="sr-only">Comparison of {visible.map((c) => c.name).join(", ")}</caption>
            <colgroup>
              <col className={styles.labelCol} />
              {visible.map((c) => (
                <col key={c.id} />
              ))}
            </colgroup>
            <thead>
              <tr>
                <td />
                {visible.map((c) => (
                  <th key={c.id} scope="col">
                    <div className={styles.headCard}>
                      <div className={styles.headTop}>
                        <Link href={`/protocols/${c.id}/`} className={styles.headName}>
                          {c.name}
                        </Link>
                        <button type="button" className={styles.remove} aria-label={`Remove ${c.name} from the comparison`} onClick={() => remove(c.id)}>
                          ×
                        </button>
                      </div>
                      <div className={styles.headMeta}>
                        {crossDomain ? <span>Native domain evidence</span> : c.inspection ? <StageInspector inspection={c.inspection} /> : <StageChip stage={c.stage} />}
                        <span className={styles.headController}>{c.controller}</span>
                      </div>
                      {!crossDomain ? <FactorStrip
                        ratings={Object.fromEntries(FACTORS.map((f) => [f, c.factors[f].rating])) as Record<Factor, Rating>}
                        reasons={Object.fromEntries(FACTORS.map((f) => [f, c.factors[f].reason])) as Record<Factor, string>}
                      /> : null}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {shown.map((r) => (
                <tr key={r.label} data-same={r.same || undefined} data-differs={!r.same || undefined}>
                  <th scope="row">
                    <span className={styles.group}>{r.group}</span>
                    {r.label}{["Upgrades", "Average interval"].includes(r.label) ? <Tooltip label={`About ${r.label}`} trigger="?">Recorded program history through this snapshot, not a fixed calendar window. Incomplete history is labelled; average interval uses recorded upgrade pairs.</Tooltip> : null}
                  </th>
                  {r.cells.slice(start, start + visible.length).map((c, i) => (
                    <td key={visible[i]!.id}>
                      {c.stage ? (
                        <span className={styles.inline}>
                          <StageSwatch stage={c.stage} /> {c.text}
                        </span>
                      ) : c.rating ? (
                        <>
                          <button className={styles.clear} aria-haspopup="dialog" onClick={() => setInspectedFactor(FACTORS.find(f => FACTOR_LABELS[f] === r.label) ?? null)}><RatingBadge rating={c.rating} tone="pill" /></button>
                          <p className={styles.observation}>{visible[i]!.findings?.[FACTORS.find(f => FACTOR_LABELS[f] === r.label)!] ?? readableFinding(c.text)}</p>
                          <button type="button" className={styles.clear} onClick={() => setInspectedFactor(FACTORS.find(f => FACTOR_LABELS[f] === r.label) ?? null)}>Reason and criteria<span className="sr-only"> for {visible[i]!.name}: {r.label}</span></button>
                        </>
                      ) : r.label === "Missing for the next Stage" && nextStageCounts(visible[i]!.inspection) ? (() => { const counts = nextStageCounts(visible[i]!.inspection)!; return <span>{counts.met} met · {counts.unmet} verified unmet · {counts.unresolved} unresolved. <details><summary>Published blockers</summary>{c.text}</details></span>; })() : (
                        c.text
                      )}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
          {view.diff && shown.length === 0 ? <Empty>These protocols match on every row: there are no differences to show.</Empty> : null}

          <h2 className={styles.h2}>TVL and activity over time</h2>
          <div className={styles.charts}>
            {(() => {
              const tvl = overlay(chosen.map((c) => ({ id: c.id, name: c.name, days: c.tvlDays })));
              return tvl.points.length >= 2 ? (
                <TimeChart title="TVL" description="Each protocol's TVL, one line per protocol." series={tvl.series} points={tvl.points} unit="usd" overlay />
              ) : (
                <Empty>The TVL chart appears once two daily snapshots exist.</Empty>
              );
            })()}
            {(() => {
              const act = overlay(chosen.map((c) => ({ id: c.id, name: c.name, days: c.fees.kind === "usd" ? c.fees.days : [] })));
              return act.points.length >= 2 ? (
                <TimeChart
                  title="Daily fees"
                  description="Comparable fee observations from DefiLlama. Missing fee coverage remains a gap."
                  series={act.series}
                  points={act.points}
                  unit="usd"
                  overlay
                />
              ) : (
                <Empty>No daily fees or volume reported for these protocols.</Empty>
              );
            })()}
          </div>
        </>
      )}
      {inspectedFactor ? <DetailPanel art="compare" title={FACTOR_LABELS[inspectedFactor]} eyebrow="Aligned comparison evidence" onClose={() => setInspectedFactor(null)}>
        {chosen.map(c => <PanelSection key={c.id} title={c.name} tone="observed"><RatingBadge rating={c.factors[inspectedFactor].rating} /><GroupedReason text={c.factors[inspectedFactor].reason} /><p>{c.evidenceFactors?.[inspectedFactor].next_step ?? "Improvement criteria unavailable in this record."}</p><EvidenceList evidence={c.evidenceFactors?.[inspectedFactor].evidence ?? []} /><Link href={`/risk/?detail=${c.id}.${inspectedFactor}`}>Full criterion and snapshot</Link></PanelSection>)}
      </DetailPanel> : null}
    </>
  );
}

export function CompareView({ items }: { items: Record<string, CompareItem> }) {
  const [view, update] = useViewState();
  const tray = useCompareTray();
  return <CompareBody items={items} view={view} update={update} fromTray={tray.list} />;
}

export function CompareViewDefault({ items }: { items: Record<string, CompareItem> }) {
  return <CompareBody items={items} view={DEFAULT_VIEW} update={() => {}} fromTray={[]} />;
}
