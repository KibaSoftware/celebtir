import { readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { filterLog, parseLog, serializeLog } from "./changelog";
import { loadSnapshot } from "./data";
import { GLOSSARY, withTerms } from "./glossary";
import { parseMarkdown } from "./markdown";
import { factorRules, loadMethodology, methodologyChangelog, methodologyVersions, stageRequirements } from "./methodology";
import type { ChangeEvent } from "./types";

describe("methodology page data", () => {
  it("reads every threshold from the published methodology file (SCO-2)", () => {
    const m = loadMethodology("0.1");
    const rules = factorRules(m);
    expect(rules.map((r) => r.factor)).toHaveLength(6);
    expect(rules[0]!.low).toContain(`${m.upgrade_control.low_min_voters}+ signers`);
    expect(rules[0]!.low).toContain(`${m.upgrade_control.low_min_threshold_pct}%+`);
    expect(rules[1]!.low).toBe("5 days or more, or immutable");
    // A different file gives different text: nothing is hard-coded.
    const changed = { ...m, upgrade_delay: { ...m.upgrade_delay, low_min_secs: 7 * 86_400 } };
    expect(factorRules(changed)[1]!.low).toBe("7 days or more, or immutable");
  });

  it("lists every published version and has a changelog entry for each", () => {
    const versions = methodologyVersions();
    expect(versions).toContain("0.1");
    const log = methodologyChangelog();
    for (const v of versions) expect(log).toContain(`## v${v}`);
  });

  it("states the Stage requirements as the scorer words them", () => {
    const reqs = stageRequirements(loadSnapshot());
    expect(reqs.length).toBeGreaterThan(3);
    expect(reqs[0]!.stage).toBe("stage_0");
    expect(new Set(reqs.map((r) => r.id)).size).toBe(reqs.length);
  });
});

describe("ui_6 glossary terms", () => {
  it("ui_6 links each term at its first use on the page only", () => {
    const seen = new Set<string>();
    const a = withTerms("The upgrade authority is a multisig with a time lock.", seen);
    expect(a.filter((p) => typeof p !== "string").map((p) => (p as { slug: string }).slug)).toEqual(["upgrade-authority", "multisig", "time-lock"]);
    const b = withTerms("Another multisig, another time lock.", seen);
    expect(b).toEqual(["Another multisig, another time lock."]);
  });

  it("ui_6 matches aliases and keeps the original wording", () => {
    const pieces = withTerms("Squads multisigs and their signers", new Set());
    expect(pieces.filter((p) => typeof p !== "string").map((p) => (p as { text: string }).text)).toEqual(["Squads", "multisigs", "signers"]);
    expect(pieces.map((p) => (typeof p === "string" ? p : p.text)).join("")).toBe("Squads multisigs and their signers");
  });

  it("ui_6 every term has a unique slug and a definition", () => {
    expect(new Set(GLOSSARY.map((t) => t.slug)).size).toBe(GLOSSARY.length);
    for (const t of GLOSSARY) expect(t.definition.length).toBeGreaterThan(20);
  });
});

describe("markdown", () => {
  it("renders headings, lists, paragraphs, code and safe links only", () => {
    const blocks = parseMarkdown("# Title\n\nIntro `x`.\n\n## v0.1\n\n- one [link](/a)\n- two [bad](javascript:alert(1))\n");
    expect(blocks.map((b) => b.kind)).toEqual(["paragraph", "heading", "list"]);
    const list = blocks[2]!;
    expect(list.kind === "list" && list.items[0]![1]).toEqual({ kind: "link", text: "link", href: "/a" });
    expect(list.kind === "list" && list.items[1]![1]).toEqual({ kind: "link", text: "bad", href: "#" });
  });
});

describe("change log filters", () => {
  const ev = (id: string, severity: ChangeEvent["severity"], source: ChangeEvent["source"], protocol: string | null): ChangeEvent => ({
    id,
    date: "2026-10-17",
    detected_within_days: 1,
    protocol,
    subject: "x",
    kind: "controller_changed",
    severity,
    source,
    summary: id,
    signature: null,
  });
  const events = [ev("a", "critical", "on_chain", "kamino-lend"), ev("b", "info", "registry_edit", "kamino-lend"), ev("c", "warning", "on_chain", "other")];

  it("filters by severity, source and protocol, kept in the URL", () => {
    const v = parseLog(new URLSearchParams("severity=critical,warning&source=on_chain&protocol=kamino-lend"));
    expect(filterLog(events, v).map((e) => e.id)).toEqual(["a"]);
    expect(serializeLog(v)).toBe("severity=critical%2Cwarning&source=on_chain&protocol=kamino-lend");
    expect(parseLog(new URLSearchParams("severity=apocalyptic&protocol=../x"))).toEqual({ severity: [], source: [], protocol: null });
  });
});

describe("lch_6 no cookies, no third parties", () => {
  const files = (dir: string): string[] =>
    readdirSync(dir).flatMap((f) => {
      const p = path.join(dir, f);
      return statSync(p).isDirectory() ? files(p) : /\.(tsx?|css)$/.test(f) && !f.includes(".test.") ? [p] : [];
    });
  const src = files(path.resolve("src")).map((f) => [f, readFileSync(f, "utf8")] as const);

  it("lch_6 no source file sets or reads cookies", () => {
    for (const [f, text] of src) expect(/document\.cookie|set-cookie/i.test(text), f).toBe(false);
  });

  it("lch_6 nothing is loaded from another origin: no remote scripts, fonts or analytics", () => {
    for (const [f, text] of src) {
      expect(/<script[^>]+src=["']https?:/i.test(text), f).toBe(false);
      expect(/next\/font\/google|googletagmanager|google-analytics|plausible|segment\.com/i.test(text), f).toBe(false);
      expect(/@import\s+url\(["']?https?:/i.test(text), f).toBe(false);
    }
  });
});
