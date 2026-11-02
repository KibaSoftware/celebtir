// The global search index (UI-3): protocols, tokens and the controllers the
// facts name. Built at build time from the snapshot; pure, so it is tested
// without a browser. It only indexes what the snapshot holds.

import { chainOfProtocol, chainOfToken, type Chain } from "./scope";
import { STAGE_LABELS } from "./labels";
import { actorsFor, POWER_LABELS, UNREADABLE } from "./permissions";
import { CATEGORY_LABELS } from "./summary";
import type { Snapshot } from "./types";
import type { Tab } from "./viewState";

export type SearchKind = "Protocol" | "Token" | "Controller" | "Market";

export interface SearchItem {
  label: string;
  chain?: Chain;
  href: string;
  kind: SearchKind;
  /** One descriptive line under the label. */
  detail?: string;
  /** Extra text matched by the search but not shown (addresses, ids). */
  keywords?: string;
}

export const KIND_ORDER: SearchKind[] = ["Protocol", "Token", "Controller", "Market"];

export function searchIndex(s: Snapshot): SearchItem[] {
  const protocols: SearchItem[] = Object.entries(s.facts.protocols).map(([id, p]) => {
    const score = s.scores.protocols[id];
    const category = CATEGORY_LABELS[p.category as Tab]?.label ?? p.category;
    return {
      chain: chainOfProtocol(p.chain),
      label: p.name ?? id,
      href: `/protocols/${id}/`,
      kind: "Protocol",
      detail: [category, score ? STAGE_LABELS[score.stage.stage] : "Unassessed", score?.controller_summary].filter(Boolean).join(" · "),
      keywords: id,
    };
  });
  const tokens: SearchItem[] = Object.entries(s.facts.tokens).map(([mint, t]) => ({
    chain: chainOfToken(s, mint),
    label: t.symbol,
    href: `/tokens/${mint}/`,
    kind: "Token",
    detail: [t.identity?.issuer ?? null, t.program === "token2022" ? "Token-2022" : "SPL Token"].filter(Boolean).join(" · "),
    keywords: mint,
  }));

  // One entry per controller, linking its node in the first protocol it controls.
  const controllers = new Map<string, SearchItem & { protocols: string[]; powers: Set<string> }>();
  for (const id of Object.keys(s.facts.protocols).sort()) {
    const facts = s.facts.protocols[id]!;
    for (const a of actorsFor(facts, s.facts.tokens).actors) {
      if (a.id === UNREADABLE || !a.address) continue;
      const key = `${facts.chain ?? "solana"}:${a.id}`;
      const entry = controllers.get(key) ?? {
        chain: chainOfProtocol(facts.chain),
        label: a.label,
        href: `/protocols/${id}/#node-${a.id}`,
        kind: "Controller" as const,
        keywords: a.address,
        protocols: [],
        powers: new Set<string>(),
      };
      entry.protocols.push(facts.name ?? id);
      for (const g of a.grants) entry.powers.add(POWER_LABELS[g.power].toLowerCase());
      controllers.set(key, entry);
    }
  }
  const controllerItems: SearchItem[] = [...controllers.values()].map(({ protocols: ps, powers, ...item }) => ({
    ...item,
    detail: `${ps.join(", ")}: ${[...powers].join(", ")}`,
  }));

  const byLabel = (a: SearchItem, b: SearchItem) => a.label.localeCompare(b.label);
  const markets: SearchItem[] = s.facts.hip3?.kind === "listed" ? Object.entries(s.facts.hip3.dexs).flatMap(([id,dex]) => Object.keys(dex.oi_caps.value).map(asset => ({label: asset, kind: "Market" as const, chain: "hyperliquid" as const, href: `/hip3/#${encodeURIComponent(id)}`, detail: `${dex.full_name} · Unassessed`, keywords: `${id} ${dex.deployer.value}`}))) : [];
  return [...protocols.sort(byLabel), ...tokens.sort(byLabel), ...controllerItems.sort(byLabel), ...markets.sort(byLabel)];
}

/** Matches by label first, then by detail or keyword; at most `limit` per kind. */
export function searchMatches(items: SearchItem[], query: string, limit = 6): SearchItem[] {
  const q = query.trim().toLowerCase();
  if (!q) return [];
  const scored = items
    .map((item) => {
      const label = item.label.toLowerCase();
      const rank = label.startsWith(q) ? 0 : label.includes(q) ? 1 : `${item.detail ?? ""} ${item.keywords ?? ""}`.toLowerCase().includes(q) ? 2 : -1;
      return { item, rank };
    })
    .filter((x) => x.rank >= 0);
  return KIND_ORDER.flatMap((kind) =>
    scored
      .filter((x) => x.item.kind === kind)
      .sort((a, b) => a.rank - b.rank || a.item.label.localeCompare(b.item.label))
      .slice(0, limit)
      .map((x) => x.item),
  );
}
