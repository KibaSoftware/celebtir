// Number formatting. Fixed locale, so build output does not depend on the
// machine that runs it.

const compact = new Intl.NumberFormat("en-US", {
  notation: "compact",
  maximumFractionDigits: 2,
  minimumFractionDigits: 0,
});

/** `$1.31B`, `$149.7K`, `$0`. */
export function formatUsd(n: number): string {
  return `$${compact.format(n)}`;
}

/** `7.84B`, `12.5K`: amounts that are not dollars. */
export function formatAmount(n: number): string {
  return compact.format(n);
}

/** A fraction as a signed percentage: 0.052 → `+5.2%`, -0.1 → `−10.0%`. */
export function formatChange(fraction: number): string {
  const pct = fraction * 100;
  if (Math.abs(pct) < 0.05) return "0.0%";
  return `${pct > 0 ? "+" : "−"}${Math.abs(pct).toFixed(1)}%`;
}

/** Basis points (as published) to a signed percentage. */
export function formatChangeBp(bp: number): string {
  return formatChange(bp / 10_000);
}

/** Same wording as the scorer's `duration`: `7 days`, `24 h`, `90 s`. */
export function formatDuration(secs: number): string {
  if (secs >= 2 * 86_400 && secs % 86_400 === 0) return `${secs / 86_400} days`;
  if (secs >= 3_600 && secs % 3_600 === 0) return `${secs / 3_600} h`;
  return `${secs} s`;
}

/** `KLen…gmjD`. The full address stays in the link and its title. */
export function shortAddress(address: string): string {
  return address.length > 12 ? `${address.slice(0, 4)}…${address.slice(-4)}` : address;
}
