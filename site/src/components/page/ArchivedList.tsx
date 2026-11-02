import { Empty } from "@/components/Evidence";
import { StageSwatch } from "@/components/ProtocolBits";
import { RatingBadge } from "@/components/RatingBadge";
import { FACTOR_LABELS, STAGE_LABELS } from "@/lib/labels";
import { FACTORS, type ArchivedFile } from "@/lib/types";
import styles from "./Page.module.css";

/** `/archived` (ADR 017): each protocol as it was last rated, with the reason. */
export function ArchivedList({ archived }: { archived: ArchivedFile["protocols"] }) {
  return (
    <>
      {archived.length ? (
        <div className={styles.cards}>
          {archived.map((a) => (
            <article key={a.id} className={styles.card} aria-labelledby={`a-${a.id}`}>
              <h3 id={`a-${a.id}`}>{a.name}</h3>
              <p>
                Archived {a.archived_on}: {a.reason} <a href={a.source}>Source</a>
              </p>
              {a.last_known?.stage && a.last_known.ratings ? (
                <>
                  <p className={styles.muted}>
                    Last rated {a.last_known.date} (methodology v{a.last_known.methodology_version}):{" "}
                    <StageSwatch stage={a.last_known.stage} /> {STAGE_LABELS[a.last_known.stage]}
                  </p>
                  <ul className={styles.risks}>
                    {FACTORS.map((f) => (
                      <li key={f}>
                        <RatingBadge rating={a.last_known!.ratings![f]} />
                        <span>{FACTOR_LABELS[f]}</span>
                      </li>
                    ))}
                  </ul>
                </>
              ) : (
                <Empty>It was archived before Celebtir rated it.</Empty>
              )}
            </article>
          ))}
        </div>
      ) : (
        <Empty>No protocol has been archived yet.</Empty>
      )}
    </>
  );
}
