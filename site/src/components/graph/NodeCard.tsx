import { RatingBadge } from "@/components/RatingBadge";
import { shortAddress } from "@/lib/format";
import { KIND_LABELS, type GraphNode } from "@/lib/graph";
import { ratingStyle, type Rating } from "@/lib/tokens";
import styles from "./Graph.module.css";

/**
 * Red and green collapse for deuteranopes (validator: ΔE 4.1), so the border
 * pattern also encodes the rating, next to its icon and text (UI-21).
 */
export const BORDER_STYLES: Record<Rating, "solid" | "dashed" | "double" | "dotted"> = {
  low: "solid",
  medium: "dashed",
  high: "double",
  unknown: "dotted",
  not_applicable: "solid",
};

/**
 * One node as a box (UI-19). The border takes the rating's status colour, and
 * the rating is always written out with its icon beside it (UI-21).
 */
export function NodeCard({ node, selected = false }: { node: GraphNode; selected?: boolean }) {
  return (
    <div
      className={styles.card}
      data-kind={node.kind}
      data-rating={node.rating ?? undefined}
      data-selected={selected || undefined}
      style={
        node.rating
          ? { borderColor: ratingStyle[node.rating].color, borderStyle: BORDER_STYLES[node.rating], borderWidth: node.rating === "high" ? 4 : 2 }
          : undefined
      }
    >
      <span className={styles.kind}>{KIND_LABELS[node.kind]}</span>
      <span className={styles.title}>{node.title}</span>
      {node.address ? <span className={styles.addr} title={node.address}>
          {shortAddress(node.address)}
        </span> : null}
      <span className={styles.foot}>
        {node.rating ? <RatingBadge rating={node.rating} /> : <span className={styles.stage}>{node.ratingReason}</span>}
        {node.signers ? <span className={styles.meta}>{node.signers.count} signers</span> : null}
        {node.sharedWith.length ? <span className={styles.meta}>+{node.sharedWith.length} protocol{node.sharedWith.length > 1 ? "s" : ""}</span> : null}
      </span>
    </div>
  );
}
