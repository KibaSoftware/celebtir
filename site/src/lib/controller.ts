// Controllers in words, matching the scorer's `short_describe` so the site and
// the published sentences never disagree.

import { formatDuration } from "./format";
import type { Controller } from "./types";

export function describeController(c: Controller): string {
  switch (c.kind) {
    case "program_controlled":
      return `Program controlled: ${describeController(c.upgrade)}`;
    case "immutable":
      return "Immutable";
    case "single_key":
      return "Single key";
    case "multisig": {
      const name = c.multisig_kind === "safe" ? "Safe" : "Squads";
      const tail =
        c.config_authority?.kind === "single_key"
          ? "config key"
          : c.time_lock_secs === 0
            ? "no delay"
            : `${formatDuration(c.time_lock_secs)} delay`;
      return `${name} ${c.threshold} of ${c.voters} · ${tail}`;
    }
    case "timelock":
      return c.admin
        ? `Timelock · ${formatDuration(c.min_delay_secs)} delay · outside admin`
        : `Timelock · ${formatDuration(c.min_delay_secs)} delay`;
    case "cannot_act":
      return "Multisig that cannot act";
    case "unrecognized":
      return "Unrecognized";
    case "pda":
      return "Program-derived address, controlling program not resolved";
    case "unavailable":
      return "Unreadable";
  }
}

/** A multisig's time lock in words: Squads v3 has no time-lock field (F12). */
export function timeLockText(c: Extract<Controller, { kind: "multisig" }>): string {
  if (c.multisig_kind === "squads_v3") return "No time lock (Squads v3)";
  return c.time_lock_secs === 0 ? "No time lock" : formatDuration(c.time_lock_secs);
}

/**
 * The effective delay in words for a controller panel: an immutable program
 * has no delay because nothing can change (F10), an unresolved
 * program-derived address has an unknown one (F2).
 */
export function effectiveDelayText(c: Controller): string {
  if (c.kind === "immutable") return "n/a (immutable)";
  if (c.kind === "pda") return "unknown (program-derived address not resolved)";
  return delayText(delayOf(c));
}

/** The address that identifies a controller, if it has one. */
export function controllerAddress(c: Controller): string | null {
  switch (c.kind) {
    case "single_key":
    case "unrecognized":
    case "pda":
    case "program_controlled":
      return c.key;
    case "multisig":
    case "timelock":
    case "cannot_act":
      return c.address;
    default:
      return null;
  }
}

/**
 * Seconds between a decision and its effect; null when unknown. A config
 * authority can change the multisig itself, so the delay is zero (ADR 005).
 */
export function delayOf(c: Controller): number | null {
  switch (c.kind) {
    case "program_controlled": return delayOf(c.upgrade);
    case "multisig":
      return c.config_authority ? 0 : c.time_lock_secs;
    case "timelock":
      // An outside admin can change the delay without waiting (ADR 024 D4).
      return c.admin ? 0 : c.min_delay_secs;
    case "single_key":
    case "unrecognized":
      return 0;
    default:
      return null;
  }
}

export function delayText(secs: number | null): string {
  if (secs === null) return "unknown delay";
  return secs === 0 ? "no delay" : `${formatDuration(secs)} delay`;
}

const LOADERS: Record<string, string> = {
  BPFLoaderUpgradeab1e11111111111111111111111: "Upgradeable loader",
  BPFLoader1111111111111111111111111111111111: "Loader v1 (cannot upgrade)",
  BPFLoader2111111111111111111111111111111111: "Loader v2 (cannot upgrade)",
  LoaderV411111111111111111111111111111111111: "Loader v4",
  NativeLoader1111111111111111111111111111111: "Native loader",
};

export function loaderName(loader: string | null): string | null {
  return loader === null ? null : (LOADERS[loader] ?? loader);
}
