// UI-1: every view's state lives in the URL, so any view can be shared as a
// link. A pure codec: parse never throws (bad values fall back to defaults),
// serialize is stable (sorted choices, defaults omitted).

import { FACTORS, type Factor, type Stage } from "./types";

export const TABS = ["all", "lending", "dex", "perps", "liquid-staking"] as const;
export type Tab = (typeof TABS)[number];

const STAGES: readonly Stage[] = ["unrated", "stage_0", "stage_1", "stage_2"];
const RATINGS = ["low", "medium", "high", "unknown", "not_applicable"] as const;
const CONTROLLERS = ["single_key", "unrecognized", "multisig_without_delay", "multisig_with_delay", "immutable"] as const;
export const COMPARE_MAX = 4;
/** HUI-1: the chain filter; `all` shows both. */
export const CHAINS = ["all", "solana", "hyperliquid"] as const;
export type ChainFilter = (typeof CHAINS)[number];
export const RANGES = ["7d", "30d", "90d", "1y", "all"] as const;
/** Days of history each range covers; `all` has no limit. */
export const RANGE_DAYS: Record<Range, number | null> = { "7d": 7, "30d": 30, "90d": 90, "1y": 365, all: null };
export type Range = (typeof RANGES)[number];

export interface ViewState {
  protections?: import('./protections').ProtectionKey[];
  includeUnresolved?: boolean;
  protectionSort?: import('./protections').ProtectionSort;
  /** Chain filter (UI-9, HUI-1). */
  chain: ChainFilter;
  tab: Tab;
  sort: { key: string; dir: "asc" | "desc" };
  /** Hidden table columns (UI-11). */
  hidden: string[];
  filters: {
    stage: Stage[];
    controller: string[];
    rating: Partial<Record<Factor, string[]>>;
  };
  /** Compare selection, in the order chosen (UI-16); `p=` as in the site map. */
  compare: string[];
  /** Compare page: hide rows where every protocol matches (UI-17). */
  diff: boolean;
  /** Search text. */
  q: string;
  /** Time range of the charts (UI-8). */
  range: Range;
  /** Risk page sub-tab: one factor's details, or null for the matrix. */
  factor: Factor | null;
  /** Risk matrix: ratings to highlight; other cells are dimmed, not hidden. */
  highlight: string[];
  /** Risk matrix: limit the highlight to one factor's column, or null for all six. */
  highlightFactor: Factor | null;
  /** Risk matrix: the open detail panel, `<protocol id>.<factor>`. */
  detail: string | null;
  /** Tokens page: token types to show (empty: all). */
  tokenKind: string[];
  /** Tokens page: keep tokens with at least one factor at one of these ratings. */
  tokenRating: string[];
  tokenGroup?: "kind" | "issuer" | "flat";
  tokenGroups?: Record<string, boolean>;
  tokenDetail?: string;
  tokenProvenance?: "native" | "bridged";
  tokenUse?: string;
}

export const DEFAULT_VIEW: ViewState = {
  chain: "all",
  tab: "all",
  sort: { key: "tvl", dir: "desc" },
  hidden: [],
  filters: { stage: [], controller: [], rating: {} },
  compare: [],
  diff: false,
  q: "",
  range: "30d",
  factor: null,
  highlight: [],
  highlightFactor: null,
  detail: null,
  tokenKind: [],
  tokenRating: [],
};

const list = (v: string | null) => (v ? v.split(",").filter(Boolean) : []);
const only = <T extends string>(xs: string[], allowed: readonly T[]) =>
  xs.filter((x): x is T => (allowed as readonly string[]).includes(x));
const SAFE = /^[a-z0-9_-]{1,64}$/;

/** `kamino-lend.upgrade_control` → kept only if the id is safe and the factor exists. */
function parseDetail(v: string | null): string | null {
  if (!v) return null;
  const dot = v.lastIndexOf(".");
  const id = v.slice(0, dot);
  const factor = v.slice(dot + 1);
  return dot > 0 && SAFE.test(id) && (FACTORS as readonly string[]).includes(factor) ? v : null;
}

export function detailOf(v: ViewState): { id: string; factor: Factor } | null {
  if (!v.detail) return null;
  const dot = v.detail.lastIndexOf(".");
  return { id: v.detail.slice(0, dot), factor: v.detail.slice(dot + 1) as Factor };
}

export function parseView(p: URLSearchParams): ViewState {
  const tokenOptions: Partial<ViewState> = {};
  const protections = only(list(p.get('protections')), ['timelock','multi_actor','transparency','bypass','unresolved_authority'] as const);
  if(protections.length)tokenOptions.protections=[...new Set(protections)].sort();
  if(p.get('includeUnresolved')==='1')tokenOptions.includeUnresolved=true;
  const protectionSort=p.get('protectionSort');
  if(protectionSort==='delay_asc'||protectionSort==='delay_desc'||protectionSort==='unresolved')tokenOptions.protectionSort=protectionSort;
  const provenance = p.get("tprovenance");
  if (provenance === "native" || provenance === "bridged") tokenOptions.tokenProvenance = provenance;
  const use = p.get("tuse");
  if (use && SAFE.test(use)) tokenOptions.tokenUse = use;
  const grouping = p.get("tgroup");
  if (grouping === "issuer" || grouping === "flat") tokenOptions.tokenGroup = grouping;
  try {
    const groups: unknown = JSON.parse(p.get("tgroups") ?? "null");
    if (groups && typeof groups === "object" && !Array.isArray(groups)) tokenOptions.tokenGroups = Object.fromEntries(Object.entries(groups).filter(([key,value]) => key.length <= 100 && typeof value === "boolean").slice(0,100));
  } catch { /* Malformed optional state does not break navigation. */ }
  const tokenDetail = p.get("tdetail");
  if (tokenDetail && /^[a-zA-Z0-9]{32,64}\.(supply_control|freeze_and_seize|transfer_restrictions|provenance)$/.test(tokenDetail)) tokenOptions.tokenDetail = tokenDetail;
  const tab = p.get("tab") ?? "all";
  const chain = p.get("chain") ?? "all";
  const range = p.get("range") ?? "";
  const [key, dir] = (p.get("sort") ?? "").split(":");
  const rating: ViewState["filters"]["rating"] = {};
  for (const f of FACTORS) {
    const vals = only(list(p.get(`rating.${f}`)), RATINGS);
    if (vals.length) rating[f] = vals;
  }
  return {
    ...tokenOptions,
    chain: (CHAINS as readonly string[]).includes(chain) ? (chain as ChainFilter) : "all",
    tab: (TABS as readonly string[]).includes(tab) ? (tab as Tab) : "all",
    sort: key && SAFE.test(key) && (dir === "asc" || dir === "desc") ? { key, dir } : DEFAULT_VIEW.sort,
    hidden: list(p.get("hide")).filter((c) => SAFE.test(c)),
    filters: {
      stage: only(list(p.get("stage")), STAGES),
      controller: only(list(p.get("controller")), CONTROLLERS),
      rating,
    },
    diff: p.get("diff") === "1",
    compare: list(p.get("p")).filter((id) => SAFE.test(id)).slice(0, COMPARE_MAX),
    q: (p.get("q") ?? "").slice(0, 80),
    range: (RANGES as readonly string[]).includes(range) ? (range as Range) : DEFAULT_VIEW.range,
    factor: only([p.get("factor") ?? ""], FACTORS)[0] ?? null,
    highlight: only(list(p.get("hl")), RATINGS),
    highlightFactor: only([p.get("hlf") ?? ""], FACTORS)[0] ?? null,
    detail: parseDetail(p.get("detail")),
    tokenKind: list(p.get("kind")).filter((k) => SAFE.test(k)),
    tokenRating: only(list(p.get("trating")), RATINGS),
  };
}

export function serializeView(v: ViewState): string {
  const p = new URLSearchParams();
  if(v.protections?.length)p.set('protections',[...new Set(v.protections)].sort().join(','));
  if(v.includeUnresolved)p.set('includeUnresolved','1');
  if(v.protectionSort&&v.protectionSort!=='name')p.set('protectionSort',v.protectionSort);
  const sorted = (xs: string[]) => [...new Set(xs)].sort().join(",");
  if (v.chain !== DEFAULT_VIEW.chain) p.set("chain", v.chain);
  if (v.tab !== DEFAULT_VIEW.tab) p.set("tab", v.tab);
  if (v.sort.key !== DEFAULT_VIEW.sort.key || v.sort.dir !== DEFAULT_VIEW.sort.dir) {
    p.set("sort", `${v.sort.key}:${v.sort.dir}`);
  }
  if (v.hidden.length) p.set("hide", sorted(v.hidden));
  if (v.filters.stage.length) p.set("stage", sorted(v.filters.stage));
  if (v.filters.controller.length) p.set("controller", sorted(v.filters.controller));
  for (const f of FACTORS) {
    const vals = v.filters.rating[f];
    if (vals?.length) p.set(`rating.${f}`, sorted(vals));
  }
  if (v.compare.length) p.set("p", v.compare.slice(0, COMPARE_MAX).join(","));
  if (v.diff) p.set("diff", "1");
  if (v.q) p.set("q", v.q);
  if (v.range !== DEFAULT_VIEW.range) p.set("range", v.range);
  if (v.factor) p.set("factor", v.factor);
  if (v.highlight.length) p.set("hl", sorted(v.highlight));
  if (v.highlightFactor) p.set("hlf", v.highlightFactor);
  if (v.detail) p.set("detail", v.detail);
  if (v.tokenKind.length) p.set("kind", sorted(v.tokenKind));
  if (v.tokenRating.length) p.set("trating", sorted(v.tokenRating));
  if (v.tokenGroup && v.tokenGroup !== "kind") p.set("tgroup", v.tokenGroup);
  if (v.tokenGroups && Object.keys(v.tokenGroups).length) p.set("tgroups", JSON.stringify(Object.fromEntries(Object.entries(v.tokenGroups).sort(([a],[b]) => a.localeCompare(b)))));
  if (v.tokenDetail) p.set("tdetail", v.tokenDetail);
  if (v.tokenProvenance) p.set("tprovenance", v.tokenProvenance);
  if (v.tokenUse) p.set("tuse", v.tokenUse);
  return p.toString();
}
