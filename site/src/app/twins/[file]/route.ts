import { loadHistory, loadSnapshot } from "@/lib/data";
import { buildProtocolPage, protocolIds } from "@/lib/protocol";
import { protocolMarkdown } from "@/lib/twin";
import type { ChangeEvent, ProtocolHistoryFile } from "@/lib/types";

// UI-14: one Markdown twin per protocol, written at build. A route handler
// can't share `/protocols/[id]` with the page, so it renders here and
// scripts/postbuild.mjs moves each file to /protocols/<id>.md.
export const dynamic = "force-static";
export const dynamicParams = false;

export function generateStaticParams() {
  return protocolIds(loadSnapshot()).map((id) => ({ file: `${id}.md` }));
}

export async function GET(_req: Request, { params }: { params: Promise<{ file: string }> }) {
  const id = (await params).file.replace(/\.md$/, "");
  const s = loadSnapshot();
  const page = buildProtocolPage(s, id, loadHistory<ProtocolHistoryFile>(id), loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? []);
  const body = protocolMarkdown(page, { snapshotDate: s.run.finished_at.slice(0, 10), methodology: s.scores.methodology_version });
  return new Response(body, { headers: { "content-type": "text/markdown; charset=utf-8" } });
}
