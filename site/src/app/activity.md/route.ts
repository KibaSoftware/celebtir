import { loadSnapshot } from "@/lib/data";
import { activityMarkdown } from "@/lib/tablesMarkdown";

// H7: /activity.md, the Activity page's protocol table as Markdown, written at build.
export const dynamic = "force-static";

export async function GET() {
  return new Response(activityMarkdown(loadSnapshot()), { headers: { "content-type": "text/markdown; charset=utf-8" } });
}
