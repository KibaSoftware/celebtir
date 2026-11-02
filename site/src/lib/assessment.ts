// F5, H2: the one publication rule every page reads. The pipeline writes
// `history/coverage.json` (which protocols the shipped methodology rates);
// the loader applies it once, so no view decides on its own which protocols
// carry a Stage or ratings.

import type { Assessment, CoverageFile, Snapshot } from "./types";

/** A reviewer named after a draft loop is not a human review (H2). */
export function isDraftReviewer(name: string | null | undefined): boolean {
  return !name || /draft/i.test(name);
}

/** Scores of unassessed protocols and draft reviewers removed. */
export function publishedSnapshot(s: Snapshot): Snapshot {
  const assessed = (id: string) => s.coverage.protocols[id]?.assessment === "assessed";
  const protocols = Object.fromEntries(Object.entries(s.scores.protocols).filter(([id]) => assessed(id)));
  const facts = Object.fromEntries(
    Object.entries(s.facts.protocols).map(([id, p]) => [id, isDraftReviewer(p.reviewed_by) ? { ...p, reviewed_by: null } : p]),
  );
  return { ...s, scores: { ...s.scores, protocols }, facts: { ...s.facts, protocols: facts } };
}

export function assessmentOf(s: Snapshot, id: string): Assessment {
  return s.coverage.protocols[id]?.assessment === "assessed" && id in s.scores.protocols ? "assessed" : "unassessed";
}

export function isAssessed(s: Snapshot, id: string): boolean {
  return assessmentOf(s, id) === "assessed";
}

/** Why a protocol carries no ratings, in the pipeline's words. */
export function unassessedReason(s: Snapshot, id: string): string {
  return s.coverage.protocols[id]?.reason ?? "Not rated by this methodology version; facts are listed unrated.";
}

/** Every protocol in the facts (assessed or not), sorted by id. */
export function allProtocolIds(s: Snapshot): string[] {
  return Object.keys(s.facts.protocols).sort();
}

export function assessedIds(s: Snapshot): string[] {
  return allProtocolIds(s).filter(id => isAssessed(s, id));
}

/** The chains the methodology rates, e.g. ["solana"]. */
export function assessedChains(c: CoverageFile): string[] {
  return c.assessed_chains;
}
