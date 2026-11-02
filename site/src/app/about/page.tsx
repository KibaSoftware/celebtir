import type { Metadata } from "next";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import ref from "@/components/reference/Reference.module.css";
import styles from "@/components/page/Page.module.css";
import { Termed } from "@/components/Term";
import { CORRECTIONS, NOTICE, REPO_URL } from "@/lib/site";

export const metadata: Metadata = {
  title: "About",
  description: "What Celebtir is, what its ratings mean and don't, how to report an error, licences and privacy.",
};

export default function AboutPage() {
  const seen = new Set<string>();
  const t = (text: string) => <Termed text={text} seen={seen} />;
  return (
    <>
      <h1>About Celebtir</h1>
      <div className={ref.prose}>
        <p>
          {t(
            "Celebtir is an open scorecard for Solana and Hyperliquid DeFi protocols. It reads who can change each protocol's code and settings straight from the chain, how fast they can do it, and links every rating to the account it came from.",
          )}
        </p>

        <h2 className={styles.h2} id="disclaimer">
          What the ratings are, and are not
        </h2>
        <p>
          <strong>{NOTICE}</strong>
        </p>
        <ul>
          <li>{t("Ratings describe on-chain configuration: keys, multisigs, time locks, verified builds and oracle sources. They are not an audit and say nothing about whether the code is correct.")}</li>
          <li>{t("A Low rating means the observed configuration meets that factor's Low criteria, such as stronger voting requirements or longer delays. It does not mean a protocol is safe to use.")}</li>
          <li>{t("Data comes from public sources and may contain errors. Check the evidence links before relying on any rating.")}</li>
        </ul>

        <h2 className={styles.h2} id="corrections">
          Corrections policy
        </h2>
        <p>{t("If a rating or fact is wrong, report it with the on-chain evidence that shows it.")}</p>
        <ul>
          <li>
            Reports are acknowledged within {CORRECTIONS.acknowledgeHours} hours and resolved within {CORRECTIONS.resolveDays} days.
          </li>
          <li>Every report and its outcome are logged publicly, in the repository&apos;s issue tracker.</li>
          <li>
            Accepted fixes ship as registry edits or methodology changes and appear in the <Link href="/changelog/">change log</Link>. A
            methodology change also gets a <Link href="/methodology/#changelog">methodology changelog</Link> entry.
          </li>
          <li>{t("Ratings never change because of announcements or private messages: only on-chain evidence moves them.")}</li>
        </ul>
        <p>
          {/* H1: the corrections channel. Release builds fail without it (scripts/check-release.mjs). */}
          {REPO_URL ? (
            <>
              To report one, <a href={`${REPO_URL}/issues/new`}>open an issue on GitHub</a>. The public repository, with the code, registry and
              every published dataset, is <a href={REPO_URL}>{REPO_URL.replace(/^https:\/\//, "")}</a>.
            </>
          ) : (
            <>To report one, open an issue in the project&apos;s public GitHub repository. This build was made without a repository address, so it is not a release build.</>
          )}
        </p>

        <h2 className={styles.h2} id="sources">
          Sources
        </h2>
        <ul>
          <li>{t("Control facts: Solana and Hyperliquid, read over public APIs, with a slot or block reference for every read.")}</li>
          <li>{t("Verified builds: OtterSec's verification API.")}</li>
          <li>{t("TVL, fees and volume: DefiLlama. Context only; they never change a rating.")}</li>
          <li>Evidence links open Solscan for Solana, HyperEVMScan for HyperEVM and the Hyperliquid explorer for HyperCore.</li>
        </ul>

        <h2 className={styles.h2} id="licences">
          Licences
        </h2>
        <p>
          Code is dual-licensed under MIT or Apache-2.0. Data is published under CC BY 4.0: reuse it freely with attribution to Celebtir. See the{" "}
          <Link href="/api/">API page</Link> for the data files.
        </p>

        <h2 className={styles.h2} id="privacy">
          Privacy
        </h2>
        <ul>
          <li>No cookies, no analytics, no accounts and no personal data.</li>
          <li>Nothing is loaded from another site: fonts, scripts and data are all served from here.</li>
          <li>The compare tray is kept in your browser&apos;s local storage and never leaves your device.</li>
        </ul>
      </div>
    </>
  );
}
