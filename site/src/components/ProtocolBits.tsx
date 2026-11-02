import { ReasonText } from "./ReasonText";
import { StageInspector } from './research/StageInspector';
import { RatingBadge, RatingIcon } from "./RatingBadge";
import { Tooltip } from "./Tooltip";
import { FACTOR_LABELS, FACTOR_SHORT, STAGE_COLORS, STAGE_LABELS, STAGE_NOTE } from "@/lib/labels";
import { ratingStyle, type Rating } from "@/lib/tokens";
import { FACTORS, type Factor, type Stage } from "@/lib/types";
import styles from "./ProtocolBits.module.css";

/** No third-party logos (LCH-6): a monogram from the protocol's name. */
export function Monogram({ name, size = 28 }: { name: string; size?: number }) {
  const letters = name
    .split(/[\s-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]!.toUpperCase())
    .join("");
  return (
    <span className={styles.monogram} style={{ width: size, height: size, fontSize: Math.round(size * 0.36) }} aria-hidden="true">
      {letters || "?"}
    </span>
  );
}

/**
 * The protocol's mark: its logo from `public/logos/<id>.(svg|png)` once one
 * has been added (src/lib/logos.ts resolves it at build time), else the
 * monogram placeholder.
 */
export function ProtocolLogo({ name, logo, size = 28 }: { name: string; logo?: string | null; size?: number }) {
  if (!logo) return <Monogram name={name} size={size} />;
  // Static export: next/image would not resize it (images.unoptimized).
  // eslint-disable-next-line @next/next/no-img-element
  return <img src={logo} alt="" width={size} height={size} className={styles.logo} />;
}

/**
 * The six factor ratings as six labelled cells (UI-10): each shows its
 * two-letter code and the rating's icon, and is its own keyboard tooltip
 * naming the factor, the rating and the reason. Six independent factors,
 * never summed into one score.
 */
export function FactorStrip({
  ratings,
  reasons,
  protocolId,
}: {
  ratings: Record<Factor, Rating>;
  reasons: Partial<Record<Factor, string>>;
  protocolId?: string;
}) {
  const label = `Risk: ${FACTORS.map((f) => `${FACTOR_LABELS[f]} ${ratingStyle[ratings[f]].label}`).join(", ")}`;
  return (
    <span className={styles.strip} role="group" aria-label={label}>
      {FACTORS.map((f) => {
        const r = ratings[f];
        return (
          <Tooltip
            key={f}
            href={protocolId ? `/risk/?detail=${protocolId}.${f}` : undefined}
            label={`${FACTOR_SHORT[f]} ${FACTOR_LABELS[f]}: ${ratingStyle[r].label}`}
            trigger={
              <span className={styles.cell} data-rating={r}>
                <RatingIcon rating={r} />
                <span className={styles.code}>{FACTOR_SHORT[f]}</span>
              </span>
            }
          >
            <span className={styles.tipHead}>
              <span className={styles.tipFactor}>{FACTOR_LABELS[f]}</span>
              <RatingBadge rating={r} />
            </span>
            {reasons[f] ? <span className={styles.reason}><ReasonText text={reasons[f]!} /></span> : null}
          </Tooltip>
        );
      })}
    </span>
  );
}

/** The six codes with their names, as a key aligned over FactorStrip cells. */
export function FactorStripHeader() {
  return (
    <span className={styles.stripHeader}>
      {FACTORS.map((f) => (
        <abbr key={f} title={FACTOR_LABELS[f]}>
          {FACTOR_SHORT[f]}
        </abbr>
      ))}
    </span>
  );
}

export function StageSwatch({ stage }: { stage: Stage }) {
  return <span className={styles.swatch} data-stage={stage} style={{ background: STAGE_COLORS[stage] }} aria-hidden="true" />;
}

/** A Stage chip in the lavender ramp, never a status color: Stage 0 is not High. */
export function StageChip({ stage }: { stage: Stage }) {
  return (
    <span className={styles.stage} data-stage={stage}>
      <StageSwatch stage={stage} />
      {STAGE_LABELS[stage]}
    </span>
  );
}

/** The Stage, with what the next Stage still needs (UI-10). */
export function StageBadge({ stage, next, missing, inspection }: { stage: Stage; next: Stage | null; missing: string[]; inspection?: import('@/lib/scoreReview').StageInspection }) {
  if (inspection) return <StageInspector inspection={inspection} />;
  return (
    <Tooltip label={STAGE_LABELS[stage]} trigger={<StageChip stage={stage} />}>
      {next && missing.length ? (
        <>
          <span className={styles.heading}>Missing for {STAGE_LABELS[next]}:</span>
          <span className={styles.list}>
            {missing.map((m) => (
              <span key={m} className={styles.missing}>
                {m}
              </span>
            ))}
          </span>
        </>
      ) : (
        <span className={styles.heading}>{next ? `Nothing recorded as missing for ${STAGE_LABELS[next]}.` : "Highest Stage."}</span>
      )}
      <span className={styles.note}>{STAGE_NOTE}</span>
    </Tooltip>
  );
}
