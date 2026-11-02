import type { ReactNode } from "react";
import { LimitedItems } from "./LimitedItems";
import { CopyButton } from "./CopyButton";
import { shortAddress } from "@/lib/format";
import { accountUrl, txUrl } from "@/lib/solscan";
import type { Evidence } from "@/lib/types";
import styles from "./Evidence.module.css";

/** A part of a page with nothing to show says why (UI-12). */
export function Empty({ children }: { children: ReactNode }) {
  return (
    <p className={styles.empty} data-empty="">
      {children}
    </p>
  );
}

/**
 * UI-5: an address shortened, with its full value on hover, a Solscan link
 * and a copy button.
 */
export function Address({ address, href }: { address: string; href?: string }) {
  return (
    <span className={styles.addressWrap}>
      <a className={styles.address} href={href ?? accountUrl(address)} title={address}>
        {shortAddress(address)}
      </a>
      <CopyButton value={address} />
    </span>
  );
}

/** A transaction signature: shortened, linked to Solscan, copyable. */
export function TxLink({ signature, label }: { signature: string; label?: string }) {
  return (
    <span className={styles.addressWrap}>
      <a className={styles.address} href={txUrl(signature)} title={signature}>
        {label ?? `tx ${shortAddress(signature)}`}
      </a>
      <CopyButton value={signature} />
    </span>
  );
}

export function evidenceAnchor(e: Evidence): string {
  const label = e.chain === "solana" ? "Solana slot" : e.chain === "hyper_core" ? "HyperEVM anchor block" : e.chain === "hyper_evm" ? "HyperEVM block" : `${e.chain} anchor`;
  return `${label} ${e.slot.toLocaleString("en-US")}`;
}

/** Preserve each source's chain and observation clock. */
export function EvidenceList({ evidence }: { evidence: Evidence[] }) {
  if (!evidence.length) return <Empty>No on-chain evidence was recorded for this.</Empty>;
  return (
    <LimitedItems className={styles.evidence} label="Evidence">
      {evidence.map((e) => (
        <li key={`${e.chain}.${e.address}.${e.field}.${e.slot}`}>
          <Address address={e.address} href={e.explorer_url} /> <span className={styles.muted}>{e.field}</span>{" "}
          <span className={styles.slot}>{evidenceAnchor(e)}</span>
        </li>
      ))}
    </LimitedItems>
  );
}
