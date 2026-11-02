// UI-19 to UI-22: the control graph as data. Built at build time; the canvas,
// the side panel and the list view all render this same model.

import { isAssessed } from "./assessment";
import { delayText, describeController, loaderName } from "./controller";
import { shortAddress } from "./format";
import { STAGE_LABELS } from "./labels";
import { actorsFor, POWER_LABELS, UNREADABLE, type Actor, type Grant, type Power } from "./permissions";
import type { Rating } from "./tokens";
import type { Evidence, Snapshot } from "./types";

export type NodeKind = "protocol" | "program" | "account" | "token" | "multisig" | "key" | "config" | "timelock" | "unreadable";

export const KIND_LABELS: Record<NodeKind, string> = {
  protocol: "Protocol",
  program: "Program",
  account: "Admin account",
  token: "Token",
  multisig: "Multisig",
  key: "Key",
  config: "Config authority",
  timelock: "Timelock",
  unreadable: "Unreadable controller",
};

export interface GraphNode {
  /** Also the list view's DOM id, so `#node-…` links reach both views. */
  id: string;
  kind: NodeKind;
  title: string;
  address: string | null;
  /** Null for the protocol node, which shows its Stage instead. */
  rating: Rating | null;
  ratingReason: string;
  facts: { label: string; value: string }[];
  evidence: Evidence[];
  /** Multisigs only: voters, and their keys when published. */
  signers: { count: number; keys: string[] } | null;
  /** 0 = leftmost. Every edge points to a higher column. */
  column: number;
  row: number;
  /** Other protocols this controller also controls (UI-22). */
  sharedWith: { id: string; name: string }[];
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label: string;
}

export interface ControlGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

const RISK: Record<Rating, number> = { not_applicable: 0, low: 1, medium: 2, unknown: 3, high: 4 };

/** The riskiest rating: High, then Unknown, then Medium, then Low. */
export function worstRating(rs: Rating[]): Rating | null {
  return rs.reduce<Rating | null>((w, r) => (w === null || RISK[r] > RISK[w] ? r : w), null);
}

const VERBS: Record<Power, string> = {
  upgrade: "can upgrade",
  parameters: "can change parameters",
  pause: "can pause",
  funds: "can move funds",
  freeze: "can freeze",
  mint: "can mint",
  reconfigure: "can change members",
  sign: "can sign for",
  propose: "can propose through",
};

const ROLE_LABELS: Record<string, string> = {
  collateral: "collateral",
  borrowable: "borrowable",
  deposit: "deposit",
  lp: "liquidity",
};

const actorNodeId = (actorId: string) => `node-${actorId}`;

function actorKind(a: Actor): NodeKind {
  if (a.grants.some((g) => g.power === "reconfigure")) return "config";
  switch (a.kind) {
    case "multisig":
    case "cannot_act":
      return "multisig";
    case "timelock":
      return "timelock";
    case "program_controlled":
      return "program";
    case "unavailable":
      return "unreadable";
    default:
      return "key";
  }
}

export interface SharedController {
  id: string;
  label: string;
  address: string | null;
  /** The controller kind, so the page groups PDAs apart from keys (F2). */
  kind: Actor["kind"];
  /** For a verified program-derived address: the program that signs for it. */
  program: string | null;
  protocols: { id: string; name: string; powers: Power[]; descriptions: string[]; actorId: string }[];
}

/** UI-22: keys and multisigs that control more than one protocol, each listed once. */
export function sharedControllers(s: Snapshot): SharedController[] {
  const byId = new Map<string, SharedController>();
  for (const id of Object.keys(s.facts.protocols).sort()) {
    const facts = s.facts.protocols[id]!;
    for (const a of actorsFor(facts, s.facts.tokens).actors) {
      if (a.id === UNREADABLE) continue;
      const scopedId = `${facts.chain ?? "solana"}:${a.id}`;
      const entry = byId.get(scopedId) ?? {
        id: scopedId,
        label: a.label,
        address: a.address,
        kind: a.kind,
        program: a.controller.kind === "program_controlled" ? a.controller.program : null,
        protocols: [],
      };
      entry.protocols.push({ id, name: facts.name ?? id, actorId: a.id, descriptions: [...new Set(a.grants.map(g => `${g.subjectLabel}: ${POWER_LABELS[g.power].toLowerCase()}`))], powers: [...new Set(a.grants.map((g) => g.power))] });
      byId.set(scopedId, entry);
    }
  }
  return [...byId.values()]
    .filter((c) => c.protocols.length > 1)
    .sort((a, b) => b.protocols.length - a.protocols.length || a.id.localeCompare(b.id));
}

export function buildGraph(s: Snapshot, id: string): ControlGraph {
  const facts = s.facts.protocols[id];
  if (!facts) throw new Error(`unknown protocol ${id}`);
  // F5: an unassessed protocol's graph shows its facts with no ratings.
  const score = isAssessed(s, id) ? s.scores.protocols[id] : undefined;
  const { actors } = actorsFor(facts, s.facts.tokens);
  const names = new Map(facts.sources.map((x) => [x.address, x.label]));
  const shared = new Map(sharedControllers(s).filter(c => c.protocols.some(p => p.id === id)).map((c) => [c.address, c]));

  // Ratings of each grant, from where the scorer published them.
  const base = (g: Grant): Rating | null => {
    if (!score) return null;
    switch (g.ref.kind) {
      case "program":
        return score.programs[g.ref.address]?.upgrade_control.rating ?? "unknown";
      case "admin":
        return score.admin[g.ref.key]?.rating ?? score.factors.admin_powers.rating;
      case "token":
        return (s.scores.tokens[g.ref.mint]?.factors[g.ref.factor]?.rating as Rating | undefined) ?? "unknown";
      case "multisig":
        return null;
    }
  };
  const byAddress = new Map(actors.map((a) => [a.id, a]));
  const actorRating = (a: Actor, seen = new Set<string>()): Rating | null => {
    if (seen.has(a.id)) return null;
    seen.add(a.id);
    return worstRating(
      a.grants.flatMap((g) => {
        if (g.ref.kind !== "multisig") return [base(g)!];
        // Whoever can change a multisig holds everything the multisig holds.
        const target = byAddress.get(g.ref.address);
        const r = target ? actorRating(target, seen) : null;
        return r ? [r] : [];
      }),
    );
  };

  const nodes: Omit<GraphNode, "column" | "row">[] = [];
  const edges = new Map<string, { source: string; target: string; labels: string[] }>();
  const edge = (source: string, target: string, label: string) => {
    const key = `${source}->${target}`;
    const e = edges.get(key) ?? { source, target, labels: [] };
    if (!e.labels.includes(label)) e.labels.push(label);
    edges.set(key, e);
  };

  // Protocol.
  nodes.push({
    id: "node-protocol",
    kind: "protocol",
    title: facts.name ?? id,
    address: null,
    rating: null,
    ratingReason: score ? `${STAGE_LABELS[score.stage.stage]}.` : "Unassessed: no Stage is published for this chain.",
    facts: [
      { label: "Stage", value: score ? STAGE_LABELS[score.stage.stage] : "Unassessed" },
      { label: "Category", value: facts.category ?? "uncategorized" },
    ],
    evidence: [],
    signers: null,
    sharedWith: [],
  });

  // Programs.
  for (const [address, p] of Object.entries(facts.programs).sort(([a, x], [b, y]) =>
    x.role === y.role ? a.localeCompare(b) : x.role === "core" ? -1 : 1,
  )) {
    const ps = score?.programs[address];
    const nodeId = `node-program-${address}`;
    nodes.push({
      id: nodeId,
      kind: "program",
      title: names.get(address) ?? shortAddress(address),
      address,
      rating: score ? (ps?.upgrade_control.rating ?? "unknown") : null,
      ratingReason: score ? (ps?.upgrade_control.reason ?? "Not scored.") : "Unassessed: facts only.",
      facts: [
        { label: "Role", value: p.role === "core" ? "Core" : "Periphery" },
        { label: "Loader", value: loaderName(p.loader) ?? "Not read" },
        { label: "Upgrade controller", value: describeController(p.upgrade_control.value) },
        { label: "Verification", value: p.code_transparency.value.kind.replaceAll("_", " ") },
      ],
      evidence: [...p.upgrade_control.evidence, ...p.code_transparency.evidence],
      signers: null,
      sharedWith: [],
    });
    edge(nodeId, "node-protocol", p.role === "core" ? "core program" : "periphery program");
  }

  // Admin accounts, one node per account.
  const accounts = new Map<string, { fields: string[]; keys: string[]; evidence: Evidence[] }>();
  for (const [key, a] of Object.entries(facts.admin)) {
    const acc = accounts.get(a.account) ?? { fields: [], keys: [], evidence: [] };
    acc.fields.push(`${a.field} (${a.power})`);
    acc.keys.push(key);
    acc.evidence.push(...a.controller.evidence.filter((e) => e.address === a.account));
    accounts.set(a.account, acc);
  }
  for (const [account, acc] of [...accounts].sort(([a], [b]) => a.localeCompare(b))) {
    const rated = score ? acc.keys.map((k) => score.admin[k]).filter((r) => r !== undefined) : [];
    nodes.push({
      id: `node-account-${account}`,
      kind: "account",
      title: names.get(account) ?? `Account ${shortAddress(account)}`,
      address: account,
      rating: score ? (worstRating(rated.map((r) => r.rating)) ?? score.factors.admin_powers.rating) : null,
      ratingReason: score ? (rated.map((r) => r.reason).join("; ") || score.factors.admin_powers.reason) : "Unassessed: facts only.",
      facts: [{ label: "Admin fields", value: acc.fields.join(", ") }],
      evidence: acc.evidence,
      signers: null,
      sharedWith: [],
    });
    edge(`node-account-${account}`, "node-protocol", "admin account");
  }

  // Tokens the protocol uses.
  for (const [mint, roles] of Object.entries(facts.tokens).sort(([a], [b]) => a.localeCompare(b))) {
    const t = s.facts.tokens[mint];
    const exposure = score?.tokens[mint];
    nodes.push({
      id: `node-token-${mint}`,
      kind: "token",
      title: t?.symbol ?? exposure?.symbol ?? shortAddress(mint),
      address: mint,
      rating: score ? (exposure?.worst ?? "unknown") : null,
      ratingReason: exposure?.warning ?? `The weakest of the token's own ratings.`,
      facts: [
        { label: "Used as", value: roles.map((r) => ROLE_LABELS[r] ?? r).join(", ") },
        { label: "Token program", value: t ? (t.program === "token2022" ? "Token-2022" : "SPL Token") : "Not read" },
      ],
      evidence: t ? [...t.mint_authority.evidence, ...t.freeze_authority.evidence] : [],
      signers: null,
      sharedWith: [],
    });
    edge(`node-token-${mint}`, "node-protocol", `used as ${roles.map((r) => ROLE_LABELS[r] ?? r).join(", ")}`);
  }

  // Controllers.
  const subjectId = (g: Grant) =>
    g.ref.kind === "program"
      ? `node-program-${g.ref.address}`
      : g.ref.kind === "admin"
        ? `node-account-${g.subject}`
        : g.ref.kind === "token"
          ? `node-token-${g.ref.mint}`
          : actorNodeId(g.ref.address);
  for (const a of actors) {
    const c = a.controller;
    const rating = score ? (actorRating(a) ?? "unknown") : null;
    const worstGrant = a.grants.find((g) => base(g) === rating);
    const delays = [...new Set(a.grants.map((g) => delayText(g.delaySecs)))];
    nodes.push({
      id: actorNodeId(a.id),
      kind: actorKind(a),
      title: a.label,
      address: a.address,
      rating,
      ratingReason: !rating
        ? "Unassessed: facts only."
        : worstGrant
          ? `${VERBS[worstGrant.power]} ${worstGrant.subjectLabel}, rated ${rating.replace("_", " ")}.`
          : "The weakest rating among what it controls.",
      facts: [
        { label: "Controls", value: [...new Set(a.grants.map((g) => `${VERBS[g.power]} ${g.subjectLabel}`))].join("; ") },
        { label: "Delay", value: delays.join(", ") },
        ...(c.kind === "multisig" || c.kind === "cannot_act" ? [{ label: "Threshold", value: `${c.threshold} of ${c.voters}` }] : []),
        ...(c.kind === "timelock"
          ? [
              { label: "Minimum delay", value: delayText(c.min_delay_secs) },
              { label: "Outside admin", value: c.admin ? describeController(c.admin) : "None found" },
            ]
          : []),
        ...(c.kind === "unrecognized" && c.owner_program ? [{ label: "Owner program", value: c.owner_program }] : []),
        ...(c.kind === "pda" ? [{ label: "Program-derived address", value: c.owner_program ? `no private key; account owned by ${c.owner_program}; controlling program not resolved` : "no private key; no account; controlling program not resolved" }] : []),
        ...(c.kind === "program_controlled" ? [{ label: "Controlling program", value: c.program }] : []),
        ...(c.kind === "unavailable" ? [{ label: "Why unreadable", value: c.reason }] : []),
      ],
      evidence: a.grants.flatMap((g) => g.evidence).filter((e, i, all) => all.findIndex((x) => x.address === e.address && x.field === e.field) === i),
      signers: c.kind === "multisig" ? { count: c.voters, keys: c.signers ?? [] } : c.kind === "cannot_act" ? { count: c.voters, keys: [] } : null,
      sharedWith: (shared.get(a.id)?.protocols ?? []).filter((p) => p.id !== id).map(({ id: pid, name }) => ({ id: pid, name })),
    });
    for (const g of a.grants) edge(actorNodeId(a.id), subjectId(g), `${VERBS[g.power]} · ${delayText(g.delaySecs)}`);
  }

  // Columns: the protocol rightmost, its parts left of it, controllers left
  // of those, and whoever can change a multisig left of the multisig.
  const col = new Map<string, number>();
  const actorCol = (nodeId: string, seen = new Set<string>()): number => {
    if (col.has(nodeId)) return col.get(nodeId)!;
    if (seen.has(nodeId)) return 0;
    seen.add(nodeId);
    const targets = [...edges.values()].filter((e) => e.source === nodeId).map((e) => e.target);
    const c = Math.min(...targets.map((t) => (t.startsWith("node-program-") || t.startsWith("node-account-") || t.startsWith("node-token-") ? 2 : actorCol(t, seen))), 2) - 1;
    col.set(nodeId, c);
    return c;
  };
  for (const n of nodes) {
    if (n.kind === "protocol") col.set(n.id, 3);
    // A program-controlled authority is an actor, not a protocol program node.
    else if (/^node-(program|account|token)-/.test(n.id)) col.set(n.id, 2);
  }
  for (const n of nodes) actorCol(n.id);
  const min = Math.min(...col.values());
  const rows = new Map<number, number>();
  const placed: GraphNode[] = nodes.map((n) => {
    const column = col.get(n.id)! - min;
    const row = rows.get(column) ?? 0;
    rows.set(column, row + 1);
    return { ...n, column, row };
  });

  return {
    nodes: placed,
    edges: [...edges.values()].map((e) => ({ id: `${e.source}->${e.target}`, source: e.source, target: e.target, label: e.labels.join("; ") })),
  };
}
