"use client";

import dynamic from "next/dynamic";
import type { ControlGraph as Graph } from "@/lib/graph";
import styles from "./Graph.module.css";

// UI-27: React Flow loads only on pages with a graph, and only in the browser.
// The list view below it is pre-rendered, so the content works without it.
const GraphCanvas = dynamic(() => import("./GraphCanvas"), {
  ssr: false,
  loading: () => <p className={styles.loading}>Loading the interactive graph…</p>,
});

export function ControlGraph({ graph }: { graph: Graph }) {
  return <GraphCanvas graph={graph} />;
}
