"use client";

import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { Finding } from "@/components/research/Finding";
import { authorityWording, SIGNER_BOUNDARY } from "@/lib/presentation";
import { LimitedItems } from "@/components/LimitedItems";
import { Fragment, useState } from "react";
import { RatingBadge } from "@/components/RatingBadge";
import { shortAddress } from "@/lib/format";
import { ratingStyle, type Rating } from "@/lib/tokens";
import type { TokenFactor, FactorScore, Controller } from "@/lib/types";
import { ControllerAuthorityTree } from "@/components/research/ControllerAuthorityTree";
import { useNarrow } from "@/lib/useNarrow";
import { EvidenceList } from "@/components/Evidence";
import { DetailPanel, PanelSection } from "@/components/ui/DetailPanel";
import { useViewState } from "@/lib/useViewState";
import { DEFAULT_VIEW, type ViewState } from "@/lib/viewState";
import styles from "./Page.module.css";
import local from "./TokenTable.module.css";

export interface TokenTableRow {
  mint: string;
  symbol: string;
  kind: string | null;
  issuer: string | null;
  program: "spl_token" | "token2022";
  provenance: string;
  ratings: Record<TokenFactor, Rating>;
  scores?: Record<TokenFactor, FactorScore>;
  methodology?: string;
  observedAt?: string;
  authorities?: Partial<Record<TokenFactor, Controller>>;
  additionalAuthorities?: {factor: TokenFactor; label: string; controller: Controller}[];
  reasons: Record<TokenFactor, string>;
  exposure: { id: string; name: string; roles: string[] }[];
  risks: { severity: "high" | "medium"; text: string }[];
}

type Update = (next: (v: ViewState) => ViewState) => void;
const RATINGS: Rating[] = ["low", "medium", "high", "unknown", "not_applicable"];
const toggle = <T,>(xs: T[], x: T) => (xs.includes(x) ? xs.filter((y) => y !== x) : [...xs, x]);

/** Tokens matching the URL's search, type and rating filters (UI-1). */
export function filterTokens(rows: TokenTableRow[], factors: readonly TokenFactor[], v: ViewState): TokenTableRow[] {
  const q = v.q.trim().toLowerCase();
  return rows.filter(
    (t) =>
      (!q || t.symbol.toLowerCase().includes(q) || (t.issuer ?? "").toLowerCase().includes(q) || t.mint.toLowerCase().startsWith(q)) &&
      (!v.tokenKind.length || v.tokenKind.includes(t.kind ?? "unlisted")) &&
      (!v.tokenProvenance || (v.tokenProvenance === "bridged" ? t.provenance.startsWith("Bridged") : t.provenance.startsWith("Native"))) &&
      (!v.tokenUse || t.exposure.some(e => e.roles.includes(v.tokenUse!))) &&
      (!v.tokenRating.length || factors.some((f) => v.tokenRating.includes(t.ratings[f]))),
  );
}

/**
 * The token directory (TOK-6): search, type and rating filters in the URL, a
 * rating per issuer-side factor, and a row that expands to the risk sentences.
 * These are the issuer's powers over the token, not the accepting protocols' risks.
 */
export function TokenTableView({
  rows,
  factors,
  factorLabels,
  kindLabels,
  view,
  update,
}: {
  rows: TokenTableRow[];
  factors: readonly TokenFactor[];
  factorLabels: Record<TokenFactor, string>;
  kindLabels: Record<string, string>;
  view: ViewState;
  update: Update;
}) {
  const [open, setOpen] = useState<string | null>(null);
  const inspected = view.tokenDetail ? {mint: view.tokenDetail.split(".")[0]!, factor: view.tokenDetail.split(".")[1] as TokenFactor} : null;
  const setInspected = (value: {mint:string;factor:TokenFactor} | null) => update(v => ({...v,tokenDetail:value ? `${value.mint}.${value.factor}` : undefined}));
  const groupBy = view.tokenGroup ?? "kind";
  const setGroupBy = (value: "kind" | "issuer" | "flat") => update(v => ({...v,tokenGroup:value}));
  const collapsed = view.tokenGroups ?? {};
  const setCollapsed = (next: (c: Record<string,boolean>) => Record<string,boolean>) => update(v => ({...v,tokenGroups:next(v.tokenGroups ?? {})}));
  const narrow = useNarrow();
  const groupOf = (t: TokenTableRow) => groupBy === "flat" ? "all" : groupBy === "issuer" ? t.issuer ?? "Unlisted issuer" : t.kind ?? "unlisted";
  const groupLabel = (key: string) => groupBy === "kind" ? kindLabels[key] ?? "Unlisted" : groupBy === "flat" ? "All tokens" : key;
  const kinds = [...new Set(rows.map((r) => r.kind ?? "unlisted"))].sort();
  const shown = filterTokens(rows, factors, view).sort((a, b) =>
    groupLabel(groupOf(a)).localeCompare(groupLabel(groupOf(b))) ||
    a.symbol.localeCompare(b.symbol) || a.mint.localeCompare(b.mint));
  const groups = [...new Set(shown.map(groupOf))];
  const isCollapsed = (key: string) => collapsed[key] ?? (narrow && key !== groups[0] && !view.q);
  const selected = rows.find(t => t.mint === inspected?.mint);
  const filtered = view.q || view.tokenKind.length || view.tokenRating.length;
  return (
    <>
      <div className={styles.filterBar}>
        <label>Provenance <select value={view.tokenProvenance ?? ""} onChange={e => update(v => ({...v,tokenProvenance:(e.target.value || undefined) as ViewState["tokenProvenance"]}))}><option value="">All provenance</option><option value="native">Native</option><option value="bridged">Bridged</option></select></label>
        <label>Protocol use <select value={view.tokenUse ?? ""} onChange={e => update(v => ({...v,tokenUse:e.target.value || undefined}))}><option value="">All uses</option>{[...new Set(rows.flatMap(t=>t.exposure.flatMap(e=>e.roles)))].sort().map(role=><option key={role} value={role}>{role}</option>)}</select></label>
        <label className={styles.inlineSelect}>
          <span className="sr-only">Search tokens</span>
          <input
            type="search"
            data-shortcut="search"
            className={local.search}
            placeholder="Symbol, issuer or mint"
            value={view.q}
            onChange={(e) => {
              const q = e.target.value;
              update((v) => ({ ...v, q }));
            }}
          />
        </label>
        <div className={styles.filterGroup} role="group" aria-label="Token type" data-shortcut="filters">
          <span className={styles.filterLabel}>Type</span>
          {kinds.map((k) => (
            <button
              key={k}
              type="button"
              className={styles.chip}
              aria-pressed={view.tokenKind.includes(k)}
              onClick={() => update((v) => ({ ...v, tokenKind: toggle(v.tokenKind, k) }))}
            >
              {k === "unlisted" ? "Not listed" : (kindLabels[k] ?? k)}{" "}
              <span className={styles.chipCount}>{rows.filter((r) => (r.kind ?? "unlisted") === k).length}</span>
            </button>
          ))}
        </div>
        <div className={styles.filterGroup} role="group" aria-label="Any factor rated">
          <span className={styles.filterLabel}>Any factor</span>
          {RATINGS.map((r) => (
            <button
              key={r}
              type="button"
              className={styles.chip}
              aria-pressed={view.tokenRating.includes(r)}
              onClick={() => update((v) => ({ ...v, tokenRating: toggle(v.tokenRating, r) }))}
            >
              <RatingBadge rating={r} />
            </button>
          ))}
        </div>
        {filtered ? (
          <span className={styles.matchCount} role="status">
            {shown.length} of {rows.length} tokens
          </span>
        ) : null}
      </div>

      <nav aria-label="Token asset groups" className={local.groups}>
        <label>Group tokens <select value={groupBy} onChange={e => setGroupBy(e.target.value as typeof groupBy)}><option value="kind">Asset type</option><option value="issuer">Issuer</option><option value="flat">Flat list</option></select></label>
        <span>{shown.length} tokens</span>
        {groups.map(kind => <a key={kind} href={`#token-group-${encodeURIComponent(kind)}`} onClick={() => setCollapsed(c => ({...c, [kind]: false}))}>{groupLabel(kind)} ({shown.filter(t => groupOf(t) === kind).length})</a>)}
      </nav>
      <div className={styles.scroll} role="region" aria-label="Token factors" tabIndex={0}>
        <table className={styles.table}>
          <caption className="sr-only">Tokens the rated protocols accept, with the issuer&apos;s powers over each</caption>
          <thead>
            <tr>
              <th scope="col">Token</th>
              <th scope="col">Type</th>
              {factors.map((f) => (
                <th key={f} scope="col">
                  {factorLabels[f]}
                </th>
              ))}
              <th scope="col">Accepted by</th>
              <th scope="col">
                <span className="sr-only">Details</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {shown.map((t, index) => (
              <Fragment key={t.mint}>
                {index === 0 || groupOf(shown[index - 1]!) !== groupOf(t) ? <tr className={local.groupRow} id={`token-group-${encodeURIComponent(groupOf(t))}`} data-token-group={groupOf(t)}>
                  <th scope="row" colSpan={factors.length + 4}><button className={styles.tab} aria-expanded={!isCollapsed(groupOf(t))} onClick={() => setCollapsed(c => ({...c, [groupOf(t)]: !isCollapsed(groupOf(t))}))}>{groupLabel(groupOf(t))} · {shown.filter(r => groupOf(r) === groupOf(t)).length} tokens {isCollapsed(groupOf(t)) ? "+" : "−"}</button></th>
                </tr> : null}
                <tr data-mint={t.mint} hidden={isCollapsed(groupOf(t))}>
                  <th scope="row">
                    <span className={local.token}>
                      <span className={local.symbol}>
                        <Link href={`/tokens/${t.mint}/`}>{t.symbol}</Link>
                      </span>
                      <span className={local.sub}>
                        {t.issuer ?? "Issuer not listed"} · <span className="mono">{shortAddress(t.mint)}</span>
                      </span>
                    </span>
                  </th>
                  <td>
                    <span className={local.kind}>{t.kind ? (kindLabels[t.kind] ?? t.kind) : <span className={styles.muted}>Not listed</span>}</span>
                    <span className={local.sub}>
                      {t.program === "token2022" ? "Token-2022" : "SPL Token"} · {t.provenance}
                    </span>
                  </td>
                  {factors.map((f) => (
                    <td key={f} title={authorityWording(t.reasons[f])}>
                      <button className={styles.cellButton} aria-haspopup="dialog" aria-label={`${t.symbol}, ${factorLabels[f]}: ${ratingStyle[t.ratings[f]].label}, open evidence`} onClick={() => setInspected({mint:t.mint, factor:f})}><RatingBadge rating={t.ratings[f]} tone="pill" /></button>
                    </td>
                  ))}
                  <td>
                    {t.exposure.length ? (
                      <span className={local.exposure}>
                        {t.exposure.slice(0, 2).map((e) => (
                          <span key={e.id}>
                            <Link href={`/protocols/${e.id}/`}>{e.name}</Link> <span className={styles.muted}>({e.roles.join(", ")})</span>
                          </span>
                        ))}
                        {t.exposure.length > 2 ? <details><summary>+{t.exposure.length - 2} more protocols</summary>
                          {t.exposure.slice(2).map(e => <span key={e.id}><Link href={`/protocols/${e.id}/`}>{e.name}</Link> <span className={styles.muted}>({e.roles.join(", ")})</span></span>)}
                        </details> : null}
                      </span>
                    ) : (
                      <span className={styles.muted}>No rated protocol</span>
                    )}
                  </td>
                  <td>
                    <button
                      type="button"
                      className={local.expand}
                      aria-expanded={open === t.mint}
                      aria-controls={`token-${t.mint}`}
                      aria-label={`${open === t.mint ? "Hide" : "Show"} ${t.symbol} details`}
                      onClick={() => setOpen(open === t.mint ? null : t.mint)}
                    >
                      <span aria-hidden="true">{open === t.mint ? "−" : "+"}</span>
                    </button>
                  </td>
                </tr>
                {open === t.mint && !isCollapsed(groupOf(t)) ? (
                  <tr id={`token-${t.mint}`} className={local.detailRow}>
                    <td colSpan={factors.length + 4}>
                      <div className={local.detail}>
                        <div>
                          <p className={local.detailTitle}>Issuer-side powers over {t.symbol}</p>
                          {t.risks.length ? (
                            <ul className={local.risks}>
                              {t.risks.map((r) => (
                                <li key={r.text}>
                                  <RatingBadge rating={r.severity} /> {authorityWording(r.text)}
                                </li>
                              ))}
                            </ul>
                          ) : (
                            <p className={styles.muted}>No power over this token is rated High or Medium.</p>
                          )}
                        </div>
                        <div>
                          <p className={local.detailTitle}>Each factor</p>
                          <dl className={local.reasons}>
                            {factors.map((f) => (
                              <div key={f}>
                                <dt>{factorLabels[f]}</dt>
                                <dd><Finding text={t.reasons[f]} /></dd>
                              </div>
                            ))}
                          </dl>
                          <p>
                            <Link href={`/tokens/${t.mint}/`}>Full token page with evidence</Link>
                          </p>
                        </div>
                      </div>
                    </td>
                  </tr>
                ) : null}
              </Fragment>
            ))}
            {shown.length === 0 ? (
              <tr>
                <td colSpan={factors.length + 4} className={styles.muted}>
                  No token matches these filters.
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
      {selected && inspected ? <DetailPanel art="tokens" title={factorLabels[inspected.factor]} eyebrow={selected.symbol} onClose={() => setInspected(null)}>
        <RatingBadge rating={selected.ratings[inspected.factor]} />
        <PanelSection title="Observed configuration" tone="observed"><p>{selected.program === "token2022" ? "Token-2022" : "SPL Token"} · <code>{selected.mint}</code></p><Finding text={selected.reasons[inspected.factor]} />{selected.authorities?.[inspected.factor]?.kind === "single_key" ? <p>{SIGNER_BOUNDARY}</p> : null}</PanelSection>
        {selected.authorities?.[inspected.factor] ? <PanelSection title="Recorded authority path"><ControllerAuthorityTree controller={selected.authorities[inspected.factor]!} /></PanelSection> : null}
        {selected.additionalAuthorities?.filter(a=>a.factor === inspected.factor).map(a=><PanelSection key={a.label} title={a.label}><ControllerAuthorityTree controller={a.controller} /></PanelSection>)}
        <PanelSection title="Implication and criteria" tone="method"><p>This is an issuer-side power; it does not automatically determine an accepting protocol&apos;s rating.</p><Link href="/methodology/#tokens">Token criteria · v{selected.methodology ?? "unavailable"}</Link></PanelSection>
        <PanelSection title="What would improve the rating" tone="method"><p>{selected.scores?.[inspected.factor].next_step ?? "Unavailable: no scored improvement criterion recorded."}</p></PanelSection>
        <PanelSection title="Protocol usage"><LimitedItems label="accepting protocols">{selected.exposure.map(p => <li key={p.id}><Link href={`/protocols/${p.id}/`}>{p.name}</Link> · {p.roles.join(", ")}</li>)}</LimitedItems></PanelSection>
        <PanelSection title="Verified sources and snapshot"><EvidenceList evidence={selected.scores?.[inspected.factor].evidence ?? []} /><p>Snapshot {selected.observedAt ?? "unavailable"}</p><Link href={`/tokens/${selected.mint}/`}>Full authority record</Link></PanelSection>
      </DetailPanel> : null}
    </>
  );
}

type Props = Omit<Parameters<typeof TokenTableView>[0], "view" | "update">;

export function TokenTable(props: Props) {
  const [view, update] = useViewState();
  return <TokenTableView {...props} view={view} update={update} />;
}

export function TokenTableDefault(props: Props) {
  return <TokenTableView {...props} view={DEFAULT_VIEW} update={() => {}} />;
}
