import axe from "axe-core";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { GraphList } from "./GraphList";
import { GraphPanel } from "./GraphPanel";
import { BORDER_STYLES, NodeCard } from "./NodeCard";
import { SharedControllers } from "./SharedControllers";
import { loadSnapshot } from "@/lib/data";
import { JITOSOL, usersOf } from "@/lib/testSnapshot";
import { buildGraph, sharedControllers, type GraphNode } from "@/lib/graph";

afterEach(cleanup);

const signers = Array.from({ length: 9 }, (_, i) => `Signer${i}111111111111111111111111111111111111`);

const multisig: GraphNode = {
  id: "node-Msig111111111111111111111111111111111111111",
  kind: "multisig",
  title: "Squads 5 of 9 · 24 h delay",
  address: "Msig111111111111111111111111111111111111111",
  rating: "medium",
  ratingReason: "can upgrade Lending, rated medium.",
  facts: [{ label: "Threshold", value: "5 of 9" }],
  evidence: [
    { chain: "solana", address: "Msig111111111111111111111111111111111111111", slot: 42, field: "threshold", explorer_url: "https://solscan.io/account/Msig111111111111111111111111111111111111111" },
  ],
  signers: { count: 9, keys: signers },
  column: 1,
  row: 0,
  sharedWith: [{ id: "other", name: "Other Protocol" }],
};

describe("ui_20 side panel", () => {
  it("ui_20 shows the node's facts, evidence and Solscan link", () => {
    render(<GraphPanel node={multisig} onClose={() => {}} />);
    const panel = screen.getByRole("complementary", { name: /Squads 5 of 9/ });
    expect(panel.textContent).toContain("5 of 9");
    expect(panel.textContent).toContain("can upgrade Lending");
    expect(within(panel).getByRole("link", { name: /View on Solscan/ }).getAttribute("href")).toBe(
      "https://solscan.io/account/Msig111111111111111111111111111111111111111",
    );
    expect(within(panel).getByRole("list", { name: "Evidence" }).textContent).toContain("threshold");
    expect(within(panel).getByRole("link", { name: "Other Protocol" }).getAttribute("href")).toMatch(/^\/protocols\/other\/?/);
  });

  it("ui_20 a 9-member multisig shows 9 signers until expanded", () => {
    render(<GraphPanel node={multisig} onClose={() => {}} />);
    const toggle = screen.getByRole("button", { name: "9 signers" });
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText(/Sign…1111|Signer0/)).toBeNull();
    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByRole("list", { name: "Signers" }).querySelectorAll("li")).toHaveLength(9);
  });

  it("ui_20 the panel closes by its button and by Escape", () => {
    let closed = 0;
    render(<GraphPanel node={multisig} onClose={() => closed++} />);
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    fireEvent.keyDown(screen.getByRole("complementary"), { key: "Escape" });
    expect(closed).toBe(2);
  });
});

describe("ui_21 node colour always has a text label", () => {
  it("ui_21 every node shows its kind and its rating in words, not colour alone", () => {
    const g = buildGraph(loadSnapshot(), "kamino-lend");
    for (const n of g.nodes) {
      cleanup();
      const { container } = render(<NodeCard node={n} />);
      const card = container.firstElementChild as HTMLElement;
      expect(card.textContent).toContain(n.title);
      if (n.rating) {
        expect(card.getAttribute("data-rating")).toBe(n.rating);
        expect(card.querySelector(`[data-rating="${n.rating}"]`)?.textContent?.length).toBeGreaterThan(0);
      }
    }
  });
});

describe("ui_21 colour-blind safe", () => {
  it("ui_21 Low, Medium, High and Unknown differ by border pattern, not only by colour", () => {
    const patterns = (["low", "medium", "high", "unknown"] as const).map((r) => BORDER_STYLES[r]);
    expect(new Set(patterns).size).toBe(4);
    const { container } = render(<NodeCard node={{ ...multisig, rating: "high" }} />);
    expect((container.firstElementChild as HTMLElement).style.borderStyle).toBe("double");
  });
});

describe("ui_23 list view", () => {
  it("ui_23 the list view has every node and every edge of the graph", () => {
    const g = buildGraph(loadSnapshot(), "kamino-lend");
    render(<GraphList graph={g} />);
    const expand = screen.queryByRole("button", { name: /Show all/ });
    if (expand) fireEvent.click(expand);
    for (const n of g.nodes) expect(document.getElementById(n.id), n.id).not.toBeNull();
    const text = document.body.textContent!;
    for (const e of g.edges) for (const part of e.label.split("; ")) expect(text).toContain(part);
  });

  it("ui_23 the list view passes an axe accessibility scan", async () => {
    const { container } = render(
      <main>
        <h1>Graph</h1>
        <GraphList graph={buildGraph(loadSnapshot(), "kamino-lend")} />
      </main>,
    );
    // jsdom cannot compute colour contrast; the token tests cover it.
    const result = await axe.run(container, { rules: { "color-contrast": { enabled: false } } });
    expect(result.violations.map((v) => `${v.id}: ${v.help}`)).toEqual([]);
  });
});

describe("ui_22 shared controllers view", () => {
  it("ui_22 lists each shared controller once with links to every protocol", () => {
    const s = loadSnapshot();
    const shared = sharedControllers(s);
    render(<SharedControllers shared={shared} />);
    const item = screen.getByRole("article", { name: /6iQK…pVSS/ });
    expect(within(item).getAllByRole("link").map((a) => a.getAttribute("href")!.split("#")[0]!.replace(/\/$/, ""))).toEqual(
      expect.arrayContaining(usersOf(s, JITOSOL).map((id) => `/protocols/${id}`)),
    );
  });

  it("ui_22 with no shared controllers the view says so", () => {
    render(<SharedControllers shared={[]} />);
    expect(screen.getByText(/No key or multisig controls more than one protocol/)).toBeTruthy();
  });
});
