import { ScopedLink } from "@/components/scope/ScopedLink";
import { LimitedItems } from "@/components/LimitedItems";

import { Finding } from "@/components/research/Finding";
import { FACTOR_QUESTIONS, protocolFinding, nextStageCounts } from "@/lib/presentation";
import type { ReactNode } from "react";
import { TimeChart } from "@/components/charts/TimeChart";
import { StageSwatch } from "@/components/ProtocolBits";
import { ReviewEvidence } from '@/components/research/ReviewEvidence';
import { AUDIT_LABELS } from '@/lib/scoreReview';
import { RatingBadge } from "@/components/RatingBadge";
import { controllerAddress, delayText, describeController, effectiveDelayText, loaderName, timeLockText } from "@/lib/controller";
import { formatUsd, formatChangeBp } from "@/lib/format";
import { weekOverWeek, CHANGE_LABEL } from "@/lib/activity";
import { FACTOR_LABELS, SERIES_COLOR, STAGE_LABELS, STAGE_NOTE } from "@/lib/labels";
import { POWER_LABELS } from "@/lib/permissions";
import type { ProtocolPage } from "@/lib/protocol";
import { CATEGORY_LABELS } from "@/lib/summary";
import { FACTORS, type Activity, type CodeTransparency, type Controller } from "@/lib/types";
import type { Tab } from "@/lib/viewState";
import { ControlGraph } from "@/components/graph/ControlGraph";
import { GraphList } from "@/components/graph/GraphList";
import { Address, Empty, EvidenceList, TxLink } from "@/components/Evidence";
import { Termed } from "@/components/Term";
import { CopyButton } from "@/components/CopyButton";
import { badgeLabel, embedSnippets } from "@/lib/badge";
import { HashFocus } from "./HashFocus";
import styles from "./Sections.module.css";

/** UI-12: the sections, in order, as on L2Beat's project pages. */
export const SECTIONS = [
  { id: "summary", title: "Summary" },
  { id: "risk-summary", title: "Risk summary" },
  { id: "risk-analysis", title: "Risk analysis" },
  { id: "stage", title: "Stage" },
  { id: "permissions", title: "Permissions" },
  { id: "control-graph", title: "Control graph" },
  { id: "programs", title: "Programs" },
  { id: "dependencies", title: "Dependencies and sources" },
  { id: "value", title: "Value" },
  { id: "activity", title: "Activity" },
  { id: "upgrades", title: "Upgrades & governance" },
  { id: "updates", title: "Updates" },
] as const;

type SectionId = (typeof SECTIONS)[number]["id"];

export const nodeId = (actorId: string) => `node-${actorId}`;

function Section({ id, children }: { id: SectionId; children: ReactNode }) {
  const title = SECTIONS.find((s) => s.id === id)!.title;
  return (
    <section id={id} aria-labelledby={`${id}-h`} className={styles.section}>
      <h2 id={`${id}-h`} className={styles.h2}>
        {title}
      </h2>
      {children}
    </section>
  );
}

/** A section or part of one with nothing to show says why (UI-12). */
function unavailable(what: string, reason: string) {
  return (
    <Empty>
      {what} unavailable: {reason}.
    </Empty>
  );
}

const KIND: Record<string, string> = {
  lending: "a lending protocol",
  dex: "a spot DEX",
  perps: "a perpetual futures exchange",
  "liquid-staking": "a liquid staking protocol",
};

/** API-4: the Stage badge and the snippets to embed it. */
function BadgeEmbed({ page, siteUrl }: { page: ProtocolPage; siteUrl: string | null }) {
  const snippets = embedSnippets(siteUrl ?? "", page.id, page.name, page.stage);
  return (
    <details className={styles.embed}>
      <summary>{page.stage ? "Embed the Stage badge" : "Embed the badge (unassessed)"}</summary>
      {/* eslint-disable-next-line @next/next/no-img-element -- a static SVG served as-is */}
      <img src={`/badges/${page.id}.svg`} alt={badgeLabel(page.name, page.stage)} height={20} />
      {(
        [
          ["HTML", snippets.html],
          ["Markdown", snippets.markdown],
        ] as const
      ).map(([label, code]) => (
        <div key={label} className={styles.snippet}>
          <span className={styles.muted}>{label}</span>
          <code>{code}</code>
          <CopyButton value={code} />
        </div>
      ))}
      {siteUrl ? null : <p className={styles.muted}>Links become absolute once the site address is set at deploy.</p>}
    </details>
  );
}

const stageRank = (s: NonNullable<ProtocolPage["stage"]>) => ({ unrated: -1, stage_0: 0, stage_1: 1, stage_2: 2 })[s];

/** A controller's facts in the panel's terms; values that do not apply are omitted. */
function controllerFacts(c: Controller): [string, ReactNode][] {
  const address = controllerAddress(c);
  const rows: [string, ReactNode][] = [["Type", describeController(c).split(" · ")[0]]];
  if (c.kind === "multisig") {
    rows.push(["Threshold", `${c.threshold} of ${c.voters} signers`]);
    rows.push(["Signers", c.signers ? `${c.signers.length} listed` : `${c.voters} voters`]);
    rows.push(["Time lock", timeLockText(c)]);
    rows.push(["Config authority", c.config_authority ? describeController(c.config_authority) : "None: changes go through the multisig"]);
  }
  if (c.kind === "timelock") rows.push(["Proposer", describeController(c.proposer)]);
  if (c.kind === "unrecognized") rows.push(["Owner program", c.owner_program ? <Address address={c.owner_program} /> : "Not read"]);
  if (c.kind === "pda") {
    rows.push(["Key", "None: an off-curve program-derived address has no private key"]);
    rows.push(["Account owner", c.owner_program ? <Address address={c.owner_program} /> : "No account at this address"]);
  }
  if (c.kind === "program_controlled") rows.push(["Controlling program", <Address key="p" address={c.program} />]);
  if (c.kind === "unavailable") rows.push(["Why", c.reason]);
  rows.push(["Effective delay", effectiveDelayText(c)]);
  if (address) rows.push(["Address", <Address key="a" address={address} />]);
  return rows;
}

/** Who can upgrade the core programs, one panel per distinct controller. */
function ControllerPanel({ page }: { page: ProtocolPage }) {
  const core = page.programs.filter((p) => p.role === "core");
  const groups = new Map<string, { controller: Controller; programs: string[] }>();
  for (const p of core) {
    const key = `${page.chain}|${p.controllerText}|${controllerAddress(p.controller.value) ?? (p.controller.value.kind === "immutable" ? "immutable" : p.address)}`;
    const g = groups.get(key) ?? { controller: p.controller.value, programs: [] };
    g.programs.push(p.name ?? p.address);
    groups.set(key, g);
  }
  if (!groups.size) return null;
  return (
    <div className={styles.controllers}>
      {[...groups.values()].map((g, i) => (
        <article key={i} className={styles.controllerPanel} aria-label={`Upgrade controller: ${describeController(g.controller)}`}>
          <h3 className={styles.panelTitle}>
            Who can upgrade <span className={styles.panelSub}>{g.programs.join(", ")}</span>
          </h3>
          <p className={styles.controllerName}>{describeController(g.controller)}</p>
          <dl className={styles.kv}>
            {controllerFacts(g.controller).map(([k, v]) => (
              <div key={k}>
                <dt>{k}</dt>
                <dd>{v}</dd>
              </div>
            ))}
          </dl>
        </article>
      ))}
    </div>
  );
}

function SummarySection({ page, siteUrl }: { page: ProtocolPage; siteUrl: string | null }) {
  return (
    <Section id="summary">
      {page.review.due ? (
        <p className={styles.notice} role="note">
          Registry review due:{" "}
          {page.review.lastReviewed ? `last reviewed ${page.review.lastReviewed}, more than 90 days ago.` : "this entry has never been reviewed."}
        </p>
      ) : null}
      <ul className={styles.badges} aria-label="Badges">
        <li>{page.chain === "solana" ? "Solana" : page.chain === "hyper_evm" ? "HyperEVM" : page.chain}</li>
        {page.assessment === "unassessed" ? <li>Unassessed</li> : null}
        <li>{page.category ? (CATEGORY_LABELS[page.category as Tab]?.label ?? page.category) : "Uncategorized"}</li>
        {page.squadsControlled ? <li>Squads-controlled</li> : null}
        {page.allCoreVerified ? <li>Verified build</li> : null}
      </ul>
      <p className={styles.about}>
        {page.name} is {(page.category && KIND[page.category]) ?? "a protocol"} on {page.chain === "solana" ? "Solana" : "HyperEVM"}. This page shows who can change it, how
        fast, and the on-chain account behind every {page.assessment === "assessed" ? "rating" : "fact"}.
        {page.unassessedReason ? ` ${page.unassessedReason}.` : ""}
      </p>
      {page.website ? (
        <p>
          <a href={page.website} rel="noopener">
            {page.website.replace(/^https?:\/\//, "").replace(/\/$/, "")}
          </a>
        </p>
      ) : (
        <Empty>No website is listed in the registry.</Empty>
      )}
      <div className={styles.assessmentGroups}>
        <div><h3 className={styles.h3}>Verified protections</h3>
          {page.programs.some(p => p.transparency.value.kind === "verified" && p.transparency.evidence.length) ? <p>Verified builds are recorded for {page.programs.filter(p => p.transparency.value.kind === "verified" && p.transparency.evidence.length).map(p => p.name ?? p.address).join(", ")}. <a href="#programs">Inspect code verification</a>.</p> : <p>No verified build protection is established by the available program evidence.</p>}
          {page.programs.filter(p => p.controller.value.kind === "immutable" && p.controller.evidence.length).map(p => <p key={p.address}>{p.name ?? p.address}: immutable upgrade control. <a href="#permissions">Inspect powers</a>.</p>)}
        </div>
        <div><h3 className={styles.h3}>Powers and trust assumptions</h3><p>Recorded upgrade and administrative powers remain material trust assumptions. <a href="#risk-summary">Read the published findings</a> and <a href="#permissions">who holds each power</a>.</p></div>
        <div><h3 className={styles.h3}>Unresolved observations</h3>{page.factors && Object.entries(page.factors).some(([,f]) => f.rating === "unknown") ? <ul>{Object.entries(page.factors).filter(([,f]) => f.rating === "unknown").map(([f]) => <li key={f}><a href={`#factor-${f}`}>{FACTOR_LABELS[f as keyof typeof FACTOR_LABELS]}</a>: evidence is insufficient.</li>)}</ul> : <p>{page.assessment === "unassessed" ? page.unassessedReason : "No factor is published as Unknown. This does not establish complete coverage of every authority path."}</p>}</div>
      </div>
      <details><summary>Recorded upgrade controllers ({page.programs.filter(p => p.role === "core").length} core programs)</summary><ControllerPanel page={page} /></details>
      <BadgeEmbed page={page} siteUrl={siteUrl} />
    </Section>
  );
}

function ValueSection({ page }: { page: ProtocolPage }) {
  const groups = page.assetGroups;
  const total = groups.kind === "usd" ? groups.stablecoins + groups.sol + groups.btc + groups.other : 0;
  return (
    <Section id="value">
      {page.tvl.kind === "usd" ? <dl className={styles.stats}><div><dt>Reported TVL</dt><dd>{formatUsd(page.tvl.value)}</dd></div></dl> : null}
      <p className={styles.muted}>Economic context only; value does not change a control rating.</p>
      {page.tvl.kind === "unavailable" ? unavailable("TVL", page.tvl.reason) : null}
      {page.tvlDays.length >= 2 ? (
        <TimeChart
          title="TVL"
          description="Total value locked, from DefiLlama, one point per daily snapshot."
          series={[{ key: "tvl", label: "TVL", color: SERIES_COLOR }]}
          points={page.tvlDays.map((d) => ({ date: d.date, values: { tvl: d.usd } }))}
          unit="usd"
        />
      ) : (
        <Empty>The TVL chart appears once two daily snapshots exist.</Empty>
      )}
      <h3 className={styles.h3}>Asset groups</h3>
      {groups.kind === "usd" && total > 0 ? (
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Group</th>
              <th scope="col">Value</th>
              <th scope="col">Share</th>
            </tr>
          </thead>
          <tbody>
            {(
              [
                ["Stablecoins", groups.stablecoins],
                ["SOL and staked SOL", groups.sol],
                ["BTC", groups.btc],
                ["Other", groups.other],
              ] as const
            ).map(([label, usd]) => (
              <tr key={label}>
                <th scope="row">{label}</th>
                <td>{formatUsd(usd)}</td>
                <td>{((usd / total) * 100).toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : groups.kind === "unavailable" ? (
        unavailable("Asset groups", groups.reason)
      ) : (
        <Empty>DefiLlama reports no assets for this protocol.</Empty>
      )}
      <h3 className={styles.h3}>Tokens used</h3>
      {page.tokens.length ? (
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Token</th>
              <th scope="col">Used as</th>
              <th scope="col">Worst rating</th>
            </tr>
          </thead>
          <LimitedItems table label="tokens">
            {page.tokens.map((t) => (
              <tr key={t.mint}>
                <th scope="row">
                  <ScopedLink href={`/tokens/${t.mint}/`}>{t.symbol}</ScopedLink>
                </th>
                <td>{t.roles.join(", ")}</td>
                <td>
                  <RatingBadge rating={t.worst} />
                  {t.warning ? <span className={styles.note}>{t.warning}</span> : null}
                </td>
              </tr>
            ))}
          </LimitedItems>
        </table>
      ) : (
        <Empty>No tokens are listed for this protocol in the registry.</Empty>
      )}
    </Section>
  );
}

function ActivityChart({ title, activity }: { title: string; activity: Activity }) {
  if (activity.kind === "unavailable") return unavailable(title, activity.reason);
  if (activity.days.length < 2) return <Empty>{title}: fewer than two days of data so far.</Empty>;
  return (
    <TimeChart
      title={title}
      description={`Daily ${title.toLowerCase()} in US dollars, from DefiLlama.`}
      series={[{ key: "usd", label: title, color: SERIES_COLOR }]}
      points={activity.days.map((d) => ({ date: d.date, values: { usd: d.usd } }))}
      unit="usd"
    />
  );
}

function ActivitySection({ page }: { page: ProtocolPage }) {
  return (
    <Section id="activity">
      <dl className={styles.stats}>{([["Volume",page.volume],["Fees",page.fees]] as const).map(([label,a]) => a.kind === "usd" ? <div key={label}><dt>Latest day {label.toLowerCase()} ({a.days.at(-1)?.date ?? "date unavailable"})</dt><dd>{formatUsd(a.latest)}</dd><dd className={styles.note}>{CHANGE_LABEL}: {weekOverWeek(a) === null ? "unavailable (a missing or near-zero day)" : formatChangeBp(weekOverWeek(a)!)}</dd></div> : null)}</dl>
      <div className={styles.charts}>
        <ActivityChart title="Volume" activity={page.volume} />
        <ActivityChart title="Fees" activity={page.fees} />
      </div>
    </Section>
  );
}

function RiskSummarySection({ page, seen }: { page: ProtocolPage; seen: Set<string> }) {
  if (!page.factors) return <Section id="risk-summary"><p>Unassessed: this methodology version does not rate this chain, so no risk sentences are published. <ScopedLink href="/hyperevm/">Read recorded HyperEVM authorities</ScopedLink>.</p></Section>;
  return (
    <Section id="risk-summary">
      {page.risks.length ? (
        <ul className={styles.risks}>
          {page.risks.map((r) => (
            <li key={r.text} data-severity={r.severity}>
              <RatingBadge rating={r.severity === "high" ? "high" : "medium"} />
              <span>
                <Termed text={r.text} seen={seen} />
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <Empty>No risk sentences apply: no factor is rated High or Medium.</Empty>
      )}
    </Section>
  );
}

function RiskAnalysisSection({ page, seen }: { page: ProtocolPage; seen: Set<string> }) {
  const factors = page.factors;
  if (!factors) return <Section id="risk-analysis"><p>Unassessed: no factor ratings are published for this chain. <ScopedLink href="/hyperevm/">Inspect native contract facts and evidence coverage</ScopedLink>.</p></Section>;
  return (
    <Section id="risk-analysis">
      <div className={styles.cards}>
        {FACTORS.map((f) => {
          const score = factors[f];
          return (
            <article key={f} className={styles.card} data-rating={score.rating} aria-labelledby={`factor-${f}`}>
              <div className={styles.cardHead}>
                <h3 id={`factor-${f}`} className={styles.h3} tabIndex={-1}>
                  {FACTOR_LABELS[f]}
                </h3>
                <ScopedLink href={`/risk/?detail=${page.id}.${f}`}><RatingBadge rating={score.rating} tone="pill" /></ScopedLink>
              </div>
              <p className={styles.label}><Termed text={FACTOR_QUESTIONS[f]} seen={seen} /></p>
              <Finding text={score.reason} summary={protocolFinding(page, f)} seen={seen} />
              <details><summary>Rating criteria</summary><p className={styles.muted}>
                <span className={styles.label}>To improve:</span> <Termed text={score.next_step} seen={seen} />
              </p></details>
              {f === "oracle_dependency" ? <><RegisteredFeeds page={page} /><AggregatorFeeds page={page} /></> : null}
              {score.registry_sources?.length ? (
                <p className={styles.muted}>
                  <span className={styles.label}>Registry review:</span> {score.registry_sources.map(u => <a key={u} href={u}>{u.replace(/^https:\/\//, "")}</a>)}
                </p>
              ) : null}
              <details className={styles.technical}>
                <summary>
                  Technical evidence <span className={styles.muted}>({score.evidence.length} account{score.evidence.length === 1 ? "" : "s"})</span>
                </summary>
                <EvidenceList evidence={score.evidence} />
              </details>
            </article>
          );
        })}
      </div>
    </Section>
  );
}

/** F4: every price feed the registry lists, with its role and who publishes it. */
function RegisteredFeeds({ page }: { page: ProtocolPage }) {
  if (!page.oracles.length) return null;
  const markets = [...new Set(page.oracles.map(o => o.market))];
  return (
    <details>
      <summary>
        Registered price feeds <span className={styles.muted}>({page.oracles.length} across {markets.length} market{markets.length === 1 ? "" : "s"})</span>
      </summary>
      <LimitedItems label="Registered price feeds">
        {page.oracles.map(o => (
          <li key={o.account}>
            {o.market}: {o.role}
            {o.provider ? ` · ${o.provider}` : ""} · {sourceText(o.source)}
            {o.boundsPrimary ? " · bounds the primary price" : ""}
            {o.guardedBy.length ? ` · documented guards: ${o.guardedBy.join(", ")}` : ""} <Address address={o.account} />
          </li>
        ))}
      </LimitedItems>
    </details>
  );
}

function sourceText(s: ProtocolPage["oracles"][number]["source"]): string {
  switch (s.kind) {
    case "network":
      return `${s.name} network`;
    case "protocol_run":
      return "protocol-run";
    case "other":
      return `not an established network (owner ${s.owner_program.slice(0, 4)}…${s.owner_program.slice(-4)})`;
    case "unavailable":
      return `unreadable: ${s.reason}`;
  }
}

/** Edge case 11: the feeds a protocol-run aggregator combines, as it names them. */
function AggregatorFeeds({ page }: { page: ProtocolPage }) {
  const open = page.aggregatorFeeds.length <= 3;
  return (
    <>
      <details><summary>Underlying feed observations ({page.aggregatorFeeds.length})</summary>
      <LimitedItems label="underlying feed observations">
      {page.aggregatorFeeds.map(({ oracle, market, underlying }) => {
        const label = `Feeds behind the ${market} price`;
        const v = underlying.value;
        return (
          <li key={`${oracle}-${market}`}><details open={open}>
            <summary className={styles.label}>{label}</summary>
            {v.kind === "listed" ? (
              <LimitedItems label={label}>
                {v.feeds.map((feed) => (
                  <li key={`${feed.source_type}-${feed.address}`}>
                    {feed.source_type} <Address address={feed.address} />
                  </li>
                ))}
              </LimitedItems>
            ) : (
              <p className={styles.muted}>Not listed: {v.reason}</p>
            )}
            <EvidenceList evidence={underlying.evidence} />
          </details></li>
        );
      })}
      </LimitedItems></details>
    </>
  );
}

function StageSection({ page, seen }: { page: ProtocolPage; seen: Set<string> }) {
  const stage = page.stage;
  if (!stage) return <Section id="stage"><p>Unassessed: no Stage is published for this chain under the current methodology. <ScopedLink href="/hyperevm/">Inspect native contract evidence</ScopedLink>.</p></Section>;
  const stages = [...new Set(page.requirements.map((r) => r.stage))];
  const counts = nextStageCounts(page.inspection);
  return (
    <Section id="stage">
      <p className={styles.stageLine}>
        <StageSwatch stage={stage} /> <strong>{STAGE_LABELS[stage]}</strong>
        {page.nextStage ? (
          <span className={styles.muted}>
            {" "}
            · {counts ? `${counts.met} met · ${counts.unmet} verified unmet · ${counts.unresolved} unresolved for ${STAGE_LABELS[page.nextStage]}` : `${page.missing.length} published blockers for ${STAGE_LABELS[page.nextStage]}`}
          </span>
        ) : null}
      </p>
      <p className={styles.muted}>
        <Termed text={STAGE_NOTE} seen={seen} />
      </p>
      <ol className={styles.ladder} aria-label="Stages">
        {(["stage_0", "stage_1", "stage_2"] as const).map((st) => (
          <li key={st} data-current={stage === st || undefined} data-reached={stageRank(stage) >= stageRank(st) || undefined}>
            <span className={styles.ladderMark} aria-hidden="true" />
            {STAGE_LABELS[st]}
            {stage === st ? <span className={styles.srOnly}> (current)</span> : null}
          </li>
        ))}
      </ol>
      {page.requirements.length ? (
        stages.map((st) => (
          <details key={st} className={styles.stageGroup}>
            <summary className={styles.h3}>
              {STAGE_LABELS[st]}{" "}
              <span className={styles.stageCount}>
                {page.inspection ? `${page.inspection.criteria.filter(c => c.requirement.stage === st && c.status === "MET").length} met · ${page.inspection.criteria.filter(c => c.requirement.stage === st && c.status === "UNMET_VERIFIED").length} verified unmet · ${page.inspection.criteria.filter(c => c.requirement.stage === st && c.status === "UNRESOLVED_EVIDENCE").length} unresolved` : `${page.requirements.filter(r => r.stage === st && r.met).length} of ${page.requirements.filter(r => r.stage === st).length} met in the published evaluator`}
              </span>
            </summary>
            <span className={styles.progress} aria-hidden="true">
              {page.requirements
                .filter((r) => r.stage === st)
                .map((r) => (
                  <span key={r.id} data-met={r.met} />
                ))}
            </span>
            <ul className={styles.checklist}>
              {page.requirements
                .filter((r) => r.stage === st)
                .map((r) => (
                  <li key={r.id} data-met={page.inspection?.criteria.find(c => c.requirement.id === r.id)?.status === "MET"}>
                    <span className={styles.mark} aria-hidden="true">
                      {page.inspection?.criteria.find(c => c.requirement.id === r.id)?.status === "UNRESOLVED_EVIDENCE" ? "?" : r.met ? "✓" : "✗"}
                    </span>
                    <span className={styles.srOnly}>{page.inspection?.criteria.find(c => c.requirement.id === r.id)?.status === "UNRESOLVED_EVIDENCE" ? "Unresolved:" : r.met ? "Met:" : "Missing:"}</span> <Termed text={r.description} seen={seen} />
                    <details><summary>{page.inspection?.criteria.find(c=>c.requirement.id===r.id) ? AUDIT_LABELS[page.inspection.criteria.find(c=>c.requirement.id===r.id)!.status] : r.met ? "Satisfied criterion and evidence" : "Requirement unresolved — evidence incomplete"}</summary><p><Termed text={r.detail} seen={seen} /></p><ReviewEvidence evidence={page.inspection?.criteria.find(c=>c.requirement.id===r.id)?.evidence??[]}/><ScopedLink href="/methodology/#stages">Published Stage criteria</ScopedLink><p><a href="#risk-analysis">Inspect the factor observations and evidence</a></p></details>
                  </li>
                ))}
            </ul>
          </details>
        ))
      ) : (
        <Empty>
          No Stage requirements were evaluated: the protocol is {STAGE_LABELS[stage]} because its core programs could not be
          assessed.
        </Empty>
      )}
    </Section>
  );
}

/** The interactive graph, with its list view below for keyboard and screen readers (UI-19 to UI-23). */
function ControlGraphSection({ page, seen }: { page: ProtocolPage; seen: Set<string> }) {
  return (
    <Section id="control-graph">
      <HashFocus prefix="node-" />
      {page.actors.length ? (
        <>
          <p className={styles.muted}>
            <Termed text="Click a box for its facts and evidence. Arrows point from who holds a power to what it controls." seen={seen} />{" "}
            <ScopedLink href="/controllers/">Controllers shared across protocols</ScopedLink>
          </p>
          <ControlGraph graph={page.graph} />
          <h3 className={styles.h3}>List view</h3>
          <GraphList graph={page.graph} />
        </>
      ) : (
        <Empty>No keys found: no program, admin account or token in the registry has a controller that can act.</Empty>
      )}
    </Section>
  );
}

function PermissionsSection({ page }: { page: ProtocolPage }) {
  return (
    <Section id="permissions">
      {page.actors.length ? (
        <LimitedItems label="permission holders" className={styles.actorList}>{page.actors.map((a) => (
          <li key={a.id}><article key={a.id} className={styles.actor} aria-labelledby={`actor-${a.id}`}>
            <h3 tabIndex={-1} id={`actor-${a.id}`} className={styles.h3}>
              {a.label} {a.address ? <Address address={a.address} /> : null}
            </h3>
            <a href={`#${nodeId(a.id)}`} className={styles.graphLink}>
              Show in control graph
            </a>
            <p>{[...new Set(a.grants.map(g => POWER_LABELS[g.power]))].join(" · ")}</p>
            <details><summary>Permissions and evidence ({a.grants.length})</summary>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th scope="col">Power</th>
                  <th scope="col">Over</th>
                  <th scope="col">Delay</th>
                  <th scope="col">Evidence</th>
                </tr>
              </thead>
              <LimitedItems table colSpan={4} label="permissions">
                {a.grants.map((g, i) => (
                  <tr key={`${g.power}.${g.subject}.${i}`}>
                    <th scope="row">{POWER_LABELS[g.power]}</th>
                    <td>{g.subjectLabel}</td>
                    <td>{delayText(g.delaySecs)}</td>
                    <td>
                      <EvidenceList evidence={g.evidence} />
                    </td>
                  </tr>
                ))}
              </LimitedItems>
            </table></details>
          </article></li>
        ))}</LimitedItems>
      ) : (
        <Empty>No admin keys or upgrade authorities found.</Empty>
      )}
      {page.renounced.length ? (
        <>
          <h3 className={styles.h3}>Held by no one</h3>
          <LimitedItems label="renounced powers">
            {page.renounced.map((g, i) => (
              <li key={`${g.power}.${g.subject}.${i}`}>
                {POWER_LABELS[g.power]}: {g.subjectLabel} <span className={styles.muted}>(renounced or immutable)</span>
              </li>
            ))}
          </LimitedItems>
        </>
      ) : null}
    </Section>
  );
}

function transparencyText(t: CodeTransparency): ReactNode {
  switch (t.kind) {
    case "verified":
      return (
        <>
          Verified build (<a href={t.repo}>{t.commit.slice(0, 7)}</a>){t.warning ? <span className={styles.note}>{t.warning}</span> : null}
        </>
      );
    case "public_source_unverified":
      return (
        <>
          Public source, not verified (<a href={t.repo}>source</a>)
        </>
      );
    case "no_public_source":
      return "No public source";
    case "unavailable":
      return <span className={styles.muted}>Unavailable: {t.reason}</span>;
  }
}

function ProgramsSection({ page }: { page: ProtocolPage }) {
  return (
    <Section id="programs">
      {page.programs.length ? (
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Program</th>
              <th scope="col">Role</th>
              <th scope="col">Loader</th>
              <th scope="col">Upgrade controller</th>
              <th scope="col">Verification</th>
              <th scope="col">Last upgrade</th>
            </tr>
          </thead>
          <tbody>
            {page.programs.map((p) => {
              const last = p.history?.lastChange;
              return (
                <tr key={p.address}>
                  <th scope="row">
                    {p.name ?? "Unnamed"} <Address address={p.address} />
                  </th>
                  <td>{p.role === "core" ? "Core" : "Periphery"}</td>
                  <td>{loaderName(p.loader) ?? <span className={styles.muted}>Not read</span>}</td>
                  <td>
                    {p.controllerText}
                    {p.controller.value.kind === "unavailable" ? <span className={styles.note}>{p.controller.value.reason}</span> : null}
                  </td>
                  <td>{transparencyText(p.transparency.value)}</td>
                  <td>
                    {!p.history ? (
                      <span className={styles.muted}>Not read</span>
                    ) : last ? (
                      (last.date ?? `slot ${last.slot.toLocaleString("en-US")}`)
                    ) : (
                      <span className={styles.muted}>{p.history.complete ? "None" : "None found yet"}</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      ) : (
        <Empty>No programs are listed for this protocol in the registry.</Empty>
      )}
    </Section>
  );
}

const UPGRADE_LABELS = {
  deploy: "Deployed",
  upgrade: "Upgraded",
  set_authority: "Authority changed",
  remove_authority: "Made immutable",
  extend: "Program data extended",
  close: "Closed",
} as const;

function UpgradesSection({ page }: { page: ProtocolPage }) {
  const u = page.upgrades;
  return (
    <Section id="upgrades">
      {u ? (
        <dl className={styles.stats}>
          <div>
            <dt>Upgrades</dt>
            <dd>{u.upgrades}</dd>
          </div>
          <div>
            <dt>Last upgrade</dt>
            <dd>{u.last_upgrade_date ?? (u.last_upgrade_slot !== null ? `slot ${u.last_upgrade_slot}` : "None")}</dd>
          </div>
          <div>
            <dt>Average interval</dt>
            <dd>{u.average_interval_days !== null ? `${u.average_interval_days} days` : "Not enough upgrades"}</dd>
          </div>
          <div>
            <dt>Last 30 days</dt>
            <dd>{u.upgrades_last_30_days}</dd>
          </div>
        </dl>
      ) : (
        <Empty>No upgrade history has been built for this protocol yet.</Empty>
      )}
      {u && !u.complete ? <p className={styles.muted}>Counts are lower bounds from recorded transaction evidence; history coverage is incomplete.</p> : null}
      {page.historyScanned && page.historyScanned.scanned < page.historyScanned.total ? (
        <p className={styles.muted}>
          History is still filling: {page.historyScanned.scanned.toLocaleString("en-US")} of{" "}
          {page.historyScanned.total.toLocaleString("en-US")} transactions examined so far.
        </p>
      ) : null}
      {page.upgradeEvents.length ? (
        <LimitedItems ordered label="upgrade records" className={styles.timeline}>
          {page.upgradeEvents.map((e) => (
            <li key={`${e.program}:${e.signature}:${e.kind}`}>
              <span className={styles.date}>{e.date ?? `slot ${e.slot.toLocaleString("en-US")}`}</span> {UPGRADE_LABELS[e.kind]}{" "}
              {e.programName ?? <Address address={e.program} />}{" "}
              <TxLink signature={e.signature} />
            </li>
          ))}
        </LimitedItems>
      ) : (
        <Empty>No upgrades were found in the transactions examined so far.</Empty>
      )}
    </Section>
  );
}

const SEVERITY_LABELS = { critical: "Critical", warning: "Warning", positive: "Positive", info: "Info" } as const;

function UpdatesSection({ page }: { page: ProtocolPage }) {
  return (
    <Section id="updates">
      {page.events.length ? (
        <LimitedItems ordered label="change records" className={styles.timeline}>
          {page.events.map((e) => (
            <li key={e.id}>
              <span className={styles.date}>{e.date}</span>{" "}
              <span className={styles.severity} data-severity={e.severity}>
                {SEVERITY_LABELS[e.severity]}
              </span>{" "}
              {e.summary}
            </li>
          ))}
        </LimitedItems>
      ) : (
        <Empty>No changes have been recorded for this protocol since tracking began.</Empty>
      )}
    </Section>
  );
}

function DependenciesSection({ page }: { page: ProtocolPage }) {
  const deps = page.dependencies.value;
  return (
    <Section id="dependencies">
      <h3 className={styles.h3}>Called programs</h3>
      {deps.kind === "unavailable" ? (
        unavailable("Called programs", deps.reason)
      ) : Object.keys(deps.callees).length === 0 ? (
        <Empty>The sampled transactions called no other programs with user funds.</Empty>
      ) : (
        <>
          {deps.stale ? <p className={styles.muted}>No transactions in the latest window: this list is carried over from an earlier run.</p> : null}
          <ul className={styles.plain}>
            {Object.entries(deps.callees).map(([address, d]) => (
              <li key={address}>
                {page.sources.find(s => s.kind === "program" && s.address === address)?.label}{" "}
                <Address address={address} /> {describeController(d.upgrade_control.value)}{" "}
                {page.calleeRatings[address] ? (
                  <RatingBadge rating={page.calleeRatings[address].rating} />
                ) : (
                  <span className={styles.muted}>not rated</span>
                )}
              </li>
            ))}
          </ul>
        </>
      )}
      <h3 className={styles.h3}>Registry sources</h3>
      {page.sources.length ? (
        <LimitedItems className={styles.plain} label="registry sources">
          {page.sources.map((s) => (
            <li key={`${s.kind}.${s.address}.${s.label}`}>
              <span className={styles.muted}>{s.kind}</span> {s.label} <Address address={s.address} />{" "}
              <a href={s.url}>source</a>
            </li>
          ))}
        </LimitedItems>
      ) : (
        <Empty>The registry lists no sources for this protocol.</Empty>
      )}
      <p>
        {page.review.lastReviewed
          ? `Registry entry dated ${page.review.lastReviewed}${page.review.reviewedBy ? `, reviewed by ${page.review.reviewedBy}` : ""}.`
          : "The registry entry has no review date."}{" "}
        {page.review.awaitingHumanReview ? "Awaiting human review under the current methodology. " : ""}
        <ScopedLink href="/about/#corrections">Report an error</ScopedLink> · <a href={`/protocols/${page.id}.md`}>Markdown version</a>
      </p>
    </Section>
  );
}

/** All twelve sections with their sticky side navigation (UI-12). */
export function ProtocolSections({ page, siteUrl = null }: { page: ProtocolPage; siteUrl?: string | null }) {
  // UI-6: glossary terms link at their first use on the page.
  const seen = new Set<string>();
  return (
    <div className={styles.layout}>
      <nav aria-label="Sections" className={styles.nav}>
        <ol>
          {SECTIONS.map((s) => (
            <li key={s.id}>
              <a href={`#${s.id}`}>{s.title}</a>
            </li>
          ))}
        </ol>
      </nav>
      <div className={styles.body}>
        <SummarySection page={page} siteUrl={siteUrl} />
        <RiskSummarySection page={page} seen={seen} />
        <RiskAnalysisSection page={page} seen={seen} />
        <StageSection page={page} seen={seen} />
        <PermissionsSection page={page} />
        <ControlGraphSection page={page} seen={seen} />
        <ProgramsSection page={page} />
        <DependenciesSection page={page} />
        <ValueSection page={page} />
        <ActivitySection page={page} />
        <UpgradesSection page={page} />
        <UpdatesSection page={page} />
      </div>
    </div>
  );
}
