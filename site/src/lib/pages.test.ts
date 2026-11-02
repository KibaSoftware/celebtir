import { describe, expect, it } from "vitest";
import { loadHistory, loadSnapshot } from "./data";
import { activityRows, activitySeries, assetTotals, factorRows, supplySeries, tokenPage, tokenRows } from "./pages";
import { protocolIds, usersOf } from "./testSnapshot";
import type { TokenHistoryFile } from "./types";

const USDC = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const JITOSOL = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

describe("risk page", () => {
  it("a factor sub-tab lists every protocol with rating, reason, next step and evidence", () => {
    const s = loadSnapshot();
    const rows = factorRows(s, "upgrade_control");
    expect([...rows.map((r) => r.id)].sort()).toEqual(protocolIds(s));
    for (const r of rows) {
      expect(r.score.reason.length).toBeGreaterThan(0);
      expect(r.score.next_step.length).toBeGreaterThan(0);
    }
    expect(rows[0]!.score.evidence.length).toBeGreaterThan(0);
  });
});

describe("value page", () => {
  it("asset groups add up across protocols with known groups", () => {
    const t = assetTotals(loadSnapshot());
    expect(t.total).toBe(t.stablecoins + t.sol + t.btc + t.other);
    expect(t.total).toBeGreaterThan(0);
  });
});

describe("activity page", () => {
  it("each protocol shows volume and fees separately, with the reason when unavailable", () => {
    const snapshot = structuredClone(loadSnapshot());
    // A complete explicit observation, independent of live provider gaps.
    const date = snapshot.snapshotDate ?? snapshot.run.started_at.slice(0, 10);
    const previous = new Date(Date.parse(date) - 86400000).toISOString().slice(0, 10);
    snapshot.facts.metrics["kamino-lend"]!.fees = {kind:"usd",latest:100,days:[{date:previous,usd:100}],change_7d_bp:null};
    const rows = activityRows(snapshot);
    const kamino = rows.find((r) => r.id === "kamino-lend")!;
    expect(kamino.fees.kind).toBe("usd");
    for (const r of rows) {
      for (const a of [r.fees, r.volume]) if (a.kind === "unavailable") expect(a.reason.length).toBeGreaterThan(0);
    }
  });

  it("daily totals sum every protocol's reported days", () => {
    const s = loadSnapshot();
    const series = activitySeries(s, "fees");
    const k = s.facts.metrics["kamino-lend"]!.fees;
    expect(k.kind).toBe("usd");
    if (k.kind !== "usd") return;
    const day = k.days[0]!;
    const total = Object.values(s.facts.metrics).reduce(
      (n, m) => n + (m.fees.kind === "usd" ? (m.fees.days.find((d) => d.date === day.date)?.usd ?? 0) : 0),
      0,
    );
    expect(series.find((p) => p.date === day.date)!.usd).toBe(total);
    expect(series.map((p) => p.date)).toEqual([...series.map((p) => p.date)].sort());
  });
});

describe("tok_6 tokens", () => {
  it("tok_6 the token table has four ratings and the protocols exposed to each token", () => {
    const s = loadSnapshot();
    const rows = tokenRows(s);
    const symbols = Object.values(s.facts.tokens).map((t) => t.symbol);
    expect([...rows.map((r) => r.symbol)].sort()).toEqual([...symbols].sort());
    const jito = rows.find((r) => r.mint === JITOSOL)!;
    expect(Object.keys(jito.ratings)).toEqual(["supply_control", "freeze_and_seize", "transfer_restrictions", "provenance"]);
    expect([...jito.exposure.map((e) => e.id)].sort()).toEqual(usersOf(s, JITOSOL));
    expect(jito.exposure.length).toBeGreaterThan(1);
  });

  it("tok_6 a token page carries identity, controllers, four rated factors with evidence and its risk summary", () => {
    const s = loadSnapshot();
    const p = tokenPage(s, USDC);
    expect(p.identity?.issuer).toBe("Circle");
    expect(p.controls.map((c) => c.label)).toEqual(["Mint authority", "Freeze authority"]);
    expect(p.controls[1]!.text).toBe("Single key");
    expect(Object.values(p.factors).every((f) => f.evidence.length > 0)).toBe(true);
    expect(p.risks.some((r) => r.text.startsWith("Your balance can be frozen if"))).toBe(true);
    expect([...p.exposure.map((e) => e.id)].sort()).toEqual(usersOf(s, USDC));
    expect(p.exposure.map((e) => e.id)).toContain("kamino-lend");
  });
});

describe("tok_9 supply history", () => {
  const history = loadHistory<TokenHistoryFile>(`tokens/${USDC}`)!;

  it("tok_9 no supply line before 30 snapshots exist", () => {
    expect(history.days.length).toBeLessThan(30);
    expect(supplySeries(history)).toBeNull();
  });

  it("tok_9 a 30-day supply line once 30 snapshots exist, in whole tokens", () => {
    const days = Array.from({ length: 35 }, (_, i) => ({
      date: `2026-09-${String(i + 1).padStart(2, "0")}`,
      supply: 5_000_000 + i * 1_000_000,
      decimals: 6,
      price_usd_micros: null,
    }));
    const line = supplySeries({ ...history, days })!;
    expect(line).toHaveLength(30);
    expect(line[0]!.supply).toBe(10);
    expect(line[29]!.date).toBe(days[34]!.date);
  });
});
