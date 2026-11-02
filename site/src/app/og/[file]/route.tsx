import { ImageResponse } from "next/og";
import { OG_SIZE, OgCard } from "@/components/OgCard";
import { loadHistory, loadSnapshot } from "@/lib/data";
import { buildProtocolPage, protocolIds } from "@/lib/protocol";
import type { ChangeEvent, ProtocolHistoryFile } from "@/lib/types";

// UI-15: a share card per protocol, rendered at build to /og/<id>.png with
// the font bundled in next/og. A route handler (not the opengraph-image
// convention) so the file keeps its .png extension on static hosting.
export const dynamic = "force-static";
export const dynamicParams = false;

export function generateStaticParams() {
  return protocolIds(loadSnapshot()).map((id) => ({ file: `${id}.png` }));
}

export async function GET(_req: Request, { params }: { params: Promise<{ file: string }> }) {
  const id = (await params).file.replace(/\.png$/, "");
  const page = buildProtocolPage(loadSnapshot(), id, loadHistory<ProtocolHistoryFile>(id), loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? []);
  return new ImageResponse(<OgCard page={page} />, OG_SIZE);
}
