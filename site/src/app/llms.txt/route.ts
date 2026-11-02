import { loadSnapshot } from "@/lib/data";
import { llmsTxt } from "@/lib/llms";

// UI-14: /llms.txt, written at build.
export const dynamic = "force-static";

export async function GET() {
  return new Response(llmsTxt(loadSnapshot()), { headers: { "content-type": "text/plain; charset=utf-8" } });
}
