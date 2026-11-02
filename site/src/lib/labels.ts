// Display labels and chart colors shared by every page.

import type { ControlClass, Factor, Stage } from "./types";

export const FACTOR_LABELS: Record<Factor, string> = {
  upgrade_control: "Upgrade control",
  upgrade_delay: "Upgrade delay",
  admin_powers: "Admin powers",
  code_transparency: "Code transparency",
  oracle_dependency: "Oracle dependency",
  dependencies: "Dependencies",
};

/** Two-letter codes for compact factor strips; the full name is always one focus away. */
export const FACTOR_SHORT: Record<Factor, string> = {
  upgrade_control: "UC",
  upgrade_delay: "UD",
  admin_powers: "AP",
  code_transparency: "CT",
  oracle_dependency: "OD",
  dependencies: "DP",
};

export const STAGE_LABELS: Record<Stage, string> = {
  unrated: "Unrated",
  stage_0: "Stage 0",
  stage_1: "Stage 1",
  stage_2: "Stage 2",
};

export const CONTROL_LABELS: Record<ControlClass, string> = {
  single_key: "Single key",
  unrecognized: "Unrecognized",
  multisig_without_delay: "Multisig, no delay",
  multisig_short_delay: "Multisig, delay under 24 h",
  multisig_with_delay: "Multisig, delay 24 h+",
  immutable: "Immutable",
};

/** What each value class means, as written in the methodology (D4). */
export const CONTROL_DEFINITIONS: Record<ControlClass, string> = {
  single_key: "A single key can upgrade the weakest core program.",
  unrecognized: "The controller is unreadable, unsupported, or a program-derived address whose controlling program is not resolved.",
  multisig_without_delay: "A multisig with an effective delay of zero: no time lock, or a config authority that bypasses it.",
  multisig_short_delay: "A multisig or timelock whose delay is above zero but below 24 hours.",
  multisig_with_delay: "A multisig or timelock that waits at least 24 hours, the methodology's Medium upgrade-delay bar.",
  immutable: "The code cannot change.",
};

/**
 * Ordinal ramps (ADR 016, retuned for the Highwatch navy in ADR 023). Control
 * classes: one Celestial Blue ramp from the weakest control (lightest, most
 * salient on navy) to immutable. Stages: a desaturated lavender ramp, used
 * only for Stage marks, never as an accent; Unrated in neutral gray. Neither
 * ramp reuses a status color, so Stage 0 never reads as High.
 */
export const CONTROL_COLORS: Record<ControlClass, string> = {
  single_key: "#D4B46A",
  unrecognized: "#98A5B5",
  multisig_without_delay: "#6599E8",
  multisig_short_delay: "#94BAF3",
  multisig_with_delay: "#7BC6C1",
  immutable: "#C5CFDD",
};

export const STAGE_COLORS: Record<Stage, string> = {
  unrated: "#8992A3",
  stage_0: "#D9D3F3",
  stage_1: "#B3A8E6",
  stage_2: "#8F80D4",
};

/** The single-series line color: Celestial Blue. */
export const SERIES_COLOR = "#6599E8";

export { STAGE_DEFINITION as STAGE_NOTE } from './scoreReview';

/**
 * Categorical series (asset groups): the Highwatch series order, Celestial
 * Blue, Highwatch Gold, Moon Silver, pale blue. Unordered, so not a ramp.
 */
export const SERIES_COLORS = ["#6599E8", "#D4B46A", "#C5CFDD", "#94BAF3"] as const;
