import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ProtocolSections, SECTIONS } from "./Sections";
import { loadHistory, loadSnapshot } from "@/lib/data";
import { FACTORS, type ChangeEvent, type Factor, type FactorScore, type ProtocolHistoryFile, type Snapshot } from "@/lib/types";
import { buildProtocolPage } from "@/lib/protocol";

afterEach(() => {
  cleanup();
  window.location.hash = "";
});

/** A protocol with nothing to show anywhere: every section must say why. */
function emptySnapshot(): Snapshot {
  const factor: FactorScore = { rating: "unknown", reason: "could not be read", next_step: "read it", evidence: [] };
  return {
    demo: true,
    run: { tool_version: "0", started_at: "2026-10-17T00:00:00Z", finished_at: "2026-10-17T00:10:00Z", protocols: ["bare"], failures: [] },
    facts: {
      schema_version: "0",
      protocols: {
        bare: {
          chain: "solana",
          category: null,
          last_reviewed: null,
          name: "Bare",
          website: null,
          reviewed_by: null,
          sources: [],
          programs: {},
          admin: {},
          oracles: {},
          dependencies: { value: { kind: "unavailable", reason: "no readable transactions in the sample window" }, evidence: [] },
          tokens: {},
          min_slot: 0,
          max_slot: 0,
        },
      },
      tokens: {},
      metrics: {},
      program_history: {},
    },
    scores: {
      schema_version: "0",
      methodology_version: "0.1",
      protocols: {
        bare: {
          control_class: "unrecognized",
          controller_summary: "No core programs",
          stage: { stage: "unrated", next: "stage_0", missing: [], requirements: [] },
          risks: [],
          factors: Object.fromEntries(FACTORS.map((f) => [f, factor])) as Record<Factor, FactorScore>,
          programs: {},
          tokens: {},
          admin: {},
          callees: {},
        },
      },
      tokens: {},
      value_by_control: { all: { classes: {}, total: 0 }, by_category: {}, excluded: [] },
    },
    coverage: { methodology_version: "0.1", assessed_chains: ["solana"], protocols: { bare: { chain: "solana", assessment: "assessed" } } },
  };
}

const section = (id: string) => document.getElementById(id)!;

describe("ui_12 all twelve sections, each with an empty state", () => {
  it("aud_6 a cold scan retains dated transaction-backed upgrades without duplicates", () => {
    const s = structuredClone(loadSnapshot());
    s.facts.program_history = {};
    const program = Object.keys(s.facts.protocols["kamino-lend"]!.programs)[0]!;
    const event: ChangeEvent = {id:"backfilled",protocol:"kamino-lend",source:"on_chain",kind:"program_upgraded",subject:program,summary:"upgraded",severity:"warning",date:"2026-10-26",occurred_on:"2026-10-19",detected_within_days:7,slot:42,signature:"transaction",evidence:[{chain:"solana",address:program,slot:42,field:"ProgramData transaction",explorer_url:"https://solscan.io/tx/transaction"}]};
    const page = buildProtocolPage(s, "kamino-lend", undefined, [event,event]);
    expect(page.upgradeEvents).toHaveLength(1);
    expect(page.upgradeEvents[0]!.date).toBe("2026-10-19");
    render(<ProtocolSections page={page} />);
    expect(section("upgrades").textContent).toContain("2026-10-19");
    expect(section("upgrades").querySelector('a[href="https://solscan.io/tx/transaction"]')).not.toBeNull();
    expect(buildProtocolPage(s,"kamino-lend",undefined,[{...event,evidence:[]}]).upgradeEvents).toHaveLength(0);
  });
  it("ui_12 the sections and the side navigation follow the spec's order", () => {
    expect(SECTIONS.map((s) => s.id)).toEqual([
      "summary", "risk-summary", "risk-analysis", "stage", "permissions", "control-graph",
      "programs", "dependencies", "value", "activity", "upgrades", "updates",
    ]);
    render(<ProtocolSections page={buildProtocolPage(emptySnapshot(), "bare", undefined, [])} />);
    const nav = screen.getByRole("navigation", { name: "Sections" });
    expect(within(nav).getAllByRole("link").map((a) => a.getAttribute("href"))).toEqual(SECTIONS.map((s) => `#${s.id}`));
  });

  for (const { id, title } of SECTIONS) {
    it(`ui_12 ${title} says why it has nothing to show`, () => {
      render(<ProtocolSections page={buildProtocolPage(emptySnapshot(), "bare", undefined, [])} />);
      const el = section(id);
      expect(within(el).getByRole("heading", { level: 2 }).textContent).toBe(title);
      const empty = el.querySelectorAll("[data-empty]");
      expect(empty.length, `${id} has no empty-state message`).toBeGreaterThan(0);
      for (const e of empty) expect(e.textContent!.trim().length).toBeGreaterThan(10);
      expect([...empty].map((e) => e.textContent)).toMatchSnapshot();
    });
  }

  it("ui_12 a populated protocol fills its sections from the published data", () => {
    const s = loadSnapshot();
    const events = loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [];
    const page = buildProtocolPage(s, "kamino-lend", loadHistory<ProtocolHistoryFile>("kamino-lend"), events);
    render(<ProtocolSections page={page} />);
    expect(section("summary").textContent).toContain("Squads-controlled");
    expect(section("summary").textContent).toContain("Verified build");
    expect(within(section("risk-analysis")).getAllByRole("article")).toHaveLength(6);
    expect(section("programs").textContent).toContain("Upgradeable loader");
    expect(section("programs").textContent).toContain("Squads 5 of 10 · 24 h delay");
    if (page.upgrades?.last_upgrade_date) expect(section("upgrades").textContent).toContain(page.upgrades.last_upgrade_date);
    else expect(section("upgrades").textContent).toContain("No upgrades were found");
    expect(section("permissions").querySelector("[data-empty]")).toBeNull();
    expect(section("dependencies").textContent).toContain("Lending");
  });
});

describe("ui_13 permissions link to the control graph", () => {
  it("ui_13 each actor lists its powers with their delay", () => {
    const s = loadSnapshot();
    render(<ProtocolSections page={buildProtocolPage(s, "kamino-lend", undefined, [])} />);
    const perms = section("permissions");
    const reveal = within(perms).queryByRole("button", {name: /Show all .* permission holders/});
    if (reveal) fireEvent.click(reveal);
    // Kamino's upgrade multisig; other 5-of-10 Squads may hold other powers.
    const squads = within(perms)
      .getAllByRole("article", { name: /Squads 5 of 10/ })
      .filter((a) => a.textContent?.includes("Upgrade the code"));
    expect(squads.length).toBeGreaterThan(0);
    // The program's upgrade multisig shows its 24 h time lock.
    expect(squads.some((a) => a.textContent?.includes("24 h delay"))).toBe(true);
  });

  it("ui_13 clicking an actor focuses it in the control graph", async () => {
    const s = loadSnapshot();
    render(<ProtocolSections page={buildProtocolPage(s, "kamino-lend", undefined, [])} />);
    const link = within(section("permissions")).getAllByRole("link", { name: /Show in control graph/ })[0]!;
    const target = link.getAttribute("href")!;
    expect(target).toMatch(/^#node-/);
    fireEvent.click(link);
    const node = document.getElementById(target.slice(1))!;
    expect(section("control-graph").contains(node)).toBe(true);
    await waitFor(() => expect(document.activeElement).toBe(node));
  });
});

describe("ui_12 feeds behind a protocol-run aggregator (edge case 11)", () => {
  function withScope(underlying: unknown) {
    const s = loadSnapshot();
    const k = s.facts.protocols["kamino-lend"]!;
    k.oracles = {
      ...k.oracles,
      "3t4JZcueEzTbVP6kLxXrL3VpWx45jDer4eqysweBchNH": {
        market: "SOL reserve",
        role: "primary",
        source: { value: { kind: "protocol_run" }, evidence: [] },
        underlying: underlying as never,
      },
    };
    return buildProtocolPage(s, "kamino-lend", undefined, []);
  }
  const card = () => document.getElementById("factor-oracle_dependency")!.closest("article")!;

  it("ui_12 the oracle card lists each feed the aggregator combines, by its own type name", () => {
    render(
      <ProtocolSections
        page={withScope({
          value: {
            kind: "listed",
            feeds: [
              { source_type: "Chainlink", address: "14HefJnxgiYQ6qiGL5b6GVZVUSAFpmehRdBJRaZ6GKx" },
              { source_type: "PythLazer", address: "HFn8GnPADiny6XqUoWE8uRPPxb29ikn4yTuPa9MF2fWJ" },
            ],
          },
          evidence: [{ chain: "solana", address: "d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q", slot: 1, field: "price_chain", explorer_url: "https://solscan.io/account/d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q" }],
        })}
      />,
    );
    const list = within(card()).getByRole("list", { name: /SOL reserve/ });
    expect(within(list).getAllByRole("listitem").map((li) => li.textContent)).toEqual([
      expect.stringContaining("Chainlink"),
      expect.stringContaining("PythLazer"),
    ]);
  });

  it("ui_12 an aggregator listing that cannot be read says why", () => {
    render(<ProtocolSections page={withScope({ value: { kind: "unavailable", reason: "the reserve now reads another price account" }, evidence: [] })} />);
    expect(card().textContent).toContain("the reserve now reads another price account");
  });
});
