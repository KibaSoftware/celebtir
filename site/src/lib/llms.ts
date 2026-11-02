// UI-14: /llms.txt, following the llms.txt convention: a title, a one-line
// summary, then sections of links with a short description each. Every
// protocol is listed; an unassessed one names no Stage (F5).

import { isAssessed } from "./assessment";
import { describeController } from "./controller";
import { STAGE_LABELS } from "./labels";
import type { Snapshot } from "./types";

const CHAIN_NAMES: Record<string, string> = { solana: "Solana", hyper_evm: "HyperEVM", hyper_core: "HyperCore" };

export function llmsTxt(s: Snapshot): string {
  const protocols = Object.entries(s.facts.protocols)
    .map(([id, p]) => {
      const score = isAssessed(s, id) ? s.scores.protocols[id] : undefined;
      const core = Object.values(p.programs).filter(x => x.role === "core").map(x => describeController(x.upgrade_control.value)).sort();
      return {
        id,
        name: p.name ?? id,
        chain: CHAIN_NAMES[p.chain] ?? p.chain,
        status: score ? STAGE_LABELS[score.stage.stage] : "Unassessed (contract facts listed, not rated)",
        controller: score?.controller_summary ?? core[0] ?? "no core programs",
      };
    })
    .sort((a, b) => a.name.localeCompare(b.name));
  const chains = [...new Set(protocols.map(p => p.chain))].sort();
  return [
    "# Celebtir",
    "",
    `> An open, evidence-linked scorecard of who controls DeFi protocols on ${chains.join(" and ")}: who can change their code and settings, how fast, and what they depend on. Every rating and fact links to the on-chain account it was read from. Not investment advice.`,
    "",
    `Data as of the latest daily snapshot, methodology v${s.scores.methodology_version}. Rated chains: ${s.coverage.assessed_chains.map(c => CHAIN_NAMES[c] ?? c).join(", ")}; other chains are listed unassessed, with facts but no Stage or ratings. Licence: CC BY 4.0.`,
    "",
    "## Protocols",
    "",
    ...protocols.map((p) => `- [${p.name}](/protocols/${p.id}.md): ${p.chain}; ${p.status}; upgrade controller ${p.controller}`),
    "",
    "## Tables",
    "",
    "- [Value by protocol](/value.md): TVL and upgrade-control class per protocol, as Markdown",
    "- [Activity by protocol](/activity.md): latest-day volume and fees with the 7-day change, as Markdown",
    "",
    "## Reference",
    "",
    "- [Methodology](/methodology/): the six factors, Stages and every threshold, versioned",
    "- [Glossary](/glossary/): upgrade authority, time lock, config authority and other terms",
    "- [About](/about/): what the ratings mean and don't, corrections policy, licences",
    "- [API](/api/): the same data as static JSON with schemas",
    "",
    "## Optional",
    "",
    "- [Change log](/changelog/): every detected change, newest first",
    "- [Atom feed](/feeds/all.atom)",
    "",
  ].join("\n");
}
