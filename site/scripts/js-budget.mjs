// UI-27: JavaScript budgets on the static export (run after `next build`).
//  - The Summary page's initial JavaScript is under 250 KB gzip-compressed.
//  - Graph code (React Flow) never loads with a page's initial JavaScript:
//    the protocol page fetches it lazily.
//  - Chart code loads only on pages that draw charts.
import { readFileSync } from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

const out = path.resolve(process.cwd(), "out");
const BUDGET = 250 * 1024;
// Strings that only the graph and chart code contain.
const GRAPH = "react-flow__pane";
const CHART = "Use the left and right arrow keys to read each day.";

/** The JavaScript files a page loads up front. */
function initialScripts(route) {
  const html = readFileSync(path.join(out, route, "index.html"), "utf8");
  const srcs = new Set([...html.matchAll(/<script[^>]+src="([^"]+\.js)"/g)].map((m) => m[1]));
  return [...srcs].map((src) => ({ src, text: readFileSync(path.join(out, src.split("?")[0])) }));
}

const problems = [];
const report = (route, scripts) => {
  const gz = scripts.reduce((n, s) => n + gzipSync(s.text).length, 0);
  console.log(`js-budget: ${route} ${scripts.length} scripts, ${(gz / 1024).toFixed(1)} KB gzip`);
  return gz;
};

const summary = initialScripts("");
const gz = report("/", summary);
if (gz > BUDGET) problems.push(`Summary page loads ${(gz / 1024).toFixed(1)} KB of gzip JavaScript (budget 250 KB)`);

const pages = { "": true, "protocols/kamino-lend": true, about: false, methodology: false, value: true };
for (const [route, charts] of Object.entries(pages)) {
  const scripts = route === "" ? summary : initialScripts(route);
  if (route) report(`/${route}/`, scripts);
  const text = scripts.map((s) => s.text.toString("utf8")).join("\n");
  if (text.includes(GRAPH)) problems.push(`/${route}${route ? "/" : ""} loads graph code up front`);
  if (!charts && text.includes(CHART)) problems.push(`/${route}/ loads chart code without drawing a chart`);
}

for (const p of problems) console.error(`js-budget: ${p}`);
if (problems.length) process.exit(1);
console.log("js-budget: ok");
