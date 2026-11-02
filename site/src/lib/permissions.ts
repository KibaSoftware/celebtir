// UI-13: who can do what, grouped by actor. Built from the facts at build
// time; the Permissions section and the control graph share the actor ids.

import { controllerAddress, delayOf, describeController } from "./controller";
import { shortAddress } from "./format";
import type { Controller, Evidence, ProtocolFacts, TokenFacts } from "./types";

export type Power =
  | "upgrade"
  | "parameters"
  | "pause"
  | "funds"
  | "freeze"
  | "mint"
  | "reconfigure"
  | "sign"
  | "propose";

export const POWER_LABELS: Record<Power, string> = {
  upgrade: "Upgrade the code",
  parameters: "Change parameters",
  pause: "Pause",
  funds: "Move funds",
  freeze: "Freeze tokens",
  mint: "Mint tokens",
  reconfigure: "Change the multisig",
  sign: "Sign for the multisig",
  propose: "Propose through the timelock",
};

/** Where a grant's rating is published in the scores (UI-21). */
export type GrantRef =
  | { kind: "program"; address: string }
  | { kind: "admin"; key: string }
  | { kind: "token"; mint: string; factor: "supply_control" | "freeze_and_seize" | "transfer_restrictions" }
  | { kind: "multisig"; address: string };

export interface Grant {
  power: Power;
  ref: GrantRef;
  /** Address of the program, account, mint or multisig the power acts on. */
  subject: string;
  subjectLabel: string;
  /** Null when the delay is unknown. */
  delaySecs: number | null;
  evidence: Evidence[];
}

export interface Actor {
  /** The actor's address, or `unreadable` for controllers that could not be read. */
  id: string;
  kind: Controller["kind"];
  label: string;
  address: string | null;
  /** The controller as first seen, for its details in the graph. */
  controller: Controller;
  grants: Grant[];
}

const ORDER: Record<Controller["kind"], number> = {
  program_controlled: 3,
  single_key: 0,
  unrecognized: 1,
  pda: 1,
  unavailable: 2,
  multisig: 3,
  timelock: 4,
  cannot_act: 5,
  immutable: 6,
};

export const UNREADABLE = "unreadable";

export function actorsFor(
  facts: ProtocolFacts,
  tokens: Record<string, TokenFacts>,
): { actors: Actor[]; renounced: Grant[] } {
  const actors = new Map<string, Actor>();
  const renounced: Grant[] = [];
  const names = new Map(facts.sources.map((s) => [s.address, s.label]));

  const add = (c: Controller, power: Power, ref: GrantRef, subject: string, subjectLabel: string, evidence: Evidence[]) => {
    const grant: Grant = { power, ref, subject, subjectLabel, delaySecs: delayOf(c), evidence };
    if (c.kind === "immutable") {
      renounced.push(grant);
      return;
    }
    const address = controllerAddress(c);
    const id = address ?? UNREADABLE;
    const actor = actors.get(id) ?? {
      id,
      kind: c.kind,
      label: address ? describeController(c) : "Unreadable controller",
      address,
      controller: c,
      grants: [],
    };
    actor.grants.push(grant);
    actors.set(id, actor);
    if (c.kind === "program_controlled") {
      add(c.upgrade, "upgrade", { kind: "multisig", address: c.key }, c.program, `program ${shortAddress(c.program)}`, evidence);
    }
    // Edge case 5: a signer that is itself a multisig (or an unknown account).
    if (c.kind === "multisig") {
      for (const inner of Object.values(c.nested_signers ?? {})) {
        add(inner, "sign", { kind: "multisig", address: c.address }, c.address, `multisig ${shortAddress(c.address)}`, evidence);
      }
    }
    if (c.kind === "multisig" && c.config_authority) {
      add(c.config_authority, "reconfigure", { kind: "multisig", address: c.address }, c.address, `multisig ${shortAddress(c.address)}`, evidence);
    }
    // HUI-4: the proposer sits behind the timelock; an outside admin can
    // rewrite its roles and delay.
    if (c.kind === "timelock") {
      add(c.proposer, "propose", { kind: "multisig", address: c.address }, c.address, `timelock ${shortAddress(c.address)}`, evidence);
      if (c.admin) {
        add(c.admin, "reconfigure", { kind: "multisig", address: c.address }, c.address, `timelock ${shortAddress(c.address)}`, evidence);
      }
    }
  };

  for (const [address, p] of Object.entries(facts.programs)) {
    add(p.upgrade_control.value, "upgrade", { kind: "program", address }, address, names.get(address) ?? shortAddress(address), p.upgrade_control.evidence);
  }
  for (const [key, a] of Object.entries(facts.admin)) {
    add(a.controller.value, a.power, { kind: "admin", key }, a.account, `${a.field} on ${shortAddress(a.account)}`, a.controller.evidence);
  }
  for (const mint of Object.keys(facts.tokens)) {
    const t = tokens[mint];
    if (!t) continue;
    const ext = t.extensions.value;
    const token = (factor: "supply_control" | "freeze_and_seize" | "transfer_restrictions") => ({ kind: "token" as const, mint, factor });
    if (ext.pause_authority) add(ext.pause_authority, "pause", token("transfer_restrictions"), mint, t.identity?.issuer ? `${t.symbol} (${t.identity.issuer})` : t.symbol, t.extensions.evidence);
    if (ext.permanent_delegate) add(ext.permanent_delegate, "funds", token("freeze_and_seize"), mint, t.identity?.issuer ? `${t.symbol} (${t.identity.issuer})` : t.symbol, t.extensions.evidence);
    add(t.freeze_authority.value, "freeze", token("freeze_and_seize"), mint, t.identity?.issuer ? `${t.symbol} (${t.identity.issuer})` : t.symbol, t.freeze_authority.evidence);
    add(t.mint_authority.value, "mint", token("supply_control"), mint, t.identity?.issuer ? `${t.symbol} (${t.identity.issuer})` : t.symbol, t.mint_authority.evidence);
  }

  const sorted = [...actors.values()].sort((a, b) => ORDER[a.kind] - ORDER[b.kind] || a.id.localeCompare(b.id));
  return { actors: sorted, renounced };
}
