import type { IconKind, Rating } from "@/lib/tokens";
import { ratingStyle, surfaces } from "@/lib/tokens";
import styles from "./RatingBadge.module.css";

const CUTOUT = surfaces.page;

// UI-24: a rating always carries text and an icon, never color alone. The
// icon wears the status color; the text stays in ink. Each rating has its own
// shape, so the icons stay distinct without color.
function Icon({ kind, color }: { kind: IconKind; color: string }) {
  const common = { width: 14, height: 14, viewBox: "0 0 16 16", "aria-hidden": true, focusable: false } as const;
  switch (kind) {
    case "check":
      return (
        <svg {...common}>
          <circle cx="8" cy="8" r="7" fill={color} />
          <path d="M4.5 8.2l2.2 2.2 4.8-4.8" stroke={CUTOUT} strokeWidth="1.8" fill="none" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "triangle":
      return (
        <svg {...common}>
          <path d="M8 1l7 13H1z" fill={color} />
          <path d="M8 6v3.5M8 11.6v.2" stroke={CUTOUT} strokeWidth="1.8" strokeLinecap="round" />
        </svg>
      );
    case "square-cross":
      return (
        <svg {...common}>
          <rect x="1" y="1" width="14" height="14" rx="3" fill={color} />
          <path d="M5 5l6 6M11 5l-6 6" stroke={CUTOUT} strokeWidth="1.8" strokeLinecap="round" />
        </svg>
      );
    case "question":
      return (
        <svg {...common}>
          <circle cx="8" cy="8" r="6.2" fill="none" stroke={color} strokeWidth="1.6" strokeDasharray="2.4 1.8" />
          <path d="M6.3 6.4a1.8 1.8 0 113 1.3c-.7.5-1.3.9-1.3 1.7M8 11.4v.2" stroke={color} strokeWidth="1.5" fill="none" strokeLinecap="round" />
        </svg>
      );
    case "dash":
      return (
        <svg {...common}>
          <path d="M4 8h8" stroke={color} strokeWidth="1.8" strokeLinecap="round" />
        </svg>
      );
  }
}

/** The rating's icon alone, for strips that label it elsewhere. */
export function RatingIcon({ rating }: { rating: Rating }) {
  const s = ratingStyle[rating];
  return <Icon kind={s.icon} color={s.color} />;
}

/**
 * A rating: icon plus label. `tone="pill"` draws it as a compact status label
 * on a tinted ground, for matrices and cards.
 */
export function RatingBadge({
  rating,
  compact = false,
  tone,
  label,
}: {
  rating: Rating;
  compact?: boolean;
  tone?: "pill";
  /** Replaces the rating's label, e.g. "Not rated in v0.1" (HUI-3). */
  label?: string;
}) {
  const s = ratingStyle[rating];
  const text = label ?? s.label;
  return (
    <span className={styles.badge} data-rating={rating} data-tone={tone} title={compact ? text : undefined}>
      <Icon kind={s.icon} color={s.color} />
      <span className={compact ? styles.srOnly : styles.label}>{text}</span>
    </span>
  );
}
