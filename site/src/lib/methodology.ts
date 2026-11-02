// The methodology page's numbers, read from `methodology/v<version>.toml` at
// build time, so the page can never disagree with the scorer (SCO-2).

import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { parse } from "smol-toml";
import { formatDuration } from "./format";
import type { Snapshot, Stage } from "./types";

export interface Methodology {
  version: string;
  upgrade_control: { low_min_voters: number; low_min_threshold_pct: number; medium_min_voters: number; medium_min_threshold_pct: number };
  upgrade_delay: { low_min_secs: number; medium_min_secs: number };
  admin_powers: { low_min_time_lock_secs: number };
  oracle_dependency: { established_networks: string[]; guarded_primary_min_networks?: number };
  stage1: { min_upgrade_delay_secs: number; emergency_min_voters: number; emergency_min_threshold_pct: number };
  stage2: { governance_min_delay_secs: number };
  /** v0.3+ (D4). */
  value_classes?: { delayed_min_secs: number };
  /** v0.3+ (F5). */
  coverage?: { assessed_chains: string[] };
}

const dir = path.resolve(process.cwd(), "..", "methodology");

export function loadMethodology(version: string): Methodology {
  return parse(readFileSync(path.join(dir, `v${version}.toml`), "utf8")) as unknown as Methodology;
}

/** Every published version, oldest first. */
export function methodologyVersions(): string[] {
  return readdirSync(dir)
    .map((f) => /^v(\d+\.\d+)\.toml$/.exec(f)?.[1])
    .filter((v): v is string => v !== undefined)
    .sort((a, b) => a.localeCompare(b, "en", { numeric: true }));
}

export function methodologyChangelog(): string {
  return readFileSync(path.join(dir, "CHANGELOG.md"), "utf8");
}

export interface FactorRule {
  factor: string;
  question: string;
  low: string;
  medium: string;
  high: string;
}

const days = (s: number) => formatDuration(s);

/** The PRD's factor table with the thresholds of `m` written in. */
export function factorRules(m: Methodology): FactorRule[] {
  const uc = m.upgrade_control;
  const ud = m.upgrade_delay;
  const networks = m.oracle_dependency.established_networks.map((n) => n[0]!.toUpperCase() + n.slice(1)).join(", ");
  // v0.3 wrote these rules down; earlier versions keep their published text.
  const v03 = m.value_classes !== undefined;
  const guards = m.oracle_dependency.guarded_primary_min_networks;
  return [
    {
      factor: "Upgrade control",
      question: "Who can replace the code?",
      low: `Immutable, or a multisig of ${uc.low_min_voters}+ signers at a ${uc.low_min_threshold_pct}%+ threshold`,
      medium: `A multisig of ${uc.medium_min_voters}+ signers at a ${uc.medium_min_threshold_pct}%+ threshold`,
      high: "A single key, a smaller multisig, a multisig whose config authority is one key, or a controller Celebtir does not recognize" + (v03 ? ". An off-curve program-derived address whose controlling program is not resolved is Unknown, not High" : ""),
    },
    {
      factor: "Upgrade delay",
      question: "How long between approval and a new version going live?",
      low: `${days(ud.low_min_secs)} or more, or immutable`,
      medium: `${days(ud.medium_min_secs)} to ${days(ud.low_min_secs)}`,
      high: `Under ${days(ud.medium_min_secs)}, or none`,
    },
    {
      factor: "Admin powers",
      question: "Can a key change parameters, pause, or move funds outside an upgrade?",
      low: v03
        ? `Every admin field renounced, only parameter changes behind a time lock of ${days(m.admin_powers.low_min_time_lock_secs)} or more, or a sourced registry review that found no admin accounts (an empty list without a review is Unknown)`
        : `No admin key, or only parameter changes behind a time lock of ${days(m.admin_powers.low_min_time_lock_secs)} or more`,
      medium: "Parameter, pause or fund powers held by a multisig that meets the Medium upgrade-control bar",
      high: "One key, or a weaker multisig, holds an admin power",
    },
    {
      factor: "Code transparency",
      question: "Does the deployed binary match public source?",
      low: "A verified build that matches what is deployed",
      medium: "Source is public but the build is not verified",
      high: "No public source",
    },
    {
      factor: "Oracle dependency",
      question: "Where do prices come from?",
      low: v03 && guards
        ? `An established oracle network (${networks}) with a fallback source, or a primary outside them whose price must agree with ${guards}+ distinct established networks that are also registered as its fallback`
        : `An established oracle network (${networks}) with a fallback source`,
      medium: "A single established oracle network",
      high: "A protocol-run or single-signer feed, or an owner program Celebtir does not recognize",
    },
    {
      factor: "Dependencies",
      question: "Which other programs does it call into with user funds?",
      low: "Every called program rated Low on upgrade control",
      medium: "Any called program rated Medium",
      high: "Any called program rated High",
    },
  ];
}

/** Stage requirements as the scorer words them, from the published scores. */
export function stageRequirements(s: Snapshot): { stage: Stage; id: string; description: string }[] {
  const seen = new Map<string, { stage: Stage; id: string; description: string }>();
  for (const p of Object.values(s.scores.protocols)) {
    for (const r of p.stage.requirements) if (!seen.has(r.id)) seen.set(r.id, { stage: r.stage, id: r.id, description: r.description });
  }
  const order: Stage[] = ["unrated", "stage_0", "stage_1", "stage_2"];
  return [...seen.values()].sort((a, b) => order.indexOf(a.stage) - order.indexOf(b.stage) || a.id.localeCompare(b.id));
}
