// Smoke test of the deployed static API (API-1, API-3, API-5): the files are
// served, readable from another origin, and carry their licence.
// Usage: node site/scripts/check-api.mjs https://<site>
import { pathToFileURL } from "node:url";

const PATHS = ["data/v2/summary.json", "data/v2/events.json", "data/v2/schemas/summary.schema.json"];

/** Resolves to a list of problems; empty when the API checks out. */
export async function checkApi(base) {
  const root = base.replace(/\/$/, "");
  const problems = [];
  for (const p of PATHS) {
    const res = await fetch(`${root}/${p}`, { headers: { Origin: "https://example.org" } });
    if (!res.ok) {
      problems.push(`${p}: HTTP ${res.status}`);
      continue;
    }
    if (res.headers.get("access-control-allow-origin") !== "*") {
      problems.push(`${p}: not readable cross-origin (API-3)`);
    }
    const body = await res.json();
    if (p.endsWith("summary.json") && body?.licence?.name !== "CC BY 4.0") {
      problems.push(`${p}: no CC BY 4.0 licence field (API-5)`);
    }
  }
  return problems;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const base = process.argv[2];
  if (!base) {
    console.error("usage: node site/scripts/check-api.mjs <base-url>");
    process.exit(2);
  }
  const problems = await checkApi(base);
  for (const p of problems) console.error(`check-api: ${p}`);
  if (problems.length) process.exit(1);
  console.log(`check-api: ok (${base})`);
}
