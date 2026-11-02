// H7: the tables the Value and Activity pages sort in the browser, as
// Markdown, so readers without JavaScript (crawlers, LLM tools) get the same
// rows. Built from the same models the pages use.

import { CHANGE_LABEL } from "./activity";
import { formatChangeBp, formatUsd } from "./format";
import { CONTROL_LABELS } from "./labels";
import { activityRows, type DisplayActivity } from "./pages";
import { NOTICE } from "./site";
import { buildRows } from "./summary";
import type { ChangeEvent, ProtocolHistoryFile, Snapshot } from "./types";

const cell = (s: string) => s.replace(/\|/g, "\\|");

export function valueMarkdown(s: Snapshot, histories: Record<string, ProtocolHistoryFile | undefined>, events: ChangeEvent[]): string {
  const rows = buildRows(s, histories, events).sort((a, b) => a.rank - b.rank);
  const total = s.scores.value_by_control.all.total;
  return [
    "# Value by protocol",
    "",
    `> ${NOTICE}`,
    "",
    `Snapshot ${s.snapshotDate ?? ""}, methodology v${s.scores.methodology_version}. TVL from DefiLlama; context only, never a rating. Share is of assessed TVL (${formatUsd(total)}).`,
    "",
    "| # | Protocol | TVL | Share | Upgrade control | Controller |",
    "| --- | --- | --- | --- | --- | --- |",
    ...rows.map(r => `| ${r.rank} | [${cell(r.name)}](/protocols/${r.id}/) | ${r.tvl === null ? "unavailable" : formatUsd(r.tvl)} | ${r.tvl === null || !total || !r.controlClass ? "–" : `${((r.tvl / total) * 100).toFixed(1)}%`} | ${r.controlClass ? CONTROL_LABELS[r.controlClass] : "Unassessed"} | ${cell(r.controller)} |`),
    "",
  ].join("\n");
}

const activityCell = (a: DisplayActivity) => {
  if (a.kind === "not_applicable") return "not applicable";
  if (a.kind === "unavailable") return `unavailable (${cell(a.reason)})`;
  return `${formatUsd(a.latest)} on ${a.days.at(-1)?.date ?? "?"}; ${a.change_7d_bp === null ? "change unavailable" : formatChangeBp(a.change_7d_bp)}`;
};

export function activityMarkdown(s: Snapshot): string {
  const rows = activityRows(s);
  return [
    "# Activity by protocol",
    "",
    `> ${NOTICE}`,
    "",
    `Snapshot ${s.snapshotDate ?? ""}. Volume and fees from DefiLlama for the latest complete day, with the ${CHANGE_LABEL} (unavailable when either week has a missing or near-zero day).`,
    "",
    "| Protocol | Volume | Fees |",
    "| --- | --- | --- |",
    ...rows.map(r => `| [${cell(r.name)}](/protocols/${r.id}/) | ${activityCell(r.volume)} | ${activityCell(r.fees)} |`),
    "",
  ].join("\n");
}
