// The change log's filters, kept in the URL (UI-1) under their own keys.

import type { ChangeEvent } from "./types";

export const SEVERITIES = ["critical", "warning", "positive", "info"] as const;
export const SOURCES = ["on_chain", "registry_edit", "methodology"] as const;

export interface LogView {
  severity: ChangeEvent["severity"][];
  source: ChangeEvent["source"][];
  protocol: string | null;
}

const list = (v: string | null) => (v ? v.split(",").filter(Boolean) : []);
const only = <T extends string>(xs: string[], allowed: readonly T[]) => xs.filter((x): x is T => (allowed as readonly string[]).includes(x));

export function parseLog(p: URLSearchParams): LogView {
  const protocol = p.get("protocol");
  return {
    severity: only(list(p.get("severity")), SEVERITIES),
    source: only(list(p.get("source")), SOURCES),
    protocol: protocol && /^[a-z0-9_-]{1,64}$/.test(protocol) ? protocol : null,
  };
}

export function serializeLog(v: LogView): string {
  const p = new URLSearchParams();
  if (v.severity.length) p.set("severity", [...v.severity].sort().join(","));
  if (v.source.length) p.set("source", [...v.source].sort().join(","));
  if (v.protocol) p.set("protocol", v.protocol);
  return p.toString();
}

export function filterLog(events: ChangeEvent[], v: LogView): ChangeEvent[] {
  return events.filter(
    (e) =>
      (!v.severity.length || v.severity.includes(e.severity)) &&
      (!v.source.length || v.source.includes(e.source)) &&
      (!v.protocol || e.protocol === v.protocol),
  );
}
