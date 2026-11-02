"use client";

import "@xyflow/react/dist/style.css";
import {
  Background,
  Controls,
  Handle,
  MarkerType,
  Position,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  type Edge,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { GraphPanel } from "./GraphPanel";
import { NodeCard } from "./NodeCard";
import type { ControlGraph, GraphNode } from "@/lib/graph";
import { chrome, ink, surfaces } from "@/lib/tokens";
import styles from "./Graph.module.css";

const COL_W = 440;
const ROW_H = 120;

type CardNode = Node<{ node: GraphNode; selected: boolean }, "card">;

function Card({ data }: NodeProps<CardNode>) {
  return (
    <>
      <Handle type="target" position={Position.Left} isConnectable={false} />
      <NodeCard node={data.node} selected={data.selected} />
      <Handle type="source" position={Position.Right} isConnectable={false} />
    </>
  );
}

const nodeTypes = { card: Card };

function Canvas({ graph }: { graph: ControlGraph }) {
  const [selected, setSelected] = useState<string | null>(null);
  const flow = useReactFlow();

  const rows = useMemo(() => {
    const m = new Map<number, number>();
    for (const n of graph.nodes) m.set(n.column, Math.max(m.get(n.column) ?? 0, n.row + 1));
    return m;
  }, [graph]);
  const tallest = Math.max(...rows.values());

  const nodes: CardNode[] = useMemo(
    () =>
      graph.nodes.map((n) => ({
        id: n.id,
        type: "card",
        position: { x: n.column * COL_W, y: (n.row + (tallest - (rows.get(n.column) ?? 1)) / 2) * ROW_H },
        data: { node: n, selected: n.id === selected },
        draggable: false,
        connectable: false,
        ariaLabel: `${n.title}${n.rating ? `, rated ${n.rating.replace("_", " ")}` : ""}`,
      })),
    [graph, rows, tallest, selected],
  );
  const edges: Edge[] = useMemo(
    () =>
      graph.edges.map((e) => ({
        id: e.id,
        source: e.source,
        target: e.target,
        label: e.label,
        markerEnd: { type: MarkerType.ArrowClosed, color: chrome.axis },
        style: { stroke: chrome.axis, strokeWidth: 1.5 },
        labelStyle: { fill: ink.secondary, fontSize: 11 },
        labelBgStyle: { fill: surfaces.surface },
        labelBgPadding: [4, 2] as [number, number],
      })),
    [graph],
  );

  const focus = useCallback(
    (id: string) => {
      if (!graph.nodes.some((n) => n.id === id)) return;
      setSelected(id);
      void flow.fitView({ nodes: [{ id }], duration: 300, maxZoom: 1.1 });
    },
    [graph, flow],
  );

  // UI-13: `#node-…` links select the node here too.
  useEffect(() => {
    const onHash = () => focus(decodeURIComponent(window.location.hash.slice(1)));
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, [focus]);

  const node = graph.nodes.find((n) => n.id === selected) ?? null;
  return (
    <div className={styles.canvasWrap}>
      <div className={styles.canvas}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          colorMode="dark"
          fitView
          minZoom={0.3}
          nodesConnectable={false}
          nodesDraggable={false}
          onNodeClick={(_, n) => setSelected(n.id)}
          onPaneClick={() => setSelected(null)}
          proOptions={{ hideAttribution: false }}
        >
          <Background color={chrome.grid} gap={24} />
          <Controls showInteractive={false} />
        </ReactFlow>
      </div>
      {node ? <GraphPanel key={node.id} node={node} onClose={() => setSelected(null)} /> : null}
    </div>
  );
}

export default function GraphCanvas({ graph }: { graph: ControlGraph }) {
  return (
    <ReactFlowProvider>
      <Canvas graph={graph} />
    </ReactFlowProvider>
  );
}
