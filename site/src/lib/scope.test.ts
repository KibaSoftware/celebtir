import { describe, expect, it } from "vitest";
import { loadSnapshot } from "./data";
import { sharedControllers } from "./graph";
import { controlShares, factorDistribution, insights, kpis, stageDistribution } from "./overview";
import { activityRows, assetTotals, tokenRows } from "./pages";
import {
  CHAIN_LABELS,
  chainOfAddress,
  chainOfProtocol,
  navVisible,
  pageChain,
  scope,
  scopeDays,
  scopeEvents,
  SCOPES,
  withChain,
} from "./scope";
import { buildRows, categoryCounts } from "./summary";
import type { ChangeEvent, Snapshot, SummaryDay } from "./types";
import type { ChainFilter } from "./viewState";

const EVM = "evm-kamino";
const EVM_MINT = "0x5555555555555555555555555555555555555555";

/** The loaded snapshot plus one HyperEVM protocol and one 0x token: both chains. */
function twoChains(): Snapshot {
  const s = structuredClone(scope(loadSnapshot(), "solana"));
  s.facts.protocols[EVM] = { ...s.facts.protocols["kamino-lend"]!, chain: "hyper_evm", name: "EVM Kamino", tokens: { [EVM_MINT]: ["collateral"] } };
  s.scores.protocols[EVM] = { ...s.scores.protocols["kamino-lend"]!, tokens: {} };
  // Scoping is chain-agnostic: this synthetic HyperEVM protocol is marked
  // assessed so every aggregator has something to count on both chains.
  s.coverage = { ...s.coverage, protocols: { ...s.coverage.protocols, [EVM]: { chain: "hyper_evm", assessment: "assessed" } } };
  s.facts.metrics[EVM] = { ...s.facts.metrics["kamino-lend"]!, tvl: { kind: "usd", value: 1_234_567 } };
  const anyMint = Object.keys(s.facts.tokens)[0]!;
  s.facts.tokens[EVM_MINT] = structuredClone(s.facts.tokens[anyMint]!);
  s.facts.tokens[EVM_MINT]!.symbol = "WHYPE";
  s.facts.tokens[EVM_MINT]!.mint_authority.evidence.forEach(e => { e.chain = "hyper_evm"; });
  s.scores.tokens[EVM_MINT] = s.scores.tokens[anyMint]!;
  return s;
}

const ev = (id: string, protocol: string | null, subject: string, source: ChangeEvent["source"] = "on_chain"): ChangeEvent => ({
  id,
  date: "2026-10-21",
  detected_within_days: 1,
  protocol,
  subject,
  kind: "controller_changed",
  severity: "warning",
  source,
  summary: id,
  signature: null,
});

const chainOf = (s: Snapshot, id: string) => chainOfProtocol(s.facts.protocols[id]?.chain);

describe("hui_1 one chain scope for every aggregator", () => {
  const s = twoChains();
  const ids = { all: Object.keys(s.scores.protocols), solana: [] as string[], hyperliquid: [] as string[] };
  for (const id of ids.all) ids[chainOf(s, id)].push(id);

  for (const chain of SCOPES) {
    it(`hui_1 ui_9 every aggregator over "${chain}" counts only that chain`, () => {
      const scoped = scope(s, chain);
      const expected = new Set(ids[chain]);
      expect(new Set(Object.keys(scoped.scores.protocols))).toEqual(expected);
      expect(new Set(Object.keys(scoped.facts.protocols))).toEqual(expected);

      const rows = buildRows(scoped, {}, []);
      expect(rows.every((r) => chain === "all" || r.chain === chain)).toBe(true);
      expect(categoryCounts(rows).all).toBe(expected.size);
      expect(Object.values(stageDistribution(rows)).reduce((a, b) => a + b, 0)).toBe(expected.size);
      for (const counts of Object.values(factorDistribution(rows))) {
        expect(Object.values(counts).reduce((a, b) => a + b, 0)).toBe(expected.size);
      }

      // TVL: the scoped total is the sum of the kept protocols' TVL.
      const tvl = [...expected].reduce((n, id) => {
        const m = s.facts.metrics[id];
        return n + (m?.tvl.kind === "usd" ? m.tvl.value : 0);
      }, 0);
      const value = scoped.scores.value_by_control;
      expect(value.all.total).toBe(tvl);
      expect(controlShares(value.all).reduce((n, x) => n + x.usd, 0)).toBe(tvl);
      expect(kpis(rows, value, [], [], "2026-10-21").totalTvl).toBe(tvl);
      expect(kpis(rows, value, [], [], "2026-10-21").assessed).toBe(expected.size);

      for (const r of activityRows(scoped)) expect(expected.has(r.id)).toBe(true);
      expect(Object.keys(scoped.facts.metrics).every((id) => expected.has(id))).toBe(true);
      expect(assetTotals(scoped).total).toBeGreaterThanOrEqual(0);

      for (const t of tokenRows(scoped)) {
        expect(chain === "all" || chainOfAddress(t.mint) === chain).toBe(true);
        for (const e of t.exposure) expect(expected.has(e.id)).toBe(true);
      }
      for (const c of sharedControllers(scoped)) {
        for (const p of c.protocols) expect(expected.has(p.id)).toBe(true);
      }
      for (const n of insights(rows, [], "2026-10-21", {})) {
        for (const i of n.items) expect(expected.has(i.id)).toBe(true);
      }
      expect(scoped.facts.hip3 === undefined || chain !== "solana").toBe(true);
    });
  }

  it("hui_1 all is the union: the scoped totals add up", () => {
    const sum = (c: ChainFilter) => scope(s, c).scores.value_by_control.all.total;
    expect(sum("solana") + sum("hyperliquid")).toBe(sum("all"));
    expect(ids.hyperliquid).toEqual([EVM]);
  });

  it("hui_1 the recomputed value by control matches the scorer's on the loaded data", () => {
    const live = loadSnapshot();
    const recomputed = scope(live, "solana").scores.value_by_control;
    // Every loaded protocol is Solana, so Solana's scope equals the published figures.
    expect(recomputed.all.total).toBe(live.scores.value_by_control.all.total);
    for (const [cls, usd] of Object.entries(live.scores.value_by_control.all.classes)) {
      expect(recomputed.all.classes[cls] ?? 0, cls).toBe(usd);
    }
    expect([...recomputed.excluded].sort()).toEqual([...live.scores.value_by_control.excluded].sort());
  });
});

describe("hui_1 events, days and links follow the chain", () => {
  const s = twoChains();

  it("hui_1 events: by protocol, by HIP-3 subject, by mint; methodology belongs to every chain", () => {
    const events = [
      ev("sol", "kamino-lend", "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
      ev("evm", EVM, EVM),
      ev("hip3", null, "hip3:xyz.oracle_updater"),
      ev("mint", null, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
      ev("method", null, "methodology", "methodology"),
    ];
    const ids = (c: ChainFilter) => scopeEvents(events, s, c).map((e) => e.id);
    expect(ids("all")).toEqual(["sol", "evm", "hip3", "mint", "method"]);
    expect(ids("solana")).toEqual(["sol", "mint", "method"]);
    expect(ids("hyperliquid")).toEqual(["evm", "hip3", "method"]);
  });

  it("hui_1 summary days use by_chain, and older days count as Solana", () => {
    const old: SummaryDay = {
      date: "2026-10-01",
      protocols: 12,
      total_tvl_usd: 100,
      value_by_control: { classes: { immutable: 100 }, total: 100 },
      stages: { unrated: 0, stage_0: 12, stage_1: 0, stage_2: 0 },
      methodology_version: "0.1",
    };
    const split: SummaryDay = {
      ...old,
      date: "2026-10-21",
      protocols: 13,
      total_tvl_usd: 130,
      by_chain: {
        solana: { protocols: 12, total_tvl_usd: 100, value_by_control: old.value_by_control, stages: old.stages },
        hyperliquid: { protocols: 1, total_tvl_usd: 30, value_by_control: { classes: { single_key: 30 }, total: 30 }, stages: { unrated: 0, stage_0: 1, stage_1: 0, stage_2: 0 } },
      },
    };
    expect(scopeDays([old, split], "all").map((d) => d.total_tvl_usd)).toEqual([100, 130]);
    expect(scopeDays([old, split], "solana").map((d) => d.total_tvl_usd)).toEqual([100, 100]);
    expect(scopeDays([old, split], "hyperliquid").map((d) => [d.protocols, d.total_tvl_usd])).toEqual([
      [0, 0],
      [1, 30],
    ]);
  });

  it("hui_1 internal links keep the chain; the default stays out of the URL", () => {
    expect(withChain("/risk/", "all")).toBe("/risk/");
    expect(withChain("/risk/", "hyperliquid")).toBe("/risk/?chain=hyperliquid");
    expect(withChain("/risk/?factor=upgrade_control", "solana")).toBe("/risk/?factor=upgrade_control&chain=solana");
    expect(withChain("/methodology/#hyperliquid", "solana")).toBe("/methodology/?chain=solana#hyperliquid");
    expect(withChain("https://example.org/", "solana")).toBe("https://example.org/");
    expect(navVisible("/hip3/", "solana")).toBe(false);
    expect(navVisible("/hip3/", "all")).toBe(true);
    expect(navVisible("/risk/", "hyperliquid")).toBe(true);
  });

  it("hui_1 protocol, token and HIP-3 pages fix the picker to their chain", () => {
    const chains = { "kamino-lend": "solana", [EVM]: "hyperliquid", [`token:${EVM_MINT}`]: "hyperliquid", "token:EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": "solana" } as const;
    expect(pageChain("/protocols/kamino-lend/", chains)).toBe("solana");
    expect(pageChain(`/protocols/${EVM}/`, chains)).toBe("hyperliquid");
    expect(pageChain("/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/", chains)).toBe("solana");
    expect(pageChain(`/tokens/${EVM_MINT}/`, chains)).toBe("hyperliquid");
    expect(pageChain("/hip3/", chains)).toBe("hyperliquid");
    expect(pageChain("/risk/", chains)).toBeNull();
    expect(pageChain("/tokens/", chains)).toBeNull();
    expect(CHAIN_LABELS.all).toBe("All chains");
  });
});
