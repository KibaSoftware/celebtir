import type { Metadata } from "next";
import { Suspense } from "react";
import { HashFocus } from "@/components/protocol/HashFocus";
import { CompareTray } from "@/components/compare/CompareTray";
import { ScopedFreshness } from "@/components/scope/ScopedFreshness";
import { Footer } from "@/components/reference/Footer";
import { TableLabels } from "@/components/mobile/TableLabels";
import { Shortcuts } from "@/components/shortcuts/Shortcuts";
import { SiteSearch } from "@/components/shortcuts/SiteSearch";
import { Brand } from "@/components/shell/Brand";
import { ChainSelector, ChainSelectorDefault } from "@/components/shell/ChainSelector";
import { MainNavigation, MainNavigationScoped } from "@/components/shell/MainNavigation";
import styles from "@/components/shell/Shell.module.css";
import { freshness, loadSnapshot } from "@/lib/data";
import { chainOfProtocol, chainOfToken, scope, type Chain } from "@/lib/scope";
import type { Snapshot } from "@/lib/types";
import { searchIndex } from "@/lib/search";
import { BRAND_HEADLINE, BRAND_SUPPORTING_LINE } from "@/lib/site";
import { cormorant, inter, jetbrainsMono } from "./fonts";
import "./globals.css";

export const metadata: Metadata = {
  // Absolute URLs for Open Graph images (UI-15); set at deploy.
  metadataBase: new URL(process.env.CELEBTIR_SITE_URL || "http://localhost:3000"),
  twitter: { card: "summary_large_image" },
  title: { default: `Celebtir: ${BRAND_HEADLINE}`, template: "%s · Celebtir" },
  applicationName: "Celebtir",
  description:
    `${BRAND_HEADLINE}. ${BRAND_SUPPORTING_LINE} Analysis of protocol upgradeability, administrative powers and value exposure on Solana and HyperEVM. Solana protocols are rated; HyperEVM protocols are listed with their contract facts. Every rating and fact links to on-chain evidence.`,
  openGraph: { siteName: "Celebtir", type: "website" },
  alternates: { types: { "application/atom+xml": [{ url: "/feeds/all.atom", title: "Celebtir change log" }] } },
};

/** Each protocol's chain, so its page can fix the picker (HUI-1). */
const protocolChains = (s: Snapshot): Record<string, Chain> =>
  Object.fromEntries([...Object.entries(s.facts.protocols).map(([id, p]) => [id, chainOfProtocol(p.chain)]), ...Object.keys(s.facts.tokens).flatMap(m => chainOfToken(s, m) ? [[`token:${m}`, chainOfToken(s, m)!]] : [])]);

export default function RootLayout({ children }: LayoutProps<"/">) {
  const snapshot = loadSnapshot();
  return (
    <html lang="en" className={`${inter.variable} ${cormorant.variable} ${jetbrainsMono.variable}`}>
      <body>
        <div className={styles.workspace}>
        <a className={styles.skip} href="#main">
          Skip to content
        </a>
        <header className={styles.header} data-research-shell>
          <div className={styles.bar}>
            <Brand />
            <Suspense fallback={<MainNavigation />}>
              <MainNavigationScoped />
            </Suspense>
            <div className={styles.tools}>
              <Suspense fallback={<ChainSelectorDefault />}>
                <ChainSelector protocolChains={protocolChains(snapshot)} />
              </Suspense>
              <Suspense fallback={<span>Search</span>}><SiteSearch items={searchIndex(snapshot)} /></Suspense>
            </div>
          </div>
        </header>
        <Shortcuts />
        <HashFocus prefix="" />
        <TableLabels />
        <main id="main" className={styles.main}>
          <ScopedFreshness protocolChains={protocolChains(snapshot)} variants={{ all: freshness(snapshot), solana: freshness(scope(snapshot, "solana")), hyperliquid: freshness(scope(snapshot, "hyperliquid")) }} />
          {children}
        </main>
        <div className={styles.footer}><Footer /></div>
        <CompareTray names={Object.fromEntries(Object.entries(snapshot.facts.protocols).map(([id, p]) => [id, p.name ?? id]))} />
        </div>
      </body>
    </html>
  );
}
