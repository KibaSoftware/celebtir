import { Address } from "@/components/Evidence";
import { authorityLabel, SIGNER_BOUNDARY } from "@/lib/presentation";
import { controllerAddress } from "@/lib/controller";
import type { Controller } from "@/lib/types";

/** Only traverse recorded controller edges; never infer an owner from its label. */
export function ControllerAuthorityTree({ controller, depth = 0 }: {controller: Controller; depth?: number}) {
  if (depth > 8) return <p>Unresolved: authority path exceeds display depth.</p>;
  const address = controllerAddress(controller);
  return <div><p>{authorityLabel(controller)} {address ? <Address address={address} /> : null}</p>
    {controller.kind === "program_controlled" ? <details open><summary>Controlling program <Address address={controller.program} /></summary><ControllerAuthorityTree controller={controller.upgrade} depth={depth+1} /></details> : null}
    {controller.kind === "timelock" ? <><p>Every upgrade waits {controller.min_delay_secs.toLocaleString("en-US")} s ({(controller.min_delay_secs / 3600).toFixed(controller.min_delay_secs % 3600 ? 1 : 0)} h) after it is scheduled.</p><details open><summary>Proposer</summary><ControllerAuthorityTree controller={controller.proposer} depth={depth+1} /></details>{controller.admin ? <details><summary>Outside administrator / possible bypass</summary><ControllerAuthorityTree controller={controller.admin} depth={depth+1} /></details> : null}</> : null}
    {controller.kind === "multisig" ? <><p>{controller.threshold} approvals from {controller.voters} voting members. {controller.signers ? `${controller.signers.length} recorded signer identities.` : "Signer identities unavailable."}</p>{controller.config_authority ? <details open><summary>Configuration authority</summary><ControllerAuthorityTree controller={controller.config_authority} depth={depth+1} /></details> : null}{Object.entries(controller.nested_signers ?? {}).map(([key,c]) => <details key={key}><summary>Nested signer <Address address={key} /></summary><ControllerAuthorityTree controller={c} depth={depth+1} /></details>)}</> : null}
    {controller.kind === "single_key" ? <p>{SIGNER_BOUNDARY}</p> : null}
    {controller.kind === "unavailable" ? <p>{controller.reason}</p> : null}
    {controller.kind === "pda" ? <p>No private key exists for this address; the program that signs for it is not resolved by a verified registry declaration.</p> : null}
  </div>;
}
