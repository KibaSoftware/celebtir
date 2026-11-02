"use client";

import { ReasonText } from "@/components/ReasonText";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { useState } from "react";
import { RatingBadge } from "@/components/RatingBadge";
import { Address } from "@/components/Evidence";
import { accountUrl } from "@/lib/solscan";
import { KIND_LABELS, type GraphNode } from "@/lib/graph";
import styles from "./Graph.module.css";

/**
 * UI-20: a node's facts, evidence and Solscan link. Multisig signers start
 * collapsed as a count and expand on demand. Escape or Close dismisses it.
 * Render it with `key={node.id}` so the signer list collapses per node.
 */
export function GraphPanel({ node, onClose }: { node: GraphNode; onClose: () => void }) {
  const [open, setOpen] = useState(false);
  const titleId = `panel-${node.id}`;

  return (
    <aside
      className={styles.panel}
      aria-labelledby={titleId}
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
      }}
    >
      <div className={styles.panelHead}>
        <span className={styles.kind}>{KIND_LABELS[node.kind]}</span>
        <button type="button" className={styles.close} onClick={onClose}>
          Close
        </button>
      </div>
      <h3 id={titleId} className={styles.panelTitle}>
        {node.title}
      </h3>
      {node.address ? (
        <p className={styles.addrLine}>
          <Address address={node.address} />{" "}
          <a href={accountUrl(node.address)} rel="noopener">
            {node.address.startsWith("0x") ? "View on HyperEVMScan" : "View on Solscan"}
          </a>
        </p>
      ) : null}
      {node.rating ? (
        <p>
          <RatingBadge rating={node.rating} /> <span className={styles.muted}>{node.ratingReason}</span>
        </p>
      ) : (
        <p className={styles.muted}>{node.ratingReason}</p>
      )}
      <dl className={styles.facts}>
        {node.facts.map((f) => (
          <div key={f.label}>
            <dt>{f.label}</dt>
            <dd><ReasonText text={f.value} /></dd>
          </div>
        ))}
      </dl>
      {node.signers ? (
        <div>
          <button type="button" className={styles.toggle} aria-expanded={open} onClick={() => setOpen((o) => !o)}>
            {node.signers.count} signers
          </button>
          {open ? (
            node.signers.keys.length ? (
              <ul className={styles.signers} aria-label="Signers">
                {node.signers.keys.map((k) => (
                  <li key={k}>
                    <Address address={k} />
                  </li>
                ))}
              </ul>
            ) : (
              <p className={styles.muted}>Signer keys were not recorded in this snapshot.</p>
            )
          ) : null}
        </div>
      ) : null}
      {node.sharedWith.length ? (
        <p>
          Also controls:{" "}
          {node.sharedWith.map((p, i) => (
            <span key={p.id}>
              {i > 0 ? ", " : ""}
              <Link href={`/protocols/${p.id}/`}>{p.name}</Link>
            </span>
          ))}
        </p>
      ) : null}
      {node.evidence.length ? (
        <>
          <h4 className={styles.h4}>Evidence</h4>
          <ul className={styles.evidence} aria-label="Evidence">
            {node.evidence.map((e) => (
              <li key={`${e.address}.${e.field}`}>
                <Address address={e.address} href={e.explorer_url} />{" "}
                {e.field} <span className={styles.muted}>slot {e.slot.toLocaleString("en-US")}</span>
              </li>
            ))}
          </ul>
        </>
      ) : null}
    </aside>
  );
}
