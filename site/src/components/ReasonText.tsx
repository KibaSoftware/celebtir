import { accountUrl } from "@/lib/solscan";
import { Fragment } from "react";
import { withTerms } from "@/lib/glossary";
import { Tooltip } from "./Tooltip";
import { shortAddress } from "@/lib/format";

/** Presentation only: historical score reasons remain byte-identical. */
export function ReasonText({ text, seen }: { text: string; seen?: Set<string> }) {
  const normalized = text
    .replace(/Funds can be stolen if a core program is upgraded to new code by (.+)\.$/, "Core program code can be replaced by $1.")
    .replace(/Funds can be stolen if a harmful upgrade is approved, because it takes effect (.+), before users can withdraw\.$/, "An approved upgrade takes effect $1. Withdrawal completion within that period is not established by these configuration facts.")
    .replace(/Funds can be stolen if they are moved by (.+)\.$/, "Funds can be moved by $1.")
    .replace(/Funds can be stolen if protocol parameters are changed by (.+)\.$/, "Protocol parameters can be changed by $1.")
    .replace(/Funds can be stolen if the deployed code of (.+) does something its users cannot review, because its source is not public\.$/, "The source of deployed program $1 is not public, so users cannot review its behavior from published source.")
    .replace(/Funds can be stolen if the deployed code of (.+) differs from its public source, because the build is not verified\.$/, "The deployed code of $1 has not been verified against its public source.")
    .replace(/Funds can be stolen if (.+), a program this protocol calls with user funds, is upgraded by (.+)\.$/, "Program $1, which this protocol calls with user funds, can be upgraded by $2.")
    // F2: older reasons call an off-curve, System-owned authority an
    // "unrecognized controller"; it is a program-derived address. The
    // scorer shortens addresses with an ellipsis (`1111…1111`).
    .replace(/unrecognized controller (\S+) owned by program (?:11111111111111111111111111111111|1111…1111|1111\.\.\.1111)/g, "program-derived address $1 (controlling program not resolved)");
  const parts = normalized.split(/(`[^`]+`|https:\/\/github\.com\/[^\s;,]+|\b0x[a-fA-F0-9]{40}\b|\b[1-9A-HJ-NP-Za-km-z]{32,44}\b)/g);
  return <>{parts.map((p, i) => {
    if (p.startsWith("`") && p.endsWith("`")) return <code key={i}>{p.slice(1, -1)}</code>;
    if (p.startsWith("https://github.com/")) {
      const url = new URL(p);
      const segments = url.pathname.split("/").filter(Boolean);
      const label = segments.length >= 4 && ["tree", "commit"].includes(segments[2]!) ? `${segments[0]}/${segments[1]}@${segments[3]!.slice(0, 7)}` : segments.slice(0, 2).join("/");
      return <a key={i} href={p}>{label}</a>;
    }
    if (/^(?:0x[a-fA-F0-9]{40}|[1-9A-HJ-NP-Za-km-z]{32,44})$/.test(p)) return <a key={i} href={accountUrl(p)} title={p}>{shortAddress(p)}</a>;
    if (seen) return <Fragment key={i}>{withTerms(p, seen).map((part, j) => typeof part === "string" ? part : <Tooltip key={j} href={`/glossary/#${part.slug}`} trigger={part.text}>{part.definition}</Tooltip>)}</Fragment>;
    return <Fragment key={i}>{p}</Fragment>;
  })}</>;
}

export function GroupedReason({ text, seen }: { text: string; seen?: Set<string> }) {
  const entries = text.split(/; (?=[1-9A-HJ-NP-Za-km-z]{32,44}: )/).map(p => p.match(/^([1-9A-HJ-NP-Za-km-z]{32,44}): ([\s\S]*)$/));
  if (entries.length < 2 || entries.some(e => !e)) return <ReasonText text={text} seen={seen} />;
  const groups = new Map<string, string[]>();
  for (const e of entries) groups.set(e![2]!, [...(groups.get(e![2]!) ?? []), e![1]!]);
  return <>{[...groups].map(([reason, addresses]) => <details key={reason}><summary>{addresses.length} program{addresses.length === 1 ? "" : "s"}: <ReasonText text={reason} seen={seen} /></summary><ul>{addresses.map(a => <li key={a}><a href={accountUrl(a)} title={a}>{shortAddress(a)}</a></li>)}</ul></details>)}</>;
}
