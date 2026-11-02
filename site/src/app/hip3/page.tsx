import { ScopedLink } from "@/components/scope/ScopedLink";
import type { Metadata } from "next";
import { Hip3List } from "@/components/page/Hip3List";
import { PageHeader } from "@/components/ui/Ui";
import styles from "@/components/research/Research.module.css";
import { loadSnapshot } from "@/lib/data";

export const metadata: Metadata = {
  title: "HIP-3 perp DEXs",
  description:
    "Every builder-deployed perpetuals exchange on Hyperliquid: who deploys it, who can set its prices, its open-interest caps and the HYPE its deployer address has staked.",
};

// HUI-3: listed from the info API, never rated in methodology v0.1 (ADR 024 D1).
export default function Hip3Page() {
  const hip3 = loadSnapshot().facts.hip3;
  return (
    <>
      <PageHeader art="hip3" title="HIP-3 perp DEXs" eyebrow="Native HyperCore market research" subtitle="Deployer permissions, markets and oracle control. Recorded facts are separate from ratings: no adopted HIP-3 methodology or Stage applies." />
      {!hip3 ? <section className={styles.panel}><h2>Coverage required · Unassessed</h2><p>No published HIP-3 observations are included in this snapshot. This does not establish zero markets or Low risk.</p></section> : null}
      <p>DEXs appear after a successful Hyperliquid collection is included in a published snapshot. No publication date is promised. <ScopedLink href="/methodology/#hyperliquid">How Hyperliquid coverage works</ScopedLink>.</p>
      <div className={styles.grid}>
        <section className={styles.panel}><h2>Market and deployer identity</h2><p>Published records identify each DEX, deployer, delegated actions, market caps and stake. Inspect account and API anchors below when available.</p></section>
        <section className={styles.panel}><h2>Oracle control</h2><p>Oracle-updater delegation is a recorded authority fact. A verified history of oracle changes requires comparable observations.</p></section>
        <section className={styles.panel}><h2>Halts and settlement</h2><p>Unassessed: the current export does not establish a verified history of halt or settlement actions.</p></section>
        <section className={styles.panel}><h2>Collateral and permissions</h2><p>Do not infer asset-movement powers from a deployer label. Native permission semantics and supporting evidence are required.</p></section>
      </div>
      <Hip3List hip3={hip3} />
    </>
  );
}
