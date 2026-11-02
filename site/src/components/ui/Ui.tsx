import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import type { CSSProperties, ReactNode } from "react";
import { Tooltip } from "@/components/Tooltip";
import styles from "./Ui.module.css";

/**
 * The shared presentation kit (ADR 023): section headers, cards, KPI cards
 * and empty states. Server components; no data logic.
 */

export type ResearchArt = "archive" | "risk" | "value" | "activity" | "tokens" | "compare" | "hip3" | "changes" | "methodology" | "hyperevm";
export function PageHeader({ title, subtitle, eyebrow, children, art = "archive" }: { title: ReactNode; subtitle?: ReactNode; eyebrow?: ReactNode; children?: ReactNode; art?: ResearchArt }) {
  return (
    <header className={styles.pageHeader} style={{"--research-art": `url('/art/${art}.webp')`} as CSSProperties}>
      <div>
        {eyebrow ? <div className={styles.eyebrow}>{eyebrow}</div> : null}
        <h1 className={styles.pageTitle}>{title}</h1>
        {subtitle ? <p className={styles.pageSubtitle}>{subtitle}</p> : null}
      </div>
      {children ? <div className={styles.pageActions}>{children}</div> : null}
    </header>
  );
}

/** A section title with the gold marker, an optional unit or source note and actions. */
export function SectionHeader({
  id,
  title,
  note,
  actions,
  level = 2,
}: {
  id?: string;
  title: ReactNode;
  note?: ReactNode;
  actions?: ReactNode;
  level?: 2 | 3;
}) {
  const H = level === 2 ? "h2" : "h3";
  return (
    <div className={styles.sectionHeader}>
      <div className={styles.sectionTitleRow}>
        <H id={id} className={level === 2 ? styles.sectionTitle : styles.subTitle}>
          <span className={styles.marker} aria-hidden="true" />
          {title}
        </H>
        {note ? <p className={styles.sectionNote}>{note}</p> : null}
      </div>
      {actions ? <div className={styles.sectionActions}>{actions}</div> : null}
    </div>
  );
}

export function Card({ children, className, as: As = "div", ...rest }: { children: ReactNode; className?: string; as?: "div" | "section" | "article"; "aria-labelledby"?: string }) {
  return (
    <As className={[styles.card, className].filter(Boolean).join(" ")} {...rest}>
      {children}
    </As>
  );
}

/**
 * A KPI: title, figure, context line, an optional real change, an
 * explanation on focus and a link to the detail. Only real figures: callers
 * pass `null` deltas when history does not cover the period.
 */
export function MetricCard({
  title,
  value,
  context,
  delta,
  help,
  href,
  hrefLabel,
}: {
  title: string;
  value: ReactNode;
  context?: ReactNode;
  delta?: { text: string; direction: "up" | "down" | "flat"; period: string } | null;
  help?: ReactNode;
  href?: string;
  hrefLabel?: string;
}) {
  return (
    <article className={styles.metric} aria-label={title}>
      <div className={styles.metricHead}>
        <h3 className={styles.metricTitle}>{title}</h3>
        {help ? (
          <Tooltip label={`About ${title}`} trigger={<span className={styles.help} aria-hidden="true">?</span>}>
            {help}
          </Tooltip>
        ) : null}
      </div>
      <div className={styles.metricValue}>{value}</div>
      {delta ? (
        <div className={styles.metricDelta} data-direction={delta.direction}>
          {delta.text} <span className={styles.metricPeriod}>{delta.period}</span>
        </div>
      ) : null}
      {context ? <div className={styles.metricContext}>{context}</div> : null}
      {href ? (
        <Link href={href} className={styles.metricLink}>
          {hrefLabel ?? "Details"} <span aria-hidden="true">→</span>
        </Link>
      ) : null}
    </article>
  );
}

export type EmptyKind = "history" | "coverage" | "unsupported" | "failed" | "none";

const EMPTY_TITLES: Record<EmptyKind, string> = {
  history: "Not enough history yet",
  coverage: "Not covered by the provider",
  unsupported: "Not measured",
  failed: "Data could not be retrieved",
  none: "Nothing to show",
};

/**
 * A designed empty state. Each kind has its own title so "no history yet",
 * "provider doesn't report it" and "retrieval failed" never look alike.
 */
export function EmptyState({ kind, title, children }: { kind: EmptyKind; title?: string; children: ReactNode }) {
  return (
    <div className={styles.empty} data-empty={kind}>
      <EmptyGlyph kind={kind} />
      <div>
        <p className={styles.emptyTitle}>{title ?? EMPTY_TITLES[kind]}</p>
        <div className={styles.emptyBody}>{children}</div>
      </div>
    </div>
  );
}

function EmptyGlyph({ kind }: { kind: EmptyKind }) {
  const common = { width: 28, height: 28, viewBox: "0 0 28 28", "aria-hidden": true, focusable: false, className: styles.emptyGlyph } as const;
  if (kind === "history") {
    return (
      <svg {...common}>
        <path d="M4 22h20" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
        <circle cx="9" cy="15" r="2" fill="currentColor" />
        <path d="M13 15h10" stroke="currentColor" strokeWidth="1.4" strokeDasharray="2 3" strokeLinecap="round" />
      </svg>
    );
  }
  if (kind === "failed") {
    return (
      <svg {...common}>
        <circle cx="14" cy="14" r="9" fill="none" stroke="currentColor" strokeWidth="1.4" />
        <path d="M14 9v6M14 18.5v.5" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
      </svg>
    );
  }
  return (
    <svg {...common}>
      <rect x="5" y="5" width="18" height="18" rx="4" fill="none" stroke="currentColor" strokeWidth="1.4" strokeDasharray="3 3" />
    </svg>
  );
}

/** A labelled chip for filter bars and tags. */
export function Chip({ children, tone }: { children: ReactNode; tone?: "gold" | "blue" }) {
  return (
    <span className={styles.chip} data-tone={tone}>
      {children}
    </span>
  );
}
