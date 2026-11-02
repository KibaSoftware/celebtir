import { describe, expect, it } from "vitest";
import { loadSnapshot } from "./data";
import { logoFor, logoMap } from "./logos";
import { searchIndex, searchMatches } from "./search";

const s = loadSnapshot();

describe("ui_3 the global search index", () => {
  const index = searchIndex(s);

  it("ui_3 lists every protocol and token in the snapshot, with a descriptive line", () => {
    for (const id of Object.keys(s.facts.protocols)) expect(index.some((i) => i.kind === "Protocol" && i.href === `/protocols/${id}/`)).toBe(true);
    for (const mint of Object.keys(s.facts.tokens)) expect(index.some((i) => i.kind === "Token" && i.href === `/tokens/${mint}/`)).toBe(true);
    for (const [id] of Object.entries(s.facts.protocols)) {
      const item = index.find(i => i.href === `/protocols/${id}/`)!;
      if (s.scores.protocols[id]) expect(item.detail).toMatch(/Stage|Unrated/);
      else { expect(item.detail).toContain("Unassessed"); expect(item.detail).not.toMatch(/Stage [012]/); }
    }
  });

  it("ui_3 controllers come from the facts only, each listed once, linking a graph node", () => {
    const controllers = index.filter((i) => i.kind === "Controller");
    expect(controllers.length).toBeGreaterThan(0);
    expect(new Set(controllers.map((c) => c.keywords)).size).toBe(controllers.length);
    for (const c of controllers) expect(c.href).toMatch(/^\/protocols\/[a-z0-9-]+\/#node-/);
  });

  it("ui_3 matches rank a name prefix first and group by kind", () => {
    const m = searchMatches(
      [
        { label: "Stake Pool", href: "/b", kind: "Protocol" },
        { label: "SOL staking", href: "/c", kind: "Token" },
        { label: "Jito", href: "/a", kind: "Protocol", detail: "stake pool" },
      ],
      "sta",
    );
    expect(m.map((x) => x.href)).toEqual(["/b", "/a", "/c"]);
    expect(searchMatches(index, "   ")).toEqual([]);
  });
});

describe("protocol logos are placeholders until added", () => {
  it("a protocol without a file in public/logos gets the monogram", () => {
    expect(logoFor("no-such-protocol")).toBeNull();
    expect(logoMap(["no-such-protocol"])).toEqual({ "no-such-protocol": null });
  });
});
