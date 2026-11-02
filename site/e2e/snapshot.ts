import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

// The data the site under test was built from, chosen the same way as
// `src/lib/data.ts`: `CELEBTIR_DATA`, else `<repo>/data` if it has a
// snapshot, else the demo set. Specs derive their expectations from it.

const repo = path.join(__dirname, "../..");
const real = path.join(repo, "data");

export const DATA_DIR = process.env.CELEBTIR_DATA
  ? path.resolve(process.env.CELEBTIR_DATA)
  : existsSync(path.join(real, "latest", "scores.json"))
    ? real
    : path.join(repo, "fixtures", "site-data");

/** True when the build used the committed demo set (pixel baselines are taken on it). */
export const IS_DEMO = path.resolve(DATA_DIR) === path.resolve(repo, "fixtures", "site-data");

type Facts = { protocols: Record<string, { name?: string; chain?: string; tokens: Record<string, unknown> }> };
type Scores = { protocols: Record<string, { control_class: string }> };

const read = <T>(name: string): T => JSON.parse(readFileSync(path.join(DATA_DIR, "latest", name), "utf8")) as T;
export const facts = read<Facts>("facts.json");
export const scores = read<Scores>("scores.json");

const JITOSOL = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

/** The protocol compared with Kamino Lend: the first other one using JitoSOL. */
export const PEER = Object.keys(facts.protocols)
  .sort()
  .find((id) => id !== "kamino-lend" && JITOSOL in facts.protocols[id]!.tokens)!;
export const PEER_NAME = facts.protocols[PEER]?.name ?? PEER;

/** Protocols in a control class, sorted. */
export const withControlClass = (c: string) =>
  Object.entries(scores.protocols)
    .filter(([, s]) => s.control_class === c)
    .map(([id]) => id)
    .sort();

/** Protocol names per chain filter, from the built data (HUI-1). */
export const namesOn = (chain: "solana" | "hyperliquid") =>
  Object.values(facts.protocols)
    .filter((p) => ((p.chain === "hyper_evm" || p.chain === "hyper_core") ? "hyperliquid" : "solana") === chain)
    .map((p) => p.name ?? "")
    .filter(Boolean);
