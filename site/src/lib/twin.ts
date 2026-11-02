// UI-14: the Markdown twin of a protocol page, for readers and tools that
// want text. Same data and order as the page; deterministic.

import { CHANGE_LABEL, weekOverWeek } from "./activity";
import { changeDate } from "./changes";
import { delayText, describeController, loaderName } from "./controller";
import { formatChangeBp, formatUsd } from "./format";
import { FACTOR_LABELS, STAGE_LABELS, STAGE_NOTE } from "./labels";
import { POWER_LABELS } from "./permissions";
import type { ProtocolPage } from "./protocol";
import { NOTICE } from "./site";
import { accountUrl, txUrl } from "./solscan";
import { ratingStyle } from "./tokens";
import { FACTORS, type Activity, type CodeTransparency, type Evidence } from "./types";

const ev = (e: Evidence) => `[${e.address}](${e.explorer_url}) \`${e.field}\` at slot ${e.slot}`;
// F8: the latest complete day, dated, and the one 7-day rule.
const activity = (a: Activity) => {
  if (a.kind !== "usd") return `unavailable: ${a.reason}`;
  const change = weekOverWeek(a);
  return `${formatUsd(a.latest)} on ${a.days.at(-1)?.date ?? "the latest day"}; ${CHANGE_LABEL}: ${change !== null ? formatChangeBp(change) : "unavailable (a missing or near-zero day)"}`;
};
const transparency = (t: CodeTransparency) =>
  t.kind === "verified"
    ? `verified build of ${t.repo}${t.commit ? ` at ${t.commit}` : ""}`
    : t.kind === "public_source_unverified"
      ? `public source at ${t.repo}${t.commit ? ` (commit ${t.commit})` : ""}, not verified`
      : t.kind === "no_public_source"
        ? "no public source"
        : `unavailable: ${t.reason}`;

export function protocolMarkdown(p: ProtocolPage, meta: { snapshotDate: string; methodology: string }): string {
  const out: string[] = [];
  const h = (title: string) => out.push("", `## ${title}`, "");
  const empty = (text: string) => out.push(text);

  out.push(`# ${p.name}`, "", `> ${NOTICE}`, "", `Snapshot ${meta.snapshotDate}, methodology v${meta.methodology}. Source: Celebtir, CC BY 4.0.`);

  h("Summary");
  out.push(
    `- Chain: ${p.chain === "solana" ? "Solana" : p.chain === "hyper_evm" ? "HyperEVM" : p.chain}`,
    `- Category: ${p.category ?? "uncategorized"}`,
    // F5: an unassessed protocol publishes no Stage.
    `- ${p.stage ? `Stage: ${STAGE_LABELS[p.stage]}` : `Assessment: unassessed. ${p.unassessedReason ?? ""}`.trim()}`,
    `- TVL: ${p.tvl.kind === "usd" ? formatUsd(p.tvl.value) : `unavailable: ${p.tvl.reason}`}`,
    `- Website: ${p.website ?? "none listed in the registry"}`,
  );
  if (p.squadsControlled) out.push("- Core programs controlled by a Squads multisig");
  if (p.allCoreVerified) out.push("- Every core program has a verified build");
  if (p.review.due) out.push(`- Registry review due (last reviewed ${p.review.lastReviewed ?? "never"})`);

  h("Risk summary");
  if (!p.factors) empty("Unassessed: this methodology version publishes no ratings or risk sentences for this chain.");
  else if (p.risks.length) for (const r of p.risks) out.push(`- ${r.severity === "high" ? "High" : "Medium"}: ${r.text}`);
  else empty("No risk sentences apply: no factor is rated High or Medium.");

  h("Risk analysis");
  if (!p.factors) empty("Unassessed: no factor ratings are published for this chain.");
  for (const f of p.factors ? FACTORS : []) {
    const sc = p.factors![f];
    out.push(`- ${FACTOR_LABELS[f]}: **${ratingStyle[sc.rating].label}**. ${sc.reason}. To improve: ${sc.next_step}.`);
    for (const e of sc.evidence) out.push(`  - Evidence: ${ev(e)}`);
    for (const u of sc.registry_sources ?? []) out.push(`  - Registry review source: ${u}`);
  }
  if (p.oracles.length) {
    out.push("", "Registered price feeds:");
    for (const o of p.oracles) out.push(`- ${o.market}: ${o.role}${o.provider ? `, ${o.provider}` : ""}${o.boundsPrimary ? ", bounds the primary price" : ""} [${o.account}](${accountUrl(o.account)})`);
  }

  h("Stage");
  if (!p.stage) {
    empty("Unassessed: no Stage is published for this chain under the current methodology.");
  } else {
    out.push(`${STAGE_LABELS[p.stage]}. ${STAGE_NOTE}`, "");
    if (p.requirements.length) {
      for (const r of p.requirements) out.push(`- [${r.met ? "x" : " "}] ${STAGE_LABELS[r.stage]}: ${r.description} (${r.detail})`);
    } else {
      empty("No Stage requirements were evaluated: the core programs could not be assessed.");
    }
  }

  h("Permissions");
  if (p.actors.length) {
    for (const a of p.actors) {
      out.push(`- ${a.label}${a.address ? ` [${a.address}](${accountUrl(a.address)})` : ""}`);
      for (const g of a.grants) out.push(`  - ${POWER_LABELS[g.power]}: ${g.subjectLabel}, ${delayText(g.delaySecs)}`);
    }
  } else {
    empty("No admin keys or upgrade authorities found.");
  }
  for (const g of p.renounced) out.push(`- Held by no one: ${POWER_LABELS[g.power]} (${g.subjectLabel})`);

  h("Control graph");
  if (p.graph.edges.length && p.actors.length) {
    const title = new Map(p.graph.nodes.map((n) => [n.id, n.title]));
    for (const e of p.graph.edges) out.push(`- ${title.get(e.source)} → ${title.get(e.target)}: ${e.label}`);
  } else {
    empty("No keys found: no program, admin account or token in the registry has a controller that can act.");
  }

  h("Programs");
  if (p.programs.length) {
    for (const pr of p.programs) {
      const last = pr.history?.lastChange;
      out.push(
        `- ${pr.name ?? "Unnamed"} [${pr.address}](${accountUrl(pr.address)}): ${pr.role}, ${loaderName(pr.loader) ?? "loader not read"}, ` +
          `controller ${describeController(pr.controller.value)}, ${transparency(pr.transparency.value)}, ` +
          `last upgrade ${!pr.history ? "not read" : last ? (last.date ?? `slot ${last.slot}`) : "none found"}`,
      );
    }
  } else {
    empty("No programs are listed for this protocol in the registry.");
  }

  h("Dependencies and sources");
  const d = p.dependencies.value;
  if (d.kind === "unavailable") empty(`Called programs unavailable: ${d.reason}.`);
  else if (!Object.keys(d.callees).length) empty("The sampled transactions called no other programs with user funds.");
  else
    for (const [addr, c] of Object.entries(d.callees)) {
      const r = p.calleeRatings[addr];
      out.push(`- Calls [${addr}](${accountUrl(addr)}): ${describeController(c.upgrade_control.value)}${r ? `, rated ${ratingStyle[r.rating].label}` : ""}`);
    }
  for (const src of p.sources) out.push(`- Registry source for ${src.kind} ${src.label} (${src.address}): ${src.url}`);
  out.push(`- Registry entry dated: ${p.review.lastReviewed ?? "never"}${p.review.reviewedBy ? `, reviewed by ${p.review.reviewedBy}` : ""}${p.review.awaitingHumanReview ? "; awaiting human review" : ""}`);
  h("Value");
  if (p.assetGroups.kind === "usd") {
    const g = p.assetGroups;
    out.push(`- Stablecoins ${formatUsd(g.stablecoins)}, SOL ${formatUsd(g.sol)}, BTC ${formatUsd(g.btc)}, other ${formatUsd(g.other)}`);
  } else {
    empty(`Asset groups unavailable: ${p.assetGroups.reason}.`);
  }
  if (p.tokens.length) {
    for (const t of p.tokens) out.push(`- ${t.symbol} (${t.roles.join(", ")}): worst rating ${ratingStyle[t.worst].label}${t.warning ? `. ${t.warning}` : ""}`);
  } else {
    empty("No tokens are listed for this protocol in the registry.");
  }

  h("Activity");
  out.push(`- Volume: ${activity(p.volume)}`, `- Fees: ${activity(p.fees)}`);

  h("Upgrades & governance");
  if (p.upgrades) {
    const u = p.upgrades;
    out.push(
      `- Upgrades: ${u.upgrades}${u.complete ? "" : " so far (history still filling)"}`,
      `- Last upgrade: ${u.last_upgrade_date ?? (u.last_upgrade_slot !== null ? `slot ${u.last_upgrade_slot}` : "none")}`,
      `- Average interval: ${u.average_interval_days !== null ? `${u.average_interval_days} days` : "not enough upgrades"}`,
    );
  } else {
    empty("No upgrade history has been built for this protocol yet.");
  }
  for (const e of p.upgradeEvents) out.push(`- ${e.date ?? `slot ${e.slot}`}: ${e.kind.replace("_", " ")} ${e.programName ?? e.program} ([transaction](${txUrl(e.signature)}))`);

  h("Updates");
  if (p.events.length) for (const e of p.events) out.push(`- ${changeDate(e)}${e.occurred_on && e.occurred_on !== e.date ? ` (recorded ${e.date})` : ""} (${e.severity}): ${e.summary}`);
  else empty("No changes have been recorded for this protocol since tracking began.");

  out.push("", `Web page: /protocols/${p.id}/`, "");
  return out.join("\n");
}
