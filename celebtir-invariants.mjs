// celebtir-invariants.mjs
// Regression checks for the fixes in CELEBTIR_FIX_INSTRUCTIONS.md.
// Reads the public v2 API and unassessed text exports. D1-D8 use methodology v0.3.
//
// Usage:  node celebtir-invariants.mjs [baseUrl]      (Node 18+, no dependencies)
// Exit code 0 = all checks pass, 1 = at least one failure.

const DEFAULT_BASE = "https://celebtir.com";

function reviewedNone(r) {
  return Boolean(r && /^https?:\/\//.test(r.source) && /^\d{4}-\d{2}-\d{2}$/.test(r.reviewed_on) && r.note?.trim());
}
const CLASSES = ["single_key", "unrecognized", "multisig_without_delay", "multisig_short_delay", "multisig_with_delay", "immutable"];
const weakest = (...cs) => CLASSES[Math.min(...cs.map(c => CLASSES.indexOf(c)))];
function delay(c) {
  if (!c) return null;
  if (c.kind === "program_controlled") return delay(c.upgrade);
  if (["single_key", "unrecognized"].includes(c.kind)) return 0;
  if (["multisig", "timelock"].includes(c.kind)) {
    const own = c.time_lock_secs ?? c.min_delay_secs;
    const bypass = c.config_authority ?? c.admin;
    return bypass ? (delay(bypass) == null ? null : Math.min(own, delay(bypass))) : own;
  }
  return null;
}
export function controllerClass(c) {
  if (!c) return "unrecognized";
  if (c.kind === "program_controlled") return controllerClass(c.upgrade);
  if (["single_key", "immutable"].includes(c.kind)) return c.kind;
  if (!["multisig", "timelock"].includes(c.kind)) return "unrecognized";
  const d = delay(c) ?? 0;
  let own = d === 0 ? "multisig_without_delay" : d < 86400 ? "multisig_short_delay" : "multisig_with_delay";
  if (c.kind === "timelock") own = weakest(own, controllerClass(c.proposer));
  const bypass = c.config_authority ?? c.admin;
  return bypass ? weakest(own, controllerClass(bypass)) : own;
}

// ---------- helpers ----------
const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function b58decode(s) {
  let n = 0n;
  for (const c of s) {
    const i = B58.indexOf(c);
    if (i < 0) throw new Error(`bad base58: ${s}`);
    n = n * 58n + BigInt(i);
  }
  const out = [];
  while (n > 0n) { out.unshift(Number(n & 255n)); n >>= 8n; }
  for (const c of s) { if (c === "1") out.unshift(0); else break; }
  return Uint8Array.from(out);
}
const P = 2n ** 255n - 19n;
const modpow = (b, e, m) => { let r = 1n; b %= m; while (e > 0n) { if (e & 1n) r = (r * b) % m; b = (b * b) % m; e >>= 1n; } return r; };
const inv = (x) => modpow(((x % P) + P) % P, P - 2n, P);
const D = ((-121665n * inv(121666n)) % P + P) % P;
// true if the 32-byte Solana address is a valid ed25519 point (could have a private key);
// false means it is a program-derived address (PDA) that no one holds a key for.
function isOnCurve(addr) {
  const b = b58decode(addr);
  if (b.length !== 32) return false;
  let y = 0n;
  for (let i = 31; i >= 0; i--) y = (y << 8n) | BigInt(b[i]);
  y &= (1n << 255n) - 1n;
  if (y >= P) return false;
  const y2 = (y * y) % P;
  const x2 = (((y2 - 1n + P) % P) * inv((D * y2 + 1n) % P)) % P;
  if (x2 === 0n) return true;
  return modpow(x2, (P - 1n) / 2n, P) === 1n;
}

// ---------- checks ----------
export async function run(base = DEFAULT_BASE, request = fetch, { applicationUpdate = false } = {}) {
  const text = async (path) => {
    const r = await request(base.replace(/\/$/, "") + path);
    if (!r.ok) throw new Error(`${path}: HTTP ${r.status}`);
    return r.text();
  };
  const get = async (path) => {
    return JSON.parse(await text(path));
  };
  const failures = [];
  let historical = false;
  const fail = (check, msg) => {
    // Code-only updates cannot correct immutable v0.1/v0.2 evidence or scores.
    // Still enforce v2 publication/assessment and reviewer hygiene. The default
    // launch gate and every v0.3 update retain the complete audit contract.
    if (!historical || /^(API|F5|H2|F2 token missing)/.test(check)) failures.push(`[${check}] ${msg}`);
  };

  const summary = await get("/data/v2/summary.json");
  historical = applicationUpdate && ["0.1", "0.2"].includes(summary.methodology_version);
  const events = (await get("/data/v2/events.json")).events || [];
  if (summary.api_version !== "v2" || !summary.protocols?.length) fail("API", "missing v2 protocol coverage");
  if (summary.methodology_version !== "0.3") fail("methodology", "release candidate must use v0.3; historical snapshots remain immutable");
  const protos = {};
  for (const p of summary.protocols) protos[p.id] = { s: p, d: await get(`/data/v2/protocols/${p.id}.json`) };
  const snap = new Date(summary.snapshot_date + "T00:00:00Z");

  // F1: the same program ID must carry the same code-transparency fact everywhere.
  const byProgram = {};
  for (const [id, { d }] of Object.entries(protos))
    for (const [pid, prog] of Object.entries(d.facts.programs || {})) {
      const v = prog.code_transparency?.value || {};
      // HyperEVM's existing verifier records provider attestations, not Git commits.
      // Require the source URL to bind to the implementation in its evidence.
      const sourcify = d.facts.chain === "hyper_evm" && v.kind === "verified" &&
        /^https:\/\/repo\.sourcify\.dev\/999\/0x[a-fA-F0-9]{40}$/.test(v.repo ?? "") &&
        (prog.code_transparency.evidence ?? []).some(e => e.chain === "hyper_evm" && e.field === "sourcify verified source" && v.repo.toLowerCase().endsWith(`/${e.address.toLowerCase()}`));
      const etherscan = d.facts.chain === "hyper_evm" && v.kind === "verified" &&
        /^https:\/\/hyperevmscan\.io\/address\/0x[a-fA-F0-9]{40}#code$/.test(v.repo ?? "") &&
        v.commit === "etherscan:etherscan" &&
        (prog.code_transparency.evidence ?? []).some(e => e.chain === "hyper_evm" && e.field === "etherscan verified source" && typeof e.address === "string" && v.repo.toLowerCase() === `https://hyperevmscan.io/address/${e.address.toLowerCase()}#code`);
      if (v.repo && !sourcify && !etherscan && !/^https:\/\/github\.com\/[^/]+\/[^/#?]+\/?$/.test(v.repo)) fail("F1 repository", `${pid}: not a repository URL or evidenced HyperEVM verifier URL: ${v.repo}`);
      if (v.commit && !etherscan && !(sourcify && /^sourcify:(exact_match|match)$/.test(v.commit)) && !/^[a-f0-9]{40}$/.test(v.commit)) fail("F1 commit", `${pid}: not a full commit or evidenced HyperEVM verification marker`);
      (byProgram[pid] ||= []).push({ id, sig: `${v.kind}|${v.repo || ""}|${v.commit || ""}` });
    }
  for (const [pid, rows] of Object.entries(byProgram))
    if (new Set(rows.map((r) => r.sig)).size > 1)
      fail("F1 program-keyed source", `${pid}: ${rows.map((r) => `${r.id}=${r.sig}`).join(" vs ")}`);

  // F2: no off-curve address (PDA) may be classified as an unrecognized key on token authorities.
  const mints = new Set();
  for (const { d } of Object.values(protos)) Object.keys(d.facts.tokens || {}).forEach((m) => mints.add(m));
  for (const mint of mints) {
    let t;
    try { t = await get(`/data/v2/tokens/${mint}.json`); } catch (e) { fail("F2 token missing", String(e)); continue; }
    for (const field of ["mint_authority", "freeze_authority"]) {
      const v = t.facts?.[field]?.value;
      if (["unrecognized", "single_key"].includes(v?.kind) && v.key && !isOnCurve(v.key))
        fail("F2 PDA as key", `${mint} ${field}: ${v.key} is off-curve (program-derived) but labelled unrecognized`);
      const factor = field === "mint_authority" ? "supply_control" : "freeze_and_seize";
      if (v?.kind === "pda" && t.score?.factors?.[factor]?.rating !== "unknown") fail("F2 unresolved PDA", `${mint} ${field}: must be Unknown`);
    }
  }

  // F3: an empty registry list must not produce a Low rating (missing data is Unknown).
  for (const [id, { s, d }] of Object.entries(protos)) {
    if ((d.facts.chain || "solana") !== "solana") continue;
    if (Object.keys(d.facts.admin || {}).length === 0 && s.ratings?.admin_powers === "low" && !reviewedNone(d.facts.admin_review))
      fail("F3 missing data rated Low", `${id}: admin_powers=low with no admin accounts read`);
    if (Object.keys(d.facts.oracles || {}).length === 0 && s.ratings?.oracle_dependency === "not_applicable" && (!reviewedNone(d.facts.oracle_review) || ["lending", "perps"].includes(d.facts.category))) fail("F3 missing oracle review", id);
  }

  // F5: protocols outside the assessed chain(s) must not carry a Stage or factor ratings.
  for (const [id, { s, d }] of Object.entries(protos)) {
    const chain = d.facts.chain;
    if (!chain || s.chain !== chain || d.chain !== chain) fail("F5 chain field", `${id}: inconsistent chain`);
    if (!["assessed", "unassessed"].includes(s.assessment) || s.assessment !== d.assessment || (s.assessment === "assessed") !== (summary.assessed_chains || []).includes(chain)) fail("F5 assessment", `${id}: inconsistent assessment`);
    if (s.assessment === "unassessed") {
      if (s.stage && !/unassessed/.test(s.stage)) fail("F5 unassessed leak", `${id} (${chain}): stage=${s.stage}`);
      const rated = Object.entries(s.ratings || {}).filter(([, r]) => r && r !== "unassessed");
      if (rated.length) fail("F5 unassessed leak", `${id} (${chain}): ${rated.length} factor ratings published`);
      if (d.score != null || s.control_class != null || d.history?.some(h => h.stage != null || h.ratings != null || h.control_class != null)) fail("F5 unassessed leak", `${id}: score, class or history leaked`);
      for (const path of [`/protocols/${id}.md`, `/badges/${id}.svg`]) {
        if (/stage[ _-]?[012]/i.test(await text(path))) fail("F5 unassessed export", path);
      }
      const html = await text(`/protocols/${id}/index.html`);
      const metadata = [...html.matchAll(/<title\b[^>]*>([^<]*)<\/title>|<meta\b[^>]*>/gi)].map(m => m[0]);
      if (metadata.some(tag => /stage[ _-]?[012]/i.test(tag))) fail("F5 unassessed metadata", id);
      const llms = await text("/llms.txt");
      if (llms.split("\n").some(line => line.includes(`/protocols/${id}.md`) && /stage[ _-]?[012]/i.test(line))) fail("F5 unassessed export", `llms.txt: ${id}`);
    } else if (s.stage == null || s.ratings == null || d.score == null) {
      fail("F5 assessed missing score", id);
    }
    if (!("chain" in s)) fail("F5 chain field", `${id}: summary.json has no chain field`);
  }

  // F6: every upgrade / authority change in the last 30 days must appear in events.json.
  for (const [id, { d }] of Object.entries(protos))
    for (const [pid, h] of Object.entries(d.program_history || {}))
      for (const e of h.events || []) {
        if (!["upgrade", "set_authority", "remove_authority"].includes(e.kind)) continue;
        const age = (snap - new Date(e.date + "T00:00:00Z")) / 864e5;
        if (age < 0 || age > 30) continue;
        const matches = events.filter(x => x.protocol === id && x.signature === e.signature && x.slot === e.slot && x.evidence?.some(ev => ev.address === pid && ev.slot === e.slot && ev.explorer_url?.endsWith(`/tx/${e.signature}`)));
        const expectedKind = { upgrade: "program_upgraded", set_authority: "upgrade_authority_set", remove_authority: "upgrade_authority_removed" }[e.kind];
        const found = matches.length === 1 && matches[0].source === "on_chain" && matches[0].kind === expectedKind && matches[0].occurred_on === e.date && matches[0].detected_within_days === Math.max(0, Math.round((new Date(matches[0].date) - new Date(e.date)) / 864e5));
        if (!found) fail("F6 missing event", `${id} ${pid.slice(0, 8)}… ${e.kind} on ${e.date} (slot ${e.slot}) not in events.json`);
      }

  // F7 / D4 option B: zero, (0, 24h), and 24h+, including bypass controllers.
  for (const [id, { s, d }] of Object.entries(protos)) {
    if (s.assessment !== "assessed") continue;
    const classes = Object.values(d.facts.programs || {})
      .filter((p) => p.role === "core")
      .map((p) => controllerClass(p.upgrade_control?.value));
    const expected = CLASSES[Math.min(...classes.map(c => CLASSES.indexOf(c)))];
    if (s.control_class !== expected) fail("F7 bucket", `${id}: expected ${expected}, got ${s.control_class}`);
  }

  // F8: no 7-day change may be published when either 7-day window contains a gap day.
  for (const [id, { d }] of Object.entries(protos))
    for (const k of ["fees", "volume"]) {
      const m = d.metrics?.[k];
      if (m?.change_7d_bp == null) continue;
      if (!m.days?.length) { fail("F8 missing series", `${id} ${k}`); continue; }
      const byDate = new Map(m.days.map(d => [d.date, d.usd]));
      const end = Date.parse(m.days.at(-1).date + "T00:00:00Z");
      const days = Array.from({length: 14}, (_, i) => new Date(end - i * 864e5).toISOString().slice(0, 10));
      const vals = m.days.map((x) => x.usd).sort((a, b) => a - b);
      const median = vals[Math.floor(vals.length / 2)];
      const gaps = days.filter(date => !byDate.has(date) || byDate.get(date) < median * 0.05);
      if (gaps.length) fail("F8 7d change over gaps", `${id} ${k}: change_7d_bp=${m.change_7d_bp} computed over gap day(s) ${gaps.join(", ")}`);
    }

  // F12: registry labels for multisigs must match the version the reader detected.
  //      (F4, F9, F10, F11 need manual or on-chain checks; see the instructions file.)
  for (const [id, { d }] of Object.entries(protos)) {
    const detected = {};
    for (const p of Object.values(d.facts.programs || {})) {
      const u = p.upgrade_control?.value;
      if (u?.address && u.multisig_kind) detected[u.address] = u.multisig_kind;
    }
    for (const a of Object.values(d.facts.admin || {})) {
      const c = (a.value || a).controller?.value;
      if (c?.address && c.multisig_kind) detected[c.address] = c.multisig_kind;
    }
    for (const src of d.facts.sources || []) {
      if (src.kind !== "multisig" || !detected[src.address]) continue;
      const want = detected[src.address].replace("squads_v", "Squads v");
      if (src.label !== want) fail("F12 Squads label", `${id}: ${src.address.slice(0, 8)}… labelled "${src.label}", reader detected ${detected[src.address]}`);
    }
  }

  // H2: no draft reviewer may ship.
  for (const [id, { d }] of Object.entries(protos))
    if (/draft/i.test(d.facts.reviewed_by || "")) fail("H2 draft reviewer", `${id}: reviewed_by=${d.facts.reviewed_by}`);

  return failures;
}

// ---------- CLI ----------
if (typeof process !== "undefined" && process.argv?.[1]?.endsWith("celebtir-invariants.mjs")) {
  const base = process.argv[2] || DEFAULT_BASE;
  const applicationUpdate = process.argv.includes("--application-update");
  run(base, fetch, { applicationUpdate }).then((f) => {
    if (!f.length) { console.log(`${applicationUpdate ? 'Application-update compatibility checks' : 'All checks'} passed against ${base}${applicationUpdate ? '; historical v0.1/v0.2 scores are preserved, not launch-audited' : ''}`); process.exit(0); }
    console.log(`${f.length} failure(s) against ${base}:\n` + f.join("\n"));
    process.exit(1);
  }).catch((e) => { console.error(e); process.exit(2); });
}
