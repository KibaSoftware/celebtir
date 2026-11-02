// Build-time data access. Pages are pre-rendered (UI-26), so this runs in
// `next build`, reading the pipeline's output from disk. Until the daily
// pipeline has published real data, the committed demo dataset is used and
// every page says so.

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import type { CoverageFile, RunFile, ScoresFile, FactsFile, Snapshot } from "./types";
import { publishedSnapshot } from "./assessment";

const repoRoot = path.resolve(process.cwd(), "..");

/** `CELEBTIR_DATA`, else `<repo>/data` if it has a snapshot, else the demo set. */
export function dataDir(): { dir: string; demo: boolean } {
  const configured = process.env.CELEBTIR_DATA;
  if (configured) return { dir: path.resolve(configured), demo: false };
  const real = path.join(repoRoot, "data");
  if (existsSync(path.join(real, "latest", "scores.json"))) return { dir: real, demo: false };
  return { dir: path.join(repoRoot, "fixtures", "site-data"), demo: true };
}

function readJson<T>(file: string): T {
  return JSON.parse(readFileSync(file, "utf8")) as T;
}

let cached: Snapshot | undefined;

export function loadSnapshot(): Snapshot {
  if (cached) return cached;
  const { dir, demo } = dataDir();
  const latest = path.join(dir, "latest");
  const coverage = loadHistory<CoverageFile>("coverage");
  if (!coverage) throw new Error(`${dir}/history/coverage.json is missing: rebuild with \`celebtir build-site-data\``);
  // One publication rule for every page (F5, H2): unassessed protocols keep
  // their facts but lose their scores; draft registry reviewers are withheld.
  cached = publishedSnapshot({
    snapshotDate: loadHistory<{days: {date: string}[]}>("summary")?.days.at(-1)?.date,
    facts: readJson<FactsFile>(path.join(latest, "facts.json")),
    scores: readJson<ScoresFile>(path.join(latest, "scores.json")),
    run: readJson<RunFile>(path.join(latest, "run.json")),
    coverage,
    demo,
  });
  return cached;
}

export function loadHistory<T>(name: string): T | undefined {
  const file = path.join(dataDir().dir, "history", `${name}.json`);
  return existsSync(file) ? readJson<T>(file) : undefined;
}

/**
 * Facts for the freshness bar (UI-2). The date is the newest snapshot's (its
 * folder date, the last day in history/summary.json); a replayed or backfilled
 * run can finish on a later day, so the run time is kept separately.
 */
export function freshness(s: Snapshot) {
  const days = loadHistory<{ days: { date: string }[] }>("summary")?.days ?? [];
  const range = solanaSlotRange(s);
  return {
    date: days[days.length - 1]?.date ?? s.run.finished_at.slice(0, 10),
    finishedAt: s.run.finished_at,
    /** The newest Solana read (kept for older callers); see `slots` (H3). */
    slot: range?.max ?? null,
    /** First and last finalized Solana slot read, protocols and tokens (H3). */
    slots: range,
    blocks: Object.values(s.facts.protocols).filter(p => p.chain === "hyper_evm").map(p => p.max_slot).reduce<number | null>((max, block) => max === null ? block : Math.max(max, block), null),
    methodology: s.scores.methodology_version,
    demo: s.demo,
  };
}

/**
 * H3: the range of finalized Solana slots the snapshot's reads span, across
 * protocols and token mints, so no evidence slot lies outside the header.
 */
export function solanaSlotRange(s: Snapshot): { min: number; max: number } | null {
  const slots: number[] = [];
  for (const p of Object.values(s.facts.protocols)) {
    if (p.chain && p.chain !== "solana") continue;
    if (p.min_slot > 0) slots.push(p.min_slot);
    if (p.max_slot > 0) slots.push(p.max_slot);
  }
  for (const t of Object.values(s.facts.tokens)) {
    for (const f of [t.mint_authority, t.freeze_authority, t.extensions, t.provenance]) {
      for (const e of f.evidence) if (e.slot > 0) slots.push(e.slot);
    }
  }
  if (!slots.length) return null;
  return { min: Math.min(...slots), max: Math.max(...slots) };
}
