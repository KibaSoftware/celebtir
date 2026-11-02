import { expect, it } from "vitest";
import { authorityLabel, authorityWording, nextStageCounts, readableFinding } from "./presentation";
import type { StageInspection } from "./scoreReview";

it("does not equate one authority address with one human signer", () => {
  const stored = {kind: "single_key" as const, key: "authority"};
  expect(authorityLabel(stored)).toBe("Recorded signing authority");
  expect(stored.kind).toBe("single_key");
  expect(authorityWording("A single key can mint tokens.")).toBe("A recorded signing authority can mint tokens.");
});
it("keeps the action and qualifications when removing identifiers from the reading layer", () => {
  const raw = "`lending_market_owner` of 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF can change parameters, time lock 12 h";
  expect(readableFinding(raw)).toBe("lending market owner of the recorded account can change parameters, time lock 12 h");
  expect(raw).toContain("7u3HeHxY");
});
it("counts existing next-stage annotations without reclassifying unresolved or other-stage requirements", () => {
  const inspection = {stage:{next:"stage_1"},criteria:[
    {requirement:{stage:"stage_1"},status:"MET"},
    {requirement:{stage:"stage_1"},status:"UNMET_VERIFIED"},
    {requirement:{stage:"stage_1"},status:"UNRESOLVED_EVIDENCE"},
    {requirement:{stage:"stage_2"},status:"MET"},
  ]} as StageInspection;
  expect(nextStageCounts(inspection)).toEqual({met:1,unmet:1,unresolved:1});
  expect(nextStageCounts()).toBeNull();
});
