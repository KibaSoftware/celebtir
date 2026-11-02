import { ScopedLink } from "@/components/scope/ScopedLink";
import type { Metadata } from "next";
import { readFileSync } from "node:fs";
import path from "node:path";
import { DATA_STATES, DataState, type DataStateKind } from "@/components/DataState";
import { Markdown } from "@/components/reference/Markdown";
import ref from "@/components/reference/Reference.module.css";
import styles from "@/components/page/Page.module.css";
import { RatingBadge } from "@/components/RatingBadge";
import { ReasonText } from "@/components/ReasonText";
import { Termed } from "@/components/Term";
import { Card, PageHeader } from "@/components/ui/Ui";
import { loadSnapshot } from "@/lib/data";
import { FACTOR_LABELS, STAGE_LABELS, STAGE_NOTE } from "@/lib/labels";
import { STAGE_ZERO_EXPLANATION } from '@/lib/scoreReview';
import { parseMarkdown } from "@/lib/markdown";
import { factorRules, loadMethodology, methodologyChangelog, methodologyVersions, stageRequirements } from "@/lib/methodology";
import { factorRows } from "@/lib/pages";
import { FACTORS, type Stage } from "@/lib/types";
import local from "./methodology.module.css";

export const metadata: Metadata = {
  title: "Methodology",
  description: "How Celebtir rates DeFi protocols: six factors, Stages, value classes and every threshold, versioned; which chains are rated.",
};

const repo = path.resolve(process.cwd(), "..");


export default function MethodologyPage() {
  const s = loadSnapshot();
  const version = s.scores.methodology_version;
  const m = loadMethodology(version);
  const toml = readFileSync(path.join(repo, "methodology", `v${version}.toml`), "utf8");
  const seen = new Set<string>();
  const t = (text: string) => <Termed text={text} seen={seen} />;
  const reqs = stageRequirements(s);
  const stages = [...new Set(reqs.map((r) => r.stage))] as Stage[];

  const rules = factorRules(m);
  const examples = Object.fromEntries(FACTORS.map((f) => [FACTOR_LABELS[f], factorRows(s, f)]));
  const TOC = [
    ["introduction", "Introduction"],
    ["principles", "Principles"],
    ["scope", "Assessment scope"],
    ["factors", "Six risk factors"],
    ["criteria", "Rating criteria"],
    ["stages", "Stages"],
    ["tokens", "Token factors"],
    ["evidence", "Evidence collection"],
    ["hyperliquid", "Hyperliquid"],
    ["limitations", "Limitations"],
    ["history", "Version history"],
  ] as const;

  return (
    <>
      <PageHeader art="methodology"
        eyebrow="Research methodology"
        title={`Methodology v${version}`}
        subtitle={t(
          "Each protocol gets a Stage as its headline and a Low, Medium or High rating on six factors. There is no single score: a composite hides trade-offs.",
        )}
      />
      <div className={local.layout}>
        <nav aria-label="On this page" className={local.toc}>
          <p className={local.tocTitle}>On this page</p>
          <ol>
            {TOC.map(([id, label]) => (
              <li key={id}>
                <a href={`#${id}`}>{label}</a>
              </li>
            ))}
          </ol>
        </nav>
        <div className={local.body}>
          <section id="introduction" className={local.section} aria-labelledby="introduction-h">
            <h2 id="introduction-h" className={styles.h2}>
              Introduction
            </h2>
            <p className={ref.prose}>
              {t(
                "Celebtir reads who controls each protocol straight from the chain: who can upgrade its programs, how fast, who holds its admin keys and where its prices come from. Every rating links to the account and slot it was read from.",
              )}{" "}
              Published versions: {methodologyVersions().map((v) => `v${v}`).join(", ")}. Thresholds on this page are read from methodology/v{version}.toml,
              the same file the scorer uses.
            </p>
          </section>

          <section id="principles" className={local.section} aria-labelledby="principles-h">
            <h2 id="principles-h" className={styles.h2}>
              Principles
            </h2>
            <ul className={`${ref.prose} ${local.principles}`}>
              <li>{t("Evidence, not claims. Every rating links to the on-chain account and slot it was read from. Docs and announcements never change a rating.")}</li>
              <li>{t("Weakest link. A protocol is rated by its weakest core program: one single-key program caps the whole protocol.")}</li>
              <li>
                {t(
                  "Effective controller. Control is followed to its end. If a multisig's config authority is one key, that key is the real controller, with no delay.",
                )}
              </li>
              <li>
                {t(
                  "Missing data is Unknown, never Low or Not applicable. An empty list in the registry is not a finding that nothing exists: admin powers and price feeds rate Unknown until they are read, or until a sourced registry review records that there are none. Other Unknown factors cap the Stage at 0. A failed read is never guessed; an observed but unrecognized controller is distinct from an unavailable read.",
                )}
              </li>
              <li>{t("Independent factors. The six ratings are never added up: each answers its own question.")}</li>
            </ul>
          </section>

          <section id="scope" className={local.section} aria-labelledby="scope-h">
            <h2 id="scope-h" className={styles.h2}>
              Assessment scope
            </h2>
            <div className={local.scopeGrid}>
              <Card>
                <h3 className={local.cardTitle}>Protocols</h3>
                <p className={ref.prose}>
                  {t("The protocols in the registry: their core programs, admin accounts, price feeds and the programs they call with user funds. Solana protocols are rated; HyperEVM protocols are listed with their contract facts and no ratings.")}
                </p>
              </Card>
              <Card>
                <h3 className={local.cardTitle}>Tokens</h3>
                <p className={ref.prose}>
                  {t(
                    "Tokens are rated on four factors and get no Stage: supply control (the mint authority), freeze and seize (freeze authority, permanent delegate, pause), transfer restrictions (transfer hook, transfer fee, default-frozen accounts) and provenance (native or bridged). Supply control and freeze and seize: Low when renounced, or held by a verified program-derived address whose controlling program is immutable or rates Low on upgrade control; Medium when held by a multisig meeting the Medium bar, or a program rated Medium; High for a single key, a weaker multisig, an unrecognized holder, a program rated High, or any permanent delegate; Unknown for an off-curve (program-derived) authority whose controlling program is not resolved. Transfer restrictions: Low with none, otherwise at least Medium. Provenance: Low native, Medium bridged.",
                  )}
                </p>
              </Card>
              <Card>
                <h3 className={local.cardTitle}>Value by control type</h3>
                <p className={ref.prose}>
                  {t(
                    "Each protocol's TVL is assigned to the class of its weakest core program's controller: single key; unrecognized (unreadable, unsupported, or a program-derived address whose controlling program is not resolved); multisig with no delay (no time lock, or a config authority that bypasses it); multisig with a delay under 24 hours; multisig or timelock with a delay of 24 hours or more (the Medium upgrade-delay bar); or immutable. Only the last two count as protected.",
                  )}{" "}
                  TVL and activity are context: they never change a rating.
                </p>
              </Card>
            </div>
          </section>

          <section id="factors" className={local.section} aria-labelledby="factors-h">
            <h2 id="factors-h" className={styles.h2}>
              Six risk factors
            </h2>
            <div className={local.factorList}>
              {rules.map((r) => (
                <article key={r.factor} id={`factor-${FACTORS.find((f) => FACTOR_LABELS[f] === r.factor)}`} className={local.factor} aria-labelledby={`rule-${r.factor}`}>
                  <h3 id={`rule-${r.factor}`} className={local.factorTitle}>
                    {r.factor}
                  </h3>
                  <p className={local.question}>{r.question}</p>
                  <dl className={local.levels}>
                    {(
                      [
                        ["low", r.low],
                        ["medium", r.medium],
                        ["high", r.high],
                        ["unknown", DATA_STATES.unknown.explain],
                        ["not_applicable", DATA_STATES.not_applicable.explain],
                      ] as const
                    ).map(([rating, text]) => (
                      <div key={rating}>
                        <dt>
                          <RatingBadge rating={rating} tone="pill" />
                        </dt>
                        <dd>{t(text)}</dd>
                      </div>
                    ))}
                  </dl>
                  {examples[r.factor]?.length ? (
                    <div className={local.examples}>
                      <p className={local.examplesTitle}>In the current snapshot</p>
                      <ul>
                        {examples[r.factor]!.map((e) => (
                          <li key={e.id}>
                            <ScopedLink href={`/protocols/${e.id}/#risk-analysis`}>{e.name}</ScopedLink> <RatingBadge rating={e.score.rating} />{" "}
                            <span className={styles.muted}>{t(e.score.reason)}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  ) : null}
                </article>
              ))}
            </div>
          </section>

          <section id="criteria" className={local.section} aria-labelledby="criteria-h">
            <h2 id="criteria-h" className={styles.h2}>
              Rating criteria
            </h2>
            <p className={styles.muted}>Every factor side by side. Thresholds are read from methodology/v{version}.toml.</p>
            <div className={styles.scroll}>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th scope="col">Factor</th>
                    <th scope="col">Question</th>
                    <th scope="col">Low</th>
                    <th scope="col">Medium</th>
                    <th scope="col">High</th>
                  </tr>
                </thead>
                <tbody>
                  {rules.map((r) => (
                    <tr key={r.factor}>
                      <th scope="row">{r.factor}</th>
                      <td>{r.question}</td>
                      <td>{r.low}</td>
                      <td>{r.medium}</td>
                      <td>{r.high}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section id="tokens" className={local.section} aria-labelledby="tokens-h">
            <h2 id="tokens-h">Token factors · methodology v{version}</h2>
            <p>Tokens have four independent factors and no Stage. The recorded scorer output supplies the reason, improvement criterion and evidence for each finding. Token issuer powers do not automatically determine the rating of an accepting protocol.</p>
            {Object.entries(s.scores.tokens).slice(0,1).map(([mint,score]) => <dl key={mint}>{Object.entries(score.factors).map(([factor,finding]) => <div key={factor}><dt>{factor.replaceAll("_"," ")}</dt><dd><p>Recorded example ({score.symbol}): <ReasonText text={finding.reason} /></p><p>Improvement criterion: <ReasonText text={finding.next_step} /></p><ScopedLink href={`/tokens/${mint}/`}>Inspect source observations</ScopedLink></dd></div>)}</dl>)}
          </section>
          <section id="stages" className={local.section} aria-labelledby="stages-h">
            <h2 id="stages-h" className={styles.h2}>
              Stages
            </h2>
            <p className={local.stageNote}>{STAGE_NOTE} A Stage is not a safety score and is never combined with the six factors.</p>
             <p>{STAGE_ZERO_EXPLANATION}</p>
            <ol className={local.progression} aria-label="Stage progression">
              {(["stage_0", "stage_1", "stage_2"] as Stage[]).map((st) => (
                <li key={st}>
                  <span className={local.progressionMark} aria-hidden="true" />
                  <span className={local.progressionLabel}>{STAGE_LABELS[st]}</span>
                  <span className={local.progressionText}>
                    {st === "stage_0"
                      ? "Rated, with requirements for Stage 1 still missing."
                      : st === "stage_1"
                        ? "Changes need many signers and give users time to react."
                        : "Meets the published Stage 2 control requirements."}
                  </span>
                </li>
              ))}
            </ol>
            <p className={ref.prose}>
              {t(
                "Unrated means at least one Stage 0 requirement is unmet: the core programs are not identified, a core program has no public source, or a core program's upgrade control could not be read (including an unresolved program-derived address). This is the only definition. Token-issuer freeze keys on accepted tokens (for example USDC's) are rated on the token pages and in the collateral warning; they do not count against Stage 2, which covers the protocol's own admin keys, where a pause counts as funds frozen in place.",
              )}{" "}
              Stages follow Celebtir&apos;s versioned control requirements below. They do not guarantee protection from exploits or all team actions.
            </p>
            {stages.length ? (
              <div className={local.stageReqs}>
                {stages.map((st) => (
                  <Card key={st}>
                    <h3 className={local.cardTitle}>{STAGE_LABELS[st]} requires</h3>
                    <ul className={ref.prose}>
                      {reqs
                        .filter((r) => r.stage === st)
                        .map((r) => (
                          <li key={r.id}>{t(r.description)}</li>
                        ))}
                    </ul>
                  </Card>
                ))}
              </div>
            ) : (
              <p className={styles.muted}>The requirement list appears once a snapshot has been scored.</p>
            )}
          </section>

          <section id="evidence" className={local.section} aria-labelledby="evidence-h">
            <h2 id="evidence-h" className={styles.h2}>
              Evidence collection
            </h2>
            <p className={ref.prose}>
              {t(
                "Facts are read each day from Solana RPC (finalized slots) and HyperEVM RPC (a pinned block), and recorded with the account, field and slot or block they came from. Build verification comes from OtterSec and Sourcify, and TVL, fees and volume from DefiLlama.",
              )}
            </p>
            <h3 className={local.subhead} id="missing-data">
              How missing data is shown
            </h3>
            <dl className={styles.facts} data-testid="data-states">
              {(Object.keys(DATA_STATES) as DataStateKind[]).map((k) => (
                <div key={k} style={{ display: "contents" }}>
                  <dt>
                    <DataState kind={k} />
                  </dt>
                  <dd>{DATA_STATES[k].explain}</dd>
                </div>
              ))}
            </dl>
          </section>

          <section id="hyperliquid" className={local.section} aria-labelledby="hyperliquid-h">
            <h2 id="hyperliquid-h" className={styles.h2}>
              Hyperliquid
            </h2>
            <h3 className={local.subhead} id="hyperevm-sources">
              HyperEVM sources
            </h3>
            <ul className={ref.prose}>
              <li>
                {t(
                  "Upgrade control: the EIP-1967 admin slot of each proxy, followed through its ProxyAdmin owner to a key, a Safe multisig or a timelock. UUPS proxies are followed through the owner of the implementation.",
                )}
              </li>
              <li>
                {t(
                  "Upgrade delay: a timelock's minimum delay. A timelock whose admin role is held by an outside address counts as no delay, because that admin can change the delay at once.",
                )}
              </li>
              <li>{t("Admin powers: the owner, or the role holders the registry names, of each contract, confirmed on chain.")}</li>
              <li>{t("Code transparency: verified source on Sourcify, then on HyperEVMScan when a key is configured; for a proxy, the implementation is checked.")}</li>
              <li>{t("Dependencies: the contracts the registry declares. They are not traced from transactions yet, and each reason says so.")}</li>
            </ul>
            <h3 className={local.subhead} id="hip3-anchor">
              HIP-3 anchor block
            </h3>
            <p className={ref.prose}>
              {t(
                "The Hyperliquid info API returns no block height. Every HIP-3 fact is anchored to the HyperEVM block read just before the request, and the site shows that block, never a guessed time.",
              )}
            </p>
            <h3 className={local.subhead} id="hip3-unrated">
              Why HIP-3 DEXs are not rated yet
            </h3>
            <p className={ref.prose}>
              {t(
                "A HIP-3 market can never be immutable: its deployer must keep pushing prices, can halt and settle markets, and sets the caps. The six factors would rate every deployer the same way, so this version lists the facts (deployer, oracle updater, delegated actions, caps and stake) without ratings or a Stage. Rules for HIP-3 arrive with a later methodology version, after public feedback.",
              )}
            </p>
          </section>

          <section id="limitations" className={local.section} aria-labelledby="limitations-h">
            <h2 id="limitations-h" className={styles.h2}>
              Limitations
            </h2>
            <ul className={ref.prose}>
              <li>
                Value by control type is a simplification: free sources do not report TVL per program, so all of a protocol&apos;s value is placed
                under its weakest link.
              </li>
              <li>
                Recomputable, within limits. Anyone can re-score any published day&apos;s facts with the CLI and get the same ratings. Solana RPC
                cannot return an account&apos;s past state, so nobody can re-fetch yesterday&apos;s raw data: live checks can only confirm today&apos;s.
              </li>
              <li>Ratings describe on-chain configuration only. They do not audit code, and they may contain errors: report one from the About page.</li>
            </ul>
          </section>

          <section id="history" className={local.section} aria-labelledby="history-h">
            <h2 id="history-h" className={styles.h2}>
              Version history
            </h2>
            <h3 className={local.subhead} id="changelog">
              Changelog
            </h3>
            <p>v0.2 verifies declared program-derived authority addresses and rates them by the program&apos;s upgrade control. Lending and perpetuals without registered feeds are Unknown. Delayed value control requires at least the Medium upgrade-delay threshold. Historical v0.1 scores retain their original rules.</p>
            <Markdown blocks={parseMarkdown(methodologyChangelog())} seen={seen} />
            <details><summary className={local.subhead}>Full methodology source (TOML)</summary>
            {/* Focusable so the scrollable block can be read by keyboard. */}
            <pre className={ref.pre} tabIndex={0} aria-label={`methodology/v${version}.toml`}>
              {toml}
            </pre></details>
          </section>
        </div>
      </div>
    </>
  );
}
