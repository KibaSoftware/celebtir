import { describeController, effectiveDelayText } from "./controller";
import type { Controller, Factor, FactorScore } from "./types";
import type { ProtocolPage } from "./protocol";
import type { StageInspection } from "./scoreReview";
import { POWER_LABELS } from "./permissions";

/** Display wording only. The serialized controller and scored reason remain unchanged. */
export function authorityLabel(controller: Controller): string {
  if (controller.kind === "single_key") return "Recorded signing authority";
  if (controller.kind === "program_controlled") return `Program controlled: ${authorityLabel(controller.upgrade)}`;
  return describeController(controller);
}

export const SIGNER_BOUNDARY = "An authority address does not establish one human signer or an off-chain approval process. Effective signer configuration is not established from the available on-chain evidence.";

export function authorityWording(text: string): string {
  return text.replace(/\bsingle key\b/gi, "recorded signing authority");
}

/** Remove identifiers from the reading layer, never from the underlying evidence. */
export function readableFinding(text: string): string {
  return authorityWording(text)
    .replace(/https:\/\/github\.com\/([^/\s]+\/[^/\s]+)(?:\/[^\s;,]*)?/g, "$1")
    .replace(/\b(?:0x[a-fA-F0-9]{40}|[1-9A-HJ-NP-Za-km-z]{32,44})\b/g, "the recorded account")
    .replace(/recorded signing authority the recorded account/g, "the recorded signing authority")
    .replace(/\bat [a-f0-9]{40}\b/g, "at the recorded revision")
    .replace(/`([^`]+)`/g, (_, field: string) => field.replaceAll("_", " "));
}

export function nextStageCounts(inspection?: StageInspection) {
  if (!inspection?.stage.next) return null;
  const criteria = inspection.criteria.filter(c => c.requirement.stage === inspection.stage.next);
  if (!criteria.length) return null;
  return {
    met: criteria.filter(c => c.status === "MET").length,
    unmet: criteria.filter(c => c.status === "UNMET_VERIFIED").length,
    unresolved: criteria.filter(c => c.status === "UNRESOLVED_EVIDENCE").length,
  };
}

export const FACTOR_QUESTIONS: Record<Factor, string> = {
  upgrade_control: "Who can replace the code?",
  upgrade_delay: "How much notice does an upgrade require?",
  admin_powers: "What can administrators change?",
  oracle_dependency: "Who controls the price inputs?",
  code_transparency: "Can the deployed code be checked?",
  dependencies: "Which other programs are relied on?",
};

export function protocolFinding(page: ProtocolPage, factor: Factor): string {
  const core = page.programs.filter(p => p.role === "core");
  if (page.factors?.[factor].rating === "unknown") return "Available evidence does not establish this factor. Inspect the unresolved observations.";
  if (factor === "upgrade_control" && core.length) return [...new Set(core.map(p => authorityLabel(p.controller.value)))].join("; ");
  if (factor === "upgrade_delay" && core.length) return [...new Set(core.map(p => effectiveDelayText(p.controller.value)))].join("; ");
  if (factor === "code_transparency" && core.length) {
    const labels = { verified: "verified build", public_source_unverified: "public source, build not verified", no_public_source: "no public source", unavailable: "verification unavailable" };
    return [...new Set(core.map(p => labels[p.transparency.value.kind]))].join("; ");
  }
  if (factor === "admin_powers") {
    const grants = page.actors.flatMap(a => a.grants).filter(g => g.ref.kind === "admin");
    if (grants.length) return `Recorded administrative powers: ${[...new Set(grants.map(g => POWER_LABELS[g.power].toLowerCase()))].join(", ")}.`;
  }
  if (factor === "oracle_dependency" && page.oracles.length) {
    const sources = [...new Set(page.oracles.map(o => o.source.kind === "network" ? o.source.name : o.source.kind === "protocol_run" ? "protocol-run price feeds" : o.source.kind === "unavailable" ? "unreadable price sources" : "other recorded price sources"))];
    return `${page.oracles.length} registered feed${page.oracles.length === 1 ? "" : "s"}: ${sources.join(", ")}.`;
  }
  if (factor === "dependencies" && page.dependencies.value.kind !== "unavailable") {
    const deps = page.dependencies.value;
    return `${Object.keys(deps.callees).length} external programs recorded in the transaction sample.${deps.stale ? " Carried over from an earlier sample; current coverage is incomplete." : " Sampling does not establish every possible dependency."}`;
  }
  const score = page.factors?.[factor];
  return score ? readableFinding(score.reason) : "No published factor assessment.";
}

/** Group labels describe the existing observation; they never replace a rating. */
export function findingBoundary(score: FactorScore): string {
  if (score.rating === "unknown") return "Unresolved observation";
  if (score.rating === "not_applicable") return "Not applicable to this assessment";
  return "Published finding";
}
