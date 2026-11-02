// Test helpers: expectations derived from whichever snapshot the tests load
// (the live `data/latest/` or the demo set), so tests check behaviour, not
// one day's protocol list.

import type { Snapshot } from "./types";

export const JITOSOL = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

/** Every scored protocol id, sorted. */
export function protocolIds(s: Snapshot): string[] {
  return Object.keys(s.scores.protocols).sort();
}

/** Protocols whose registry lists `mint`, sorted. */
export function usersOf(s: Snapshot, mint: string): string[] {
  return Object.entries(s.facts.protocols)
    .filter(([, p]) => mint in p.tokens)
    .map(([id]) => id)
    .sort();
}

export function nameOf(s: Snapshot, id: string): string {
  return s.facts.protocols[id]?.name ?? id;
}

/**
 * The protocol the two-protocol tests compare with Kamino Lend: the first
 * other protocol that also uses JitoSOL (the demo set's SPL Stake Pool).
 */
export function peerOfKamino(s: Snapshot): string {
  const peer = usersOf(s, JITOSOL).find((id) => id !== "kamino-lend");
  if (!peer) throw new Error("no protocol besides kamino-lend uses JitoSOL");
  return peer;
}
