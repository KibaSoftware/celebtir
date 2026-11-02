"use client";

import { LimitedItems } from "@/components/LimitedItems";
import { eventAnchor } from "@/lib/changes";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { Empty, TxLink, EvidenceList } from "@/components/Evidence";
import { EmptyState } from "@/components/ui/Ui";
import { filterLog, parseLog, serializeLog, SEVERITIES, SOURCES, type LogView } from "@/lib/changelog";
import type { ChangeEvent } from "@/lib/types";
import styles from "./Reference.module.css";

const SEVERITY_LABELS = { critical: "Critical", warning: "Warning", positive: "Positive", info: "Info" } as const;
const SOURCE_LABELS = { on_chain: "On-chain", registry_edit: "Registry edit", methodology: "Methodology" } as const;
const toggle = <T,>(xs: T[], x: T) => (xs.includes(x) ? xs.filter((y) => y !== x) : [...xs, x]);

/** `controller_changed` → "Controller changed". */
const kindLabel = (kind: string) => {
  const words = kind.replace(/_/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
};

/** A shape per severity, so it reads without colour. */
function SeverityIcon({ severity }: { severity: ChangeEvent["severity"] }) {
  const common = { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": true, focusable: false } as const;
  switch (severity) {
    case "critical":
      return (
        <svg {...common}>
          <rect x="1" y="1" width="10" height="10" rx="2" fill="currentColor" />
        </svg>
      );
    case "warning":
      return (
        <svg {...common}>
          <path d="M6 1l5 10H1z" fill="currentColor" />
        </svg>
      );
    case "positive":
      return (
        <svg {...common}>
          <circle cx="6" cy="6" r="5" fill="currentColor" />
        </svg>
      );
    case "info":
      return (
        <svg {...common}>
          <circle cx="6" cy="6" r="4.2" fill="none" stroke="currentColor" strokeWidth="1.6" />
        </svg>
      );
  }
}

/** The global event feed with filters (site map: /changelog). */
export function ChangelogBody({
  events,
  names,
  feeds,
  view,
  update,
}: {
  events: ChangeEvent[];
  names: Record<string, string>;
  /** Protocol IDs that have their own Atom feed. */
  feeds: string[];
  view: LogView;
  update: (v: LogView) => void;
}) {
  const shown = filterLog(events, view);
  const feed = view.protocol && feeds.includes(view.protocol) ? `/feeds/${view.protocol}.atom` : "/feeds/all.atom";
  return (
    <>
      <p>
        <a href={feed}>Subscribe with Atom{view.protocol && feed !== "/feeds/all.atom" ? ` (${names[view.protocol] ?? view.protocol})` : ""}</a>
      </p>
      <div className={styles.filters} data-shortcut="filters">
        <fieldset>
          <legend>Severity</legend>
          {SEVERITIES.map((s) => (
            <label key={s}>
              <input type="checkbox" checked={view.severity.includes(s)} onChange={() => update({ ...view, severity: toggle(view.severity, s) })} />
              {SEVERITY_LABELS[s]}
            </label>
          ))}
        </fieldset>
        <fieldset>
          <legend>Source</legend>
          {SOURCES.map((s) => (
            <label key={s}>
              <input type="checkbox" checked={view.source.includes(s)} onChange={() => update({ ...view, source: toggle(view.source, s) })} />
              {SOURCE_LABELS[s]}
            </label>
          ))}
        </fieldset>
        <label>
          Protocol
          <select value={view.protocol ?? ""} onChange={(e) => update({ ...view, protocol: e.target.value || null })}>
            <option value="">All protocols</option>
            {Object.entries(names)
              .sort(([, a], [, b]) => a.localeCompare(b))
              .map(([id, name]) => (
                <option key={id} value={id}>
                  {name}
                </option>
              ))}
          </select>
        </label>
      </div>
      {events.length === 0 ? (
        <EmptyState kind="history" title="No transition records available">
          <p>Events appear here once two daily snapshots differ; nothing is shown that was not observed.</p>
        </EmptyState>
      ) : shown.length === 0 ? (
        <Empty>No events match these filters.</Empty>
      ) : (
        <LimitedItems ordered className={styles.timeline} label="Events, newest first">
          {shown.map((e, i) => {
            const firstOfDay = i === 0 || shown[i - 1]!.date !== e.date;
            return (
              <li key={e.id} id={eventAnchor(e.id)} className={styles.event} data-severity={e.severity} data-source={e.source} data-first={firstOfDay || undefined}>
                <span className={styles.eventDate} aria-hidden={!firstOfDay || undefined}>
                  {firstOfDay ? e.date : ""}
                </span>
                <span className={styles.eventDot} aria-hidden="true" />
                <div className={styles.eventCard}>
                  <div className={styles.eventHead}>
                    <span className={styles.severity} data-severity={e.severity}>
                      <SeverityIcon severity={e.severity} />
                      {SEVERITY_LABELS[e.severity]}
                    </span>
                    <span className={styles.source} data-source={e.source}>
                      {SOURCE_LABELS[e.source]}
                    </span>
                    <span className={styles.kind}>{kindLabel(e.kind)}</span>
                    {firstOfDay ? null : <span className="sr-only">{e.date}</span>}
                  </div>
                  <p className={styles.eventSummary}>
                    {e.protocol ? (
                      <>
                        <Link href={`/protocols/${e.protocol}/#updates`} className={styles.eventProtocol}>
                          {names[e.protocol] ?? e.protocol}
                        </Link>
                        :{" "}
                      </>
                    ) : null}
                    {e.summary}
                  </p>
                  <details><summary>Source record and observed fields</summary>
                    <p>Record <code>{e.id}</code> · subject <code>{e.subject}</code> · {e.chain ?? e.evidence?.[0]?.chain ?? "Chain unavailable"}</p>
                    {e.before !== undefined && e.after !== undefined ? <dl><dt>Before</dt><dd><pre style={{whiteSpace:"pre-wrap",overflowWrap:"anywhere"}}>{JSON.stringify(e.before,null,2)}</pre></dd><dt>After</dt><dd><pre style={{whiteSpace:"pre-wrap",overflowWrap:"anywhere"}}>{JSON.stringify(e.after,null,2)}</pre></dd></dl> : <p>Before/after fields unavailable in this record. No transition is reconstructed.</p>}
                    <EvidenceList evidence={e.evidence ?? []} />
                    {e.corrects ? <p>Correction to <code>{e.corrects}</code></p> : null}
                  </details>
                  <p className={styles.eventMeta}>
                    <span>
                      ({SOURCE_LABELS[e.source]}
                      {e.occurred_on ? `, on chain ${e.occurred_on}${e.slot ? ` at slot ${e.slot.toLocaleString("en-US")}` : ""}` : ""}
                      {e.backfill ? ", recorded from upgrade history older than the comparison window" : e.detected_within_days > 1 ? `, detected within ${e.detected_within_days} days` : ""})
                    </span>
                    {e.signature ? (
                      <>
                        {" "}
                        <TxLink signature={e.signature} label="transaction" />
                      </>
                    ) : null}
                  </p>
                </div>
              </li>
            );
          })}
        </LimitedItems>
      )}
    </>
  );
}

export function ChangelogView(props: { events: ChangeEvent[]; names: Record<string, string>; feeds: string[] }) {
  const params = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();
  const view = parseLog(new URLSearchParams(params.toString()));
  const update = (v: LogView) => {
    const q = serializeLog(v);
    router.replace(q ? `${pathname}?${q}` : pathname, { scroll: false });
  };
  return <ChangelogBody {...props} view={view} update={update} />;
}

export function ChangelogDefault(props: { events: ChangeEvent[]; names: Record<string, string>; feeds: string[] }) {
  return <ChangelogBody {...props} view={{ severity: [], source: [], protocol: null }} update={() => {}} />;
}
