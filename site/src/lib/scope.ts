// HUI-1, UI-9: one chain scope for every page. A page asks for the scoped
// data once and hands it to the existing aggregators; no page filters by
// chain on its own. "all" is the unfiltered scope, so sums and unions need
// no special case.

import { CONTROL_CLASSES, type ChangeEvent, type ControlClass, type Snapshot, type SummaryDay, type ValueRow } from "./types";
import { CHAINS, type ChainFilter } from "./viewState";

export type Chain = Exclude<ChainFilter, "all">;
export const SCOPES = CHAINS;

/** The one place chain names are spelled (no hard-coded "Solana" in pages). */
export const CHAIN_LABELS: Record<ChainFilter, string> = {
  all: "All chains",
  solana: "Solana",
  hyperliquid: "Hyperliquid",
};

/** A protocol's `chain` field (`solana`, `hyper_evm`, `hyper_core`) → its filter. */
export function chainOfProtocol(chain: string | undefined): Chain {
  if (chain === undefined || chain === "solana") return "solana"; // legacy Solana protocol schema
  if (chain === "hyper_evm" || chain === "hyper_core" || chain === "hyperliquid") return "hyperliquid";
  throw new Error(`Unsupported chain provenance: ${chain}`);
}

/** Mints and accounts: 0x addresses are Hyperliquid, base58 is Solana. */
export function chainOfAddress(address: string): Chain {
  return address.startsWith("0x") ? "hyperliquid" : "solana";
}

/** Token provenance comes from its account evidence, never its address spelling. */
export function chainOfToken(s: Snapshot, mint: string): Chain | undefined {
  const t = s.facts.tokens[mint];
  if (!t) return undefined;
  const chains = new Set(t.mint_authority.evidence.map(e => chainOfProtocol(e.chain)));
  return chains.size === 1 ? [...chains][0] : undefined;
}

export const inScope = (scope: ChainFilter, chain: Chain) => scope === "all" || scope === chain;

const emptyRow = (): ValueRow => ({ classes: Object.fromEntries(CONTROL_CLASSES.map((c) => [c, 0])), total: 0 });

/**
 * The snapshot restricted to one chain: protocols, their scores and
 * metrics, tokens, and HIP-3 DEXs (Hyperliquid only). Value by control is
 * recomputed from the kept protocols the way the scorer does it.
 */
export function scope(s: Snapshot, chain: ChainFilter): Snapshot {
  const keep = (id: string) => inScope(chain, chainOfProtocol(s.facts.protocols[id]?.chain));
  const pick = <T,>(o: Record<string, T>, ok: (k: string) => boolean) =>
    Object.fromEntries(Object.entries(o).filter(([k]) => ok(k)));
  const mintOk = (m: string) => chain === "all" || chainOfToken(s, m) === chain;

  const protocols = pick(s.scores.protocols, keep);
  const all = emptyRow();
  const byCategory: Record<string, ValueRow> = {};
  const excluded: string[] = [];
  for (const id of Object.keys(protocols).sort()) {
    const m = s.facts.metrics[id];
    if (m?.tvl.kind !== "usd") {
      excluded.push(id);
      continue;
    }
    const cls: ControlClass = protocols[id]!.control_class;
    const cat = s.facts.protocols[id]?.category ?? "uncategorized";
    for (const row of [all, (byCategory[cat] ??= emptyRow())]) {
      row.classes[cls] = (row.classes[cls] ?? 0) + m.tvl.value;
      row.total += m.tvl.value;
    }
  }

  return {
    ...s,
    facts: {
      ...s.facts,
      protocols: pick(s.facts.protocols, keep),
      metrics: pick(s.facts.metrics, keep),
      tokens: pick(s.facts.tokens, mintOk),
      hip3: chain === "solana" ? undefined : s.facts.hip3,
    },
    scores: {
      ...s.scores,
      protocols,
      tokens: pick(s.scores.tokens, mintOk),
      value_by_control: { all, by_category: byCategory, excluded },
    },
  };
}

/** Which chain an event is about; methodology changes concern every chain. */
export function chainOfEvent(e: ChangeEvent, s: Snapshot): Chain | null | undefined {
  if (e.chain) return chainOfProtocol(e.chain);
  if (e.evidence?.length) return chainOfProtocol(e.evidence[0]!.chain);
  if (e.protocol) return s.facts.protocols[e.protocol] ? chainOfProtocol(s.facts.protocols[e.protocol]!.chain) : undefined;
  if (e.subject.startsWith("hip3:")) return "hyperliquid";
  if (e.source === "methodology" || e.subject === "methodology") return null;
  return chainOfToken(s, e.subject);
}

export function scopeEvents(events: ChangeEvent[], s: Snapshot, chain: ChainFilter): ChangeEvent[] {
  if (chain === "all") return events;
  return events.filter((e) => {
    const c = chainOfEvent(e, s);
    return c === null || c === chain;
  });
}

/**
 * Summary days for one chain. Days written before `by_chain` existed were
 * Solana only, so their totals are Solana's and Hyperliquid had nothing.
 */
export function scopeDays(days: SummaryDay[], chain: ChainFilter): SummaryDay[] {
  if (chain === "all") return days;
  return days.map((d) => {
    const part = d.by_chain?.[chain];
    // An unassessed chain carries protocols and TVL, never Stages (F5).
    if (part) {
      return {
        ...d,
        protocols: part.assessment === "unassessed" ? 0 : part.protocols,
        unassessed: part.assessment === "unassessed" ? part.protocols : 0,
        total_tvl_usd: part.total_tvl_usd,
        value_by_control: part.value_by_control ?? emptyRow(),
        stages: part.stages ?? { unrated: 0, stage_0: 0, stage_1: 0, stage_2: 0 },
        by_chain: undefined,
      };
    }
    if (chain === "solana" && !d.by_chain) return d;
    return {
      ...d,
      protocols: 0,
      total_tvl_usd: 0,
      value_by_control: emptyRow(),
      stages: { unrated: 0, stage_0: 0, stage_1: 0, stage_2: 0 },
      by_chain: undefined,
    };
  });
}

/**
 * A page about one chain's entity fixes the picker (HUI-1): protocol pages
 * take their protocol's chain, token pages their mint's, HIP-3 is
 * Hyperliquid. Other pages follow the picker (null).
 */
export function pageChain(pathname: string, protocolChains: Record<string, Chain>): Chain | null {
  const parts = pathname.split("/").filter(Boolean);
  if (parts[0] === "protocols" && parts[1]) return protocolChains[parts[1]] ?? "solana";
  if (parts[0] === "tokens" && parts[1]) return protocolChains[`token:${parts[1]}`] ?? null;
  if (parts[0] === "hip3" || parts[0] === "hyperevm") return "hyperliquid";
  return null;
}

/**
 * HUI-1: an internal link that keeps the chosen chain. The default ("all")
 * stays out of the URL; any hash stays last.
 */
export function withChain(href: string, chain: ChainFilter): string {
  if (!href.startsWith("/") || href.startsWith("//")) return href;
  const url = new URL(href, "https://celebtir.invalid");
  if (chain === "all") url.searchParams.delete("chain");
  else url.searchParams.set("chain", chain);
  return `${url.pathname}${url.search.replace(/%2C/gi, ",")}${url.hash}`;
}

/** Nav entries with nothing on the chosen chain are hidden (HIP-3 under Solana). */
export function navVisible(href: string, chain: ChainFilter): boolean {
  return !(href === "/hip3/" && chain === "solana");
}
