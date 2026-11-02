// Protocol logos (placeholders until the owner adds them): drop a square
// `public/logos/<protocol id>.svg` (or .png / .webp) and it is used
// everywhere the protocol appears; without one, the monogram stands in.
// Resolved at build time, so no page probes for files that do not exist.

import { existsSync } from "node:fs";
import path from "node:path";

const EXTENSIONS = ["svg", "png", "webp"] as const;
const dir = path.resolve(process.cwd(), "public", "logos");

export function logoFor(id: string): string | null {
  for (const ext of EXTENSIONS) {
    if (existsSync(path.join(dir, `${id}.${ext}`))) return `/logos/${id}.${ext}`;
  }
  return null;
}

export function logoMap(ids: string[]): Record<string, string | null> {
  return Object.fromEntries(ids.map((id) => [id, logoFor(id)]));
}
