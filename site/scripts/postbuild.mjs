// After `next build`: move each protocol's Markdown twin from /twins/<id>.md
// to /protocols/<id>.md (UI-14). A route handler can't share the
// /protocols/[id] segment with the page, so it renders under /twins/.
import { copyFileSync, existsSync, readdirSync, renameSync, rmSync, statSync } from "node:fs";
import path from "node:path";

const out = path.join(process.cwd(), "out");
const twins = path.join(out, "twins");
if (!existsSync(twins)) {
  console.error("postbuild: out/twins/ is missing; the Markdown twins were not generated");
  process.exit(1);
}
let moved = 0;
for (const name of readdirSync(twins)) {
  const from = path.join(twins, name);
  // Depending on trailingSlash, a route renders to a file or to <name>/index.
  const file = statSync(from).isDirectory() ? path.join(from, "index") : from;
  if (!name.endsWith(".md") || !existsSync(file)) continue;
  renameSync(file, path.join(out, "protocols", name));
  moved++;
}
rmSync(twins, { recursive: true, force: true });
console.log(`postbuild: ${moved} Markdown twins in out/protocols/`);

// Next 16 segment requests use dotted paths; Windows export emits nested
// segment folders. Publish aliases as real files so plain static hosts work.
function segmentAliases(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (!entry.isDirectory()) continue;
    if (entry.name.startsWith("__next.")) {
      function flatten(segmentDir, pieces) {
        for (const item of readdirSync(segmentDir, { withFileTypes: true })) {
          const source = path.join(segmentDir, item.name);
          const names = [...pieces, item.name];
          if (item.isDirectory()) flatten(source, names);
          else if (item.name.endsWith(".txt")) copyFileSync(source, path.join(dir, names.join(".")));
        }
      }
      flatten(full, [entry.name]);
    } else segmentAliases(full);
  }
}
segmentAliases(out);
