import { ScopedLink } from "@/components/scope/ScopedLink";
import { CompareToggle } from "@/components/compare/CompareToggle";
import { ProtocolLogo, StageChip } from "@/components/ProtocolBits";
import { StageInspector } from '@/components/research/StageInspector';
import { RatingBadge } from "@/components/RatingBadge";
import { protocolFinding, nextStageCounts, authorityWording } from "@/lib/presentation";
import { changeDate, latestOnChain } from "@/lib/changes";
import { FACTOR_LABELS, STAGE_LABELS } from "@/lib/labels";
import type { ProtocolPage } from "@/lib/protocol";
import { CATEGORY_LABELS } from "@/lib/summary";
import { FACTORS } from "@/lib/types";
import type { Tab } from "@/lib/viewState";
import styles from "./ProtocolHeader.module.css";

/**
 * The research profile's header: identity, the figures a reader looks for
 * first, and the six factors at a glance, each linking to its full card. The
 * Stage sits apart from the factors: it is not a score.
 */
export function ProtocolHeader({
  page,
  logo,
  controller,
  snapshot,
}: {
  page: ProtocolPage;
  logo: string | null;
  /** The published controller summary, e.g. "Squads 5 of 10 · 24 h delay". */
  controller: string;
  snapshot: { date: string; slot: number | null; minSlot?: number | null; methodology: string };
}) {
  const category = page.category ? (CATEGORY_LABELS[page.category as Tab]?.label ?? page.category) : "Uncategorized";
  const chainLabel = page.chain === "solana" ? "Solana" : page.chain === "hyper_evm" ? "HyperEVM" : page.chain;
  const counts = nextStageCounts(page.inspection);
  const lastChange = latestOnChain(page.events, page.id);
  return (
    <header className={styles.header}>
      <div className={styles.identity}>
        <ProtocolLogo name={page.name} logo={logo} size={56} />
        <div className={styles.titleBlock}>
          <p className={styles.eyebrow}>
            {chainLabel} · {category}
          </p>
          <h1 className={styles.title}>{page.name}</h1>
          <div className={styles.meta}>
            {page.inspection ? <StageInspector inspection={page.inspection} /> : page.stage ? <StageChip stage={page.stage} /> : <span>Unassessed · native contract criteria</span>}
            {page.website ? (
              <a href={page.website} rel="noopener" className={styles.link}>
                {page.website.replace(/^https?:\/\//, "").replace(/\/$/, "")} <span aria-hidden="true">↗</span>
              </a>
            ) : null}
            <a href={`/protocols/${page.id}.md`} className={styles.link}>
              Markdown version
            </a>
            <ScopedLink href="/about/#corrections" className={styles.link}>
              Report an error
            </ScopedLink>
            <span className={styles.compare}>
              <CompareToggle id={page.id} name={page.name} />
              <span aria-hidden="true">Compare</span>
            </span>
          </div>
        </div>
      </div>

      <p className={styles.figureNote}>Snapshot {snapshot.date} · methodology v{snapshot.methodology}. Stages describe control arrangements, not code safety.</p>
      <dl className={styles.figures}>
        <div><dt>Upgrade controller</dt><dd className={styles.figureText}>{authorityWording(controller)}</dd><dd className={styles.figureNote}>Weakest control over the core programs</dd></div>
        <div><dt>Stage</dt><dd className={styles.figureText}>{page.stage ? STAGE_LABELS[page.stage] : "Unassessed"}</dd><dd className={styles.figureNote}>{counts ? `${counts.met} met · ${counts.unmet} verified unmet · ${counts.unresolved} unresolved for ${STAGE_LABELS[page.nextStage!]}` : page.unassessedReason ?? "Inspect the published criteria"} <a href="#stage">Requirements</a></dd></div>
        <div><dt>Last on-chain change</dt><dd className={styles.figureText}>{lastChange ? changeDate(lastChange) : "None recorded"}</dd><dd className={styles.figureNote}>Recorded history can be incomplete. <a href="#updates">Inspect changes</a></dd></div>
      </dl>
      <details className={styles.figureNote}><summary>Observation anchors and economic context</summary><p>{snapshot.slot !== null ? `${page.chain === "solana" ? "Solana slots" : "HyperEVM block"} ${snapshot.minSlot && snapshot.minSlot !== snapshot.slot ? `${snapshot.minSlot.toLocaleString("en-US")}–` : ""}${snapshot.slot.toLocaleString("en-US")}` : "Observation anchor unavailable"}. <a href="#value">Value</a> · <a href="#activity">Activity</a></p></details>

      {page.factors ? <section aria-labelledby="glance-h" className={styles.glance}>
        <h2 id="glance-h" className={styles.glanceTitle}>
          Six risk factors <span className={styles.glanceNote}>independent ratings, never combined into one score</span>
        </h2>
        <ul className={styles.factors}>
          {FACTORS.map((f) => (
            <li key={f} className={styles.factor} data-rating={page.factors![f].rating}>
              <a href={`#factor-${f}`} className={styles.factorLink}>
                <span className={styles.factorName}>{FACTOR_LABELS[f]}</span>
                <RatingBadge rating={page.factors![f].rating} tone="pill" />
                <span className={styles.factorReason}>{protocolFinding(page, f)}</span>
              </a>
            </li>
          ))}
        </ul>
      </section> : <ScopedLink href="/hyperevm/">Native contract authority assessment and evidence</ScopedLink>}
    </header>
  );
}
