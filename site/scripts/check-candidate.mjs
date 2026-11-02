// Validate the actual static export without a server. The HTTP gate also runs
// before deployment, so server routing/CORS cannot be hidden by this check.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { run } from "../../celebtir-invariants.mjs";
import { checkRelease } from "./check-release.mjs";

export async function checkCandidate(out, env = process.env) {
  const root = path.resolve(out);
  const request = async url => {
    const relative = decodeURIComponent(new URL(url).pathname).replace(/^\/+/, "");
    const file = path.resolve(root, relative);
    if (!file.startsWith(root + path.sep)) return new Response("", { status: 403 });
    try { return new Response(await readFile(file)); }
    catch { return new Response("", { status: 404 }); }
  };
  return [...checkRelease(env, root), ...await run("http://candidate", request)];
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  try {
    const failures = await checkCandidate(process.argv[2] ?? "out");
    failures.forEach(f => console.error(f));
    process.exitCode = failures.length ? 1 : 0;
    if (!failures.length) console.log("candidate: release configuration and invariants passed");
  } catch (e) { console.error(e); process.exitCode = 2; }
}
