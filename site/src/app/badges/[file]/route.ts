import { isAssessed } from "@/lib/assessment";
import { badgeSvg } from "@/lib/badge";
import { loadSnapshot } from "@/lib/data";
import { protocolIds } from "@/lib/protocol";

// API-4: /badges/<id>.svg, rebuilt from the latest snapshot on every daily
// build, so a Stage change shows the same day.
export const dynamic = "force-static";
export const dynamicParams = false;

export function generateStaticParams() {
  return protocolIds(loadSnapshot()).map((id) => ({ file: `${id}.svg` }));
}

export async function GET(_req: Request, { params }: { params: Promise<{ file: string }> }) {
  const id = (await params).file.replace(/\.svg$/, "");
  const s = loadSnapshot();
  if (!s.facts.protocols[id]) return new Response("unknown protocol", { status: 404 });
  const stage = isAssessed(s, id) ? s.scores.protocols[id]!.stage.stage : null;
  return new Response(badgeSvg(s.facts.protocols[id]?.name ?? id, stage), {
    headers: { "content-type": "image/svg+xml" },
  });
}
