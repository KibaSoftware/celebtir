import type { Metadata } from "next";
import { Address, EvidenceList } from "@/components/Evidence";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { PageHeader } from "@/components/ui/Ui";
import { unassessedReason } from "@/lib/assessment";
import { shortAddress } from "@/lib/format";
import { loadSnapshot } from "@/lib/data";
import { describeController, effectiveDelayText } from "@/lib/controller";
import type { Evidence } from "@/lib/types";
import styles from "@/components/research/Research.module.css";
import { ControllerAuthorityTree } from "@/components/research/ControllerAuthorityTree";

// H4: this page's own description, not the site's.
export const metadata: Metadata = {
  title: "HyperEVM contract authorities",
  description: "HyperEVM protocols listed unassessed: who can upgrade each contract (proxy admin, owner, timelock delay and proposer, Safe), with pinned-block evidence. No ratings or Stage.",
};

/** The upgrade path as read, in order: each read names the contract and the field (F10). */
function readPath(evidence: Evidence[]): string {
  return evidence
    .map(e => {
      const short = `${e.address.slice(0, 6)}…${e.address.slice(-4)}`;
      const field = e.field
        .replace("eip1967.admin", "EIP-1967 admin slot")
        .replace("uups.owner()", "UUPS owner()")
        .replace("code (empty: a key)", "no code: a key")
        .replace("proxy: none (registry)", "not a proxy (registry)");
      return `${field} of ${short}`;
    })
    .join(" → ");
}

export default function HyperEvmPage() {
  const s = loadSnapshot();
  const protocols = Object.entries(s.facts.protocols).filter(([,p]) => p.chain === "hyper_evm");
  return <>
    <PageHeader art="hyperevm" title="HyperEVM" eyebrow="Contract authority research" subtitle="Contract identities, effective upgrade controllers, privileged roles and pinned-block evidence." />
    <section className={`${styles.panel} ${styles.bastion}`}>
      <h2>{protocols.length ? `${protocols.length} protocols with recorded contract facts` : "Coverage required"}</h2>
      <p>{protocols.length ? `Unassessed: ${protocols[0] ? unassessedReason(s, protocols[0][0]) : ""}. Native contract observations are listed below; unavailable fields stay unavailable.` : "Unassessed: this snapshot contains no published HyperEVM contract observations. This is a coverage gap, not evidence of zero protocols or Low risk."}</p>
      <p className={styles.note}>Supported readers: EIP-1967 proxies and ProxyAdmin owners, UUPS owners, role-registry upgrade guards, Safe multisigs and modules, and OpenZeppelin TimelockControllers (delay read with getMinDelay; proposers confirmed with hasRole when the registry names them). Solana program Stages are not assigned by this view.</p>
      <Link href="/methodology/#hyperliquid">Domain scope and evidence requirements</Link>
    </section>
    <div className={styles.grid}>{[ ["Proxy and upgrade authority", "Each contract's upgrade path is shown as it was read: the proxy's admin slot or owner, then each controller in turn, down to a key, a Safe or a timelock with its delay."], ["Ownership and roles", "Recorded owner and AccessControl fields identify parameter, pause and asset-movement authorities separately."], ["Code verification", "Published verification state includes repository, commit and bytecode hashes where available."], ["Oracles and dependencies", "Feed classification is distinct from external-call coverage. Missing dependency tracing remains unavailable."] ].map(([title,text]) => <section className={styles.panel} key={title}><h2>{title}</h2><p>{text}</p></section>)}</div>
    {protocols.map(([id,p]) => <section className={styles.panel} key={id}>
      <h2>{p.name ?? id}</h2><p>HyperEVM block {p.min_slot.toLocaleString("en-US")}–{p.max_slot.toLocaleString("en-US")}</p>
      {Object.entries(p.programs).map(([address,c]) => <details className={styles.record} key={address}><summary>{shortAddress(address)} · {c.role} contract · {describeController(c.upgrade_control.value)}</summary><p>Contract: <Address address={address} href={`https://hyperevmscan.io/address/${address}`} /></p>
        <p>Upgrade path read: {readPath(c.upgrade_control.evidence)}</p>
        <p>Effective upgrade delay: {effectiveDelayText(c.upgrade_control.value)}</p>
        <ControllerAuthorityTree controller={c.upgrade_control.value} /><EvidenceList evidence={c.upgrade_control.evidence} />
        <p>Code verification: {c.code_transparency.value.kind.replaceAll("_", " ")}</p><EvidenceList evidence={c.code_transparency.evidence} />
      </details>)}
      {Object.entries(p.admin).map(([key,a]) => <details className={styles.record} key={key}><summary>{a.power} · {a.field} on {shortAddress(a.account)}</summary><p>Account: <Address address={a.account} /></p><p>{describeController(a.controller.value)}</p><ControllerAuthorityTree controller={a.controller.value} /><EvidenceList evidence={a.controller.evidence} /></details>)}
      {Object.entries(p.oracles).map(([key,o]) => <details className={styles.record} key={key}><summary>Oracle · {o.market} · {o.role} · {shortAddress(key)}</summary><p>Oracle: <Address address={key} /></p><p>{o.source.value.kind.replaceAll("_", " ")}</p><EvidenceList evidence={o.source.evidence} /></details>)}
      <Link href={`/protocols/${id}/#control-graph`}>Inspect full controller paths and evidence</Link>
    </section>)}
  </>;
}
