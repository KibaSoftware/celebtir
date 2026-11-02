import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { StackedBar } from "@/components/ui/Charts";
import { FACTOR_LABELS } from "@/lib/labels";
import { factorDistribution, RATINGS } from "@/lib/overview";
import type { ProtocolRow } from "@/lib/summary";
import { FACTORS } from "@/lib/types";
import { ratingStyle } from "@/lib/tokens";
import styles from "./Research.module.css";

export function FactorDistribution({ rows, art = "archive" }: { rows: ProtocolRow[]; art?: "archive" | "risk" }) {
  const distribution = factorDistribution(rows.filter(r => r.chain === "solana"));
  return <section className={`${styles.panel} ${art === "risk" ? styles.risk : styles.archive}`} aria-label="Six-factor risk landscape">
    <h2>Six-factor risk landscape</h2>
    <p className={styles.note}>Counts per assessed Solana protocol · Low / Medium / High / Unknown / N/A</p>
    <dl className={styles.factors}>{FACTORS.map(f => <div key={f}>
      <dt><Link href={`/risk/?factor=${f}`}>{FACTOR_LABELS[f]}</Link></dt>
      <dd><StackedBar compact label={`${FACTOR_LABELS[f]}, protocols per rating`} unit="count" segments={RATINGS.map(r => ({ key: r, label: ratingStyle[r].label, value: distribution[f][r], color: ratingStyle[r].color, pattern: r === "unknown" ? "dashed" as const : undefined }))} /></dd>
      <dd aria-label={RATINGS.map(r => `${ratingStyle[r].label}: ${distribution[f][r]}`).join(", ")}>{RATINGS.map(r => distribution[f][r]).join(" / ")}</dd>
    </div>)}</dl>
  </section>;
}
