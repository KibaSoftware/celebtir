import { describe, expect, it } from "vitest";
import { loadSnapshot } from "./data";
import { buildGraph, sharedControllers, worstRating } from "./graph";
import { actorsFor } from "./permissions";
import { JITOSOL, usersOf } from "./testSnapshot";

const KAMINO_MS = "6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM";
const JITOSOL_MINT_AUTHORITY = "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS";

describe("ui_19 graph nodes and labelled edges", () => {
  const s = loadSnapshot();
  const g = buildGraph(s, "kamino-lend");

  it("ui_19 Kamino's graph shows every controller in its registry", () => {
    const { actors } = actorsFor(s.facts.protocols["kamino-lend"]!, s.facts.tokens);
    for (const a of actors) expect(g.nodes.map((n) => n.id)).toContain(`node-${a.id}`);
    expect(g.nodes.find((n) => n.id === `node-${KAMINO_MS}`)!.kind).toBe("multisig");
  });

  it("ui_19 the protocol, its programs, admin accounts and tokens are nodes too", () => {
    const kinds = new Set(g.nodes.map((n) => n.kind));
    for (const k of ["protocol", "program", "account", "token", "multisig", "key"]) expect(kinds).toContain(k);
    expect(g.nodes.filter((n) => n.kind === "protocol")).toHaveLength(1);
  });

  it("ui_19 every edge is labelled with its power and delay", () => {
    const upgrade = g.edges.find((e) => e.source === `node-${KAMINO_MS}`)!;
    expect(upgrade.target).toBe("node-program-KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");
    expect(upgrade.label).toBe("can upgrade · 24 h delay");
    for (const e of g.edges) expect(e.label.length).toBeGreaterThan(3);
    const ids = new Set(g.nodes.map((n) => n.id));
    for (const e of g.edges) {
      expect(ids.has(e.source), e.source).toBe(true);
      expect(ids.has(e.target), e.target).toBe(true);
    }
  });

  it("ui_19 nodes sit in columns: controllers left of what they control, the protocol on the right", () => {
    const col = (id: string) => g.nodes.find((n) => n.id === id)!.column;
    for (const e of g.edges) expect(col(e.source)).toBeLessThan(col(e.target));
  });

  it("ui_19 the graph is the same on every build", () => {
    expect(buildGraph(s, "kamino-lend")).toEqual(g);
  });

  it("ui_19 program-controlled token authorities remain left of their subjects", () => {
    const snapshot = structuredClone(s);
    snapshot.facts.tokens[JITOSOL]!.mint_authority.value = {
      kind: "program_controlled", key: JITOSOL_MINT_AUTHORITY,
      program: "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy",
      upgrade: {kind:"single_key",key:"11111111111111111111111111111111"},
    };
    const graph = buildGraph(snapshot, "kamino-lend");
    const column = (id: string) => graph.nodes.find(n => n.id === id)!.column;
    expect(graph.nodes.some(n => n.id === `node-${JITOSOL_MINT_AUTHORITY}`)).toBe(true);
    for (const edge of graph.edges) expect(column(edge.source), edge.id).toBeLessThan(column(edge.target));
  });
});

describe("ui_21 node ratings come from the scores", () => {
  const s = loadSnapshot();
  const g = buildGraph(s, "kamino-lend");

  it("ui_21 programs, admin accounts, tokens and controllers carry the scorer's rating", () => {
    const node = (id: string) => g.nodes.find((n) => n.id === id)!;
    const klend = s.scores.protocols["kamino-lend"]!.programs["KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"]!;
    expect(node("node-program-KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD").rating).toBe(klend.upgrade_control.rating);
    expect(node(`node-${KAMINO_MS}`).rating).toBe(klend.upgrade_control.rating);
    const admin = Object.values(s.scores.protocols["kamino-lend"]!.admin)[0]!;
    expect(node("node-account-7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF").rating).toBe(admin.rating);
    expect(node("node-protocol").rating).toBeNull();
    for (const n of g.nodes.filter((n) => n.kind !== "protocol")) expect(n.rating, n.id).not.toBeNull();
  });

  it("ui_21 the worst rating ranks High over Unknown over Medium over Low", () => {
    expect(worstRating(["low", "medium"])).toBe("medium");
    expect(worstRating(["medium", "unknown"])).toBe("unknown");
    expect(worstRating(["unknown", "high", "low"])).toBe("high");
    expect(worstRating(["not_applicable"])).toBe("not_applicable");
    expect(worstRating([])).toBeNull();
  });
});

describe("ui_22 shared controllers", () => {
  it("ui_22 a key controlling two protocols appears once, linked to both", () => {
    const s = loadSnapshot();
    const shared = sharedControllers(s);
    const jito = shared.filter((c) => c.id === `solana:${JITOSOL_MINT_AUTHORITY}`);
    expect(jito).toHaveLength(1);
    // Every protocol using JitoSOL is exposed to its mint authority.
    expect(jito[0]!.protocols.map((p) => p.id)).toEqual(usersOf(s, JITOSOL));
    expect(jito[0]!.protocols.length).toBeGreaterThan(1);
    for (const c of shared) expect(c.protocols.length).toBeGreaterThan(1);
    expect(shared.some((c) => c.id === "unreadable")).toBe(false);
  });

  it("ui_22 a shared node in one protocol's graph names the others", () => {
    const s = loadSnapshot();
    const g = buildGraph(s, "kamino-lend");
    const node = g.nodes.find((n) => n.id === `node-${JITOSOL_MINT_AUTHORITY}`)!;
    expect(node.sharedWith.map((p) => p.id)).toEqual(usersOf(s, JITOSOL).filter((id) => id !== "kamino-lend"));
  });
});
