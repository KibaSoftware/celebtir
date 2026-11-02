// Copies the published Atom feeds (CHG-6) and the static API (API-1) into
// public/ before `next build`, so the static export serves them. Same data folder rule as
// src/lib/data.ts: CELEBTIR_DATA, else <repo>/data if it has a snapshot, else
// the demo set.
import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";

const repo = path.resolve(process.cwd(), "..");
const real = path.join(repo, "data");
const dir = process.env.CELEBTIR_DATA
  ? path.resolve(process.env.CELEBTIR_DATA)
  : existsSync(path.join(real, "latest", "scores.json"))
    ? real
    : path.join(repo, "fixtures", "site-data");

/** Copies <data>/<from> to public/<to>, replacing what was there. */
function copy(from, to) {
  const target = path.join(process.cwd(), "public", to);
  rmSync(target, { recursive: true, force: true });
  mkdirSync(target, { recursive: true });
  const source = path.join(dir, from);
  if (existsSync(source)) cpSync(source, target, { recursive: true });
  console.log(`copy-data: ${path.relative(repo, source)} -> public/${to}`);
}

copy("feeds", "feeds"); // CHG-6
// API-1, D7: v2 replaced v1 before launch; no older version is published.
rmSync(path.join(process.cwd(), "public", "data"), { recursive: true, force: true });
copy(path.join("api", "v2"), path.join("data", "v2"));
