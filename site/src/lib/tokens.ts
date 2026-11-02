// Design tokens: the Elvish Highwatch system (ADR 023, amending UI-24). Dark
// only. globals.css mirrors these as CSS variables, and tokens.test.ts checks
// every pairing for contrast. Components never use raw hex.

/** Midnight-navy surfaces, darkest first. */
export const surfaces = {
  /** Midnight Navy: the page. */
  page: "#080E1A",
  /** Observatory Navy: sections, tables, the navigation bar. */
  surface: "#141F30",
  /** Elevated navy: cards, panels, tooltips. */
  raised: "#19283C",
  /** Interactive hover surface. */
  hover: "#21334B",
} as const;

/** Text: Starlight White, Moon Silver, and a lifted Ancient Silver for small labels. */
export const ink = {
  primary: "#F5F7FC",
  secondary: "#C5CFDD",
  muted: "#98A6BA",
  /** Celestial Blue, lifted so links stay AA on the hover surface. */
  link: "#7EABEF",
} as const;

/** Brand identity colors: structure, selection and series, never ratings. */
export const brand = {
  gold: "#D4B46A",
  goldMuted: "#A68B55",
  blue: "#6599E8",
  sapphire: "#345FAD",
  silver: "#C5CFDD",
  ancientSilver: "#8B9CB2",
} as const;

export const chrome = {
  grid: "#1D2B40",
  axis: "#2C3D55",
  border: "#293B52",
} as const;

/** Status colors, reserved for ratings; always shown with an icon and a text label. */
export const status = {
  good: "#43CE9B",
  warning: "#F0BA5E",
  serious: "#EE8A5E",
  critical: "#ED6267",
  neutral: "#98A6BA",
  subdued: "#76839A",
} as const;

export type Rating = "low" | "medium" | "high" | "unknown" | "not_applicable";

export type IconKind = "check" | "triangle" | "square-cross" | "question" | "dash";

export const ratingStyle: Record<Rating, { color: string; label: string; icon: IconKind }> = {
  low: { color: status.good, label: "Low", icon: "check" },
  medium: { color: status.warning, label: "Medium", icon: "triangle" },
  high: { color: status.critical, label: "High", icon: "square-cross" },
  unknown: { color: status.neutral, label: "Unknown", icon: "question" },
  not_applicable: { color: status.subdued, label: "Not applicable", icon: "dash" },
};

/** WCAG 2.x relative luminance of a `#rrggbb` color. */
export function luminance(hex: string): number {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const [r, g, b] = c.map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrast(a: string, b: string): number {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
}
