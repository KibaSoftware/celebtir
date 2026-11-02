import { LimitedItems } from "@/components/LimitedItems";
import { RatingBadge } from "@/components/RatingBadge";
import { Address } from "@/components/Evidence";
import { KIND_LABELS, type ControlGraph } from "@/lib/graph";
import styles from "./Graph.module.css";

/**
 * UI-23: the graph as a list, with the same nodes and edges, for keyboard and
 * screen-reader users. Each node is a link target (`#node-…`) that can take
 * focus (UI-13).
 */
export function GraphList({ graph }: { graph: ControlGraph }) {
  const title = new Map(graph.nodes.map((n) => [n.id, n.title]));
  const order = [...graph.nodes].sort((a, b) => a.column - b.column || a.row - b.row);
  return (
    <LimitedItems className={styles.list} label="Control graph as a list">
      {order.map((n) => {
        const out = graph.edges.filter((e) => e.source === n.id);
        const into = graph.edges.filter((e) => e.target === n.id);
        return (
          <li key={n.id} id={n.id} tabIndex={-1} className={styles.listItem}>
            <span className={styles.listHead}>
              <span className={styles.kind}>{KIND_LABELS[n.kind]}</span> <strong>{n.title}</strong>{" "}
              {n.address ? <Address address={n.address} /> : null}{" "}
              {n.rating ? <RatingBadge rating={n.rating} /> : <span className={styles.muted}>{n.ratingReason}</span>}
            </span>
            {out.length || into.length ? (
              <details><summary>Connections ({out.length + into.length})</summary><LimitedItems className={styles.edges} label="connections">
                {out.map((e) => (
                  <li key={e.id}>
                    {e.label} → <a href={`#${e.target}`}>{title.get(e.target)}</a>
                  </li>
                ))}
                {into.map((e) => (
                  <li key={e.id}>
                    ← <a href={`#${e.source}`}>{title.get(e.source)}</a> {e.label}
                  </li>
                ))}
              </LimitedItems></details>
            ) : null}
          </li>
        );
      })}
    </LimitedItems>
  );
}
