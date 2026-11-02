import { loadHistory, loadSnapshot } from "@/lib/data";
import { valueMarkdown } from "@/lib/tablesMarkdown";
import type { ChangeEvent, ProtocolHistoryFile } from "@/lib/types";

// H7: /value.md, the Value page's protocol table as Markdown, written at build.
export const dynamic = "force-static";

export async function GET() {
  const s = loadSnapshot();
  const histories = Object.fromEntries(Object.keys(s.facts.protocols).map(id => [id, loadHistory<ProtocolHistoryFile>(id)]));
  const events = loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [];
  return new Response(valueMarkdown(s, histories, events), { headers: { "content-type": "text/markdown; charset=utf-8" } });
}
