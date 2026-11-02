import type { Metadata } from "next";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { TimeChart } from "@/components/charts/TimeChart";
import { Address, Empty, EvidenceList } from "@/components/Evidence";
import styles from "@/components/page/Page.module.css";
import { RatingBadge } from "@/components/RatingBadge";
import { LimitedItems } from "@/components/LimitedItems";
import { Finding } from "@/components/research/Finding";
import { authorityLabel, authorityWording, SIGNER_BOUNDARY } from "@/lib/presentation";
import { Termed } from "@/components/Term";
import { Card, EmptyState, PageHeader } from "@/components/ui/Ui";
import { loadHistory, loadSnapshot } from "@/lib/data";
import { SERIES_COLOR } from "@/lib/labels";
import { supplySeries, TOKEN_FACTOR_LABELS, TOKEN_KIND_LABELS, tokenPage } from "@/lib/pages";
import { TOKEN_FACTORS, type TokenHistoryFile } from "@/lib/types";

// One static page per token read (TOK-6). Typed by hand: generated route
// types only exist after `next build`.
type Props = { params: Promise<{ mint: string }> };
export const dynamicParams = false;

export function generateStaticParams() {
  return Object.keys(loadSnapshot().facts.tokens)
    .sort()
    .map((mint) => ({ mint }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const t = tokenPage(loadSnapshot(), (await params).mint);
  return { title: `${t.symbol} token`, description: `Who can mint, freeze, move or block ${t.symbol}, with on-chain evidence.` };
}

export default async function TokenPage({ params }: Props) {
  const { mint } = await params;
  const t = tokenPage(loadSnapshot(), mint);
  const supply = supplySeries(loadHistory<TokenHistoryFile>(`tokens/${mint}`));
  const days = loadHistory<TokenHistoryFile>(`tokens/${mint}`)?.days.length ?? 0;
  const prov = t.provenance.value;
  // UI-6: glossary terms link at their first use on the page.
  const seen = new Set<string>();

  return (
    <>
      <PageHeader art="tokens"
        eyebrow={`Token · ${t.identity ? (TOKEN_KIND_LABELS[t.identity.kind] ?? t.identity.kind) : "Type not listed"}`}
        title={t.symbol}
        subtitle={
          <>
            {t.identity ? `Issued by ${t.identity.issuer}. ` : ""}This page covers the issuer&apos;s powers over the token. The risks of each
            protocol that accepts it are on that protocol&apos;s own page.
          </>
        }
      />
      {t.identity?.kind === "stablecoin" && t.identity.issuer && t.controls.some(c => c.controller.kind !== "immutable") ? <p>Issuer-controlled mint and freeze powers can be intentional features of centrally issued stablecoins. They remain material trust assumptions.</p> : null}
      <h2 className={styles.h2}>Issuer-side risk summary</h2>
      {t.risks.length ? (
        <ul className={styles.risks}>
          {t.risks.map((r) => (
            <li key={r.text}>
              <RatingBadge rating={r.severity} />
              <span>
                <Termed text={authorityWording(r.text)} seen={seen} />
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <Empty>No risk sentences apply: no power over this token is rated High or Medium.</Empty>
      )}

      <h2 className={styles.h2}>Ratings of the issuer&apos;s powers</h2>
      <div className={styles.cards}>
        {TOKEN_FACTORS.map((f) => (
          <article key={f} className={styles.card} aria-labelledby={`tf-${f}`}>
            <h3 id={`tf-${f}`}>{TOKEN_FACTOR_LABELS[f]}</h3>
            <Link href={`/tokens/?tdetail=${mint}.${f}`}><RatingBadge rating={t.factors[f].rating} tone="pill" /></Link>
            <Finding text={t.factors[f].reason} seen={seen} />
            <details><summary>Rating criteria</summary><p className={styles.muted}>
              To improve: <Termed text={t.factors[f].next_step} seen={seen} />
            </p></details>
            <details><summary>Source evidence ({t.factors[f].evidence.length})</summary><EvidenceList evidence={t.factors[f].evidence} /></details>
          </article>
        ))}
      </div>

      <details><summary>Token identity, supply and observation anchors</summary><Card>
      <dl className={styles.facts}>
        <dt>Mint</dt>
        <dd>
          <Address address={t.mint} />
        </dd>
        <dt>Type</dt>
        <dd>{t.identity ? (TOKEN_KIND_LABELS[t.identity.kind] ?? t.identity.kind) : <span className={styles.muted}>Not listed</span>}</dd>
        <dt>Issuer</dt>
        <dd>{t.identity ? <a href={t.identity.issuer_url}>{t.identity.issuer}</a> : <span className={styles.muted}>Not listed</span>}</dd>
        <dt>Token program</dt>
        <dd>{t.program === "token2022" ? "Token-2022" : "SPL Token"}</dd>
        <dt>Supply</dt>
        <dd className={styles.mono}>{(t.supply / 10 ** t.decimals).toLocaleString("en-US", { maximumFractionDigits: 2 })}</dd>
        {t.identity ? (
          <>
            <dt>Source</dt>
            <dd>
              <a href={t.identity.source}>registry source</a>
            </dd>
          </>
        ) : null}
        <dt>Snapshot</dt>
        <dd className={styles.muted}>
          Read at slot{" "}
          <span className="mono">
            {Math.max(0, ...[...t.controls, ...t.extensions].flatMap((c) => c.evidence.map((e) => e.slot))).toLocaleString("en-US")}
          </span>
        </dd>
      </dl>
      </Card></details>

      <h2 className={styles.h2}>Supply, freeze and transfer authorities</h2>
      {t.controls.some(c => c.controller.kind === "single_key") ? <p><Termed text={SIGNER_BOUNDARY} seen={seen} /></p> : null}
      <p>These ratings describe concentration of technical control. They do not establish reserve quality, legal compliance or investment suitability.</p>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Power</th>
              <th scope="col">Held by</th>
              <th scope="col">What it can do</th>
              <th scope="col">Evidence</th>
            </tr>
          </thead>
          <tbody>
            {[...t.controls, ...t.extensions].map((c) => (
              <tr key={c.label}>
                <th scope="row">{c.label}</th>
                <td>
                  {c.controller.kind === "immutable" ? c.text : authorityLabel(c.controller)}{" "}
                  {c.controller.kind === "single_key" || c.controller.kind === "unrecognized" || c.controller.kind === "pda" || c.controller.kind === "program_controlled" ? <Address address={c.controller.key} /> : null}
                  {c.controller.kind === "multisig" ? <Address address={c.controller.address} /> : null}
                </td>
                <td className={styles.muted}>{c.controller.kind === "immutable" ? "nobody: renounced" : c.meaning}</td>
                <td>
                  <EvidenceList evidence={c.evidence} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {t.program === "token2022" ? (
        <p className={styles.muted}>
          <Termed
            text={[
              t.defaultFrozen ? "New token accounts start frozen (an allowlist)." : "",
              t.extensions.length === 0 ? "No Token-2022 extension carries a control power." : "",
              t.otherExtensions.length ? `Other extensions: ${t.otherExtensions.join(", ")}.` : "",
              t.unknownExtensions.length ? `Extension types Celebtir does not decode: ${t.unknownExtensions.join(", ")}.` : "",
            ]
              .filter(Boolean)
              .join(" ")}
            seen={seen}
          />
        </p>
      ) : (
        <p className={styles.muted}>
          <Termed text="A classic SPL Token mint: no Token-2022 extensions." seen={seen} />
        </p>
      )}

      <h2 className={styles.h2}>Provenance</h2>
      {prov.kind === "native" ? (
        <p>Native to Solana.</p>
      ) : (
        <p>
          Bridged through {prov.bridge}, backed by{" "}
          {/^0x[0-9a-fA-F]{40}$/.test(prov.backing_address) && /ethereum/i.test(prov.backing_chain) ? (
            <Address address={prov.backing_address} href={`https://etherscan.io/token/${prov.backing_address}`} />
          ) : (
            <span className={styles.mono}>{prov.backing_address}</span>
          )}{" "}
          on {prov.backing_chain}.
          Celebtir does not rate the bridge itself.
        </p>
      )}

      <h2 className={styles.h2}>Protocols that accept this token</h2>
      <p className={styles.muted}>Each protocol&apos;s own control and upgrade risks are rated separately, on its page.</p>
      {t.exposure.length ? (
        <LimitedItems label="accepting protocols">
          {t.exposure.map((e) => (
            <li key={e.id}>
              <Link href={`/protocols/${e.id}/`}>{e.name}</Link> <span className={styles.muted}>as {e.roles.join(", ")}</span>
            </li>
          ))}
        </LimitedItems>
      ) : (
        <Empty>No rated protocol lists this token.</Empty>
      )}

      <h2 className={styles.h2}>Supply</h2>
      {supply ? (
        <TimeChart
          title="Supply, last 30 days"
          description={`Circulating supply in whole ${t.symbol}, one point per daily snapshot.`}
          series={[{ key: "supply", label: "Supply", color: SERIES_COLOR }]}
          points={supply.map((d) => ({ date: d.date, values: { supply: d.supply } }))}
          unit="amount"
        />
      ) : (
        <EmptyState kind="history">
          <p>The supply chart appears once 30 daily snapshots exist ({days} so far).</p>
        </EmptyState>
      )}
    </>
  );
}
