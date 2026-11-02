import { ScopedLink } from "@/components/scope/ScopedLink";
import { StaleWarning } from "./StaleWarning";
import styles from "./FreshnessBar.module.css";

export interface FreshnessInfo {
  date: string;
  finishedAt: string;
  slot: number | null;
  /** H3: the first and last finalized Solana slot the snapshot read. */
  slots?: { min: number; max: number } | null;
  blocks?: number | null;
  methodology: string;
  demo: boolean;
}

/**
 * UI-2: snapshot date, slot and methodology version on every page, as a
 * compact status row.
 */
export function FreshnessBar({ info, now }: { info: FreshnessInfo; now?: number }) {
  return (
    <div className={styles.bar} aria-label="Data freshness">
      <span className={styles.item}>
        <span className={styles.dot} aria-hidden="true" />
        <span className={styles.key}>Snapshot</span>{" "}
        <time dateTime={info.date} title={`Read by the run that finished ${info.finishedAt}`} className={styles.value}>
          {info.date}
        </time>
      </span>
      {(info.slot !== null || info.blocks != null) ? <details className={styles.anchors}><summary>Observation anchors</summary>
      {info.slot !== null && (
        <span className={styles.item} title="The range of finalized Solana slots this snapshot's reads span, protocols and token mints included; each fact also names its own slot.">
          <span className="mono">
            {info.slots && info.slots.min !== info.slots.max
              ? `Solana slots ${info.slots.min.toLocaleString("en-US")}–${info.slots.max.toLocaleString("en-US")}`
              : `Solana slot ${info.slot.toLocaleString("en-US")}`}
          </span>
        </span>
      )}
      {info.blocks != null && <span className={styles.item}>HyperEVM block {info.blocks.toLocaleString("en-US")}</span>}
      </details> : null}
      <span className={styles.item}>
        <ScopedLink href="/methodology/" className={styles.link}>
          methodology v{info.methodology}
        </ScopedLink>
      </span>
      {info.demo && <span className={styles.demo}>Demo data: the daily pipeline has not published yet</span>}
      <span className={styles.warning}>
        <StaleWarning finishedAt={info.finishedAt} now={now} />
      </span>
    </div>
  );
}
