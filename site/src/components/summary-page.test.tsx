import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { SummaryChartsView } from "./charts/SummaryCharts";
import { COLUMNS, ProtocolTableView } from "./ProtocolTable";
import { ChainSelectorView } from "./shell/ChainSelector";
import { categoryCounts, withRanks, type ProtocolRow } from "@/lib/summary";
import type { SummaryDay } from "@/lib/types";
import { DEFAULT_VIEW, parseView, serializeView, type ViewState } from "@/lib/viewState";

afterEach(cleanup);

const ratings = {
  upgrade_control: "medium",
  upgrade_delay: "medium",
  admin_powers: "high",
  code_transparency: "low",
  oracle_dependency: "medium",
  dependencies: "unknown",
} as const;

function row(id: string, name: string, over: Partial<ProtocolRow> = {}): ProtocolRow {
  return {
    rank: 0,
    id,
    chain: "solana",
    name,
    category: "lending",
    website: null,
    assessment: "assessed",
    stage: "stage_0",
    nextStage: "stage_1",
    missing: ["Upgrades wait at least 7 days"],
    ratings: { ...ratings },
    reasons: { upgrade_control: "Squads 5 of 10.", upgrade_delay: "24 h time lock." } as ProtocolRow["reasons"],
    controller: "Squads 5 of 10 · 24 h delay",
    controlClass: "multisig_with_delay",
    tvl: 1_000_000,
    tvlChange7d: 0.05,
    activity: { kind: "fees", usd: 20_000, date: "2026-10-16", changeBp: -300 },
    lastChange: null,
    ...over,
  };
}

const ROWS = withRanks([
  row("kamino-lend", "Kamino Lend"),
  row("pump", "Pump Swap", { category: "dex", controlClass: "single_key", controller: "Single key", tvl: 500_000 }),
]);

/** The table with its view held in state, round-tripped through the URL codec like the real page. */
function Harness({ initial = DEFAULT_VIEW, onUrl }: { initial?: ViewState; onUrl?: (q: string) => void }) {
  const [view, setView] = useState(initial);
  return (
    <ProtocolTableView
      rows={ROWS}
      counts={categoryCounts(ROWS)}
      view={view}
      update={(next) => {
        const q = serializeView(next(view));
        onUrl?.(q);
        setView(parseView(new URLSearchParams(q)));
      }}
    />
  );
}

const bodyRows = () => within(screen.getAllByRole("rowgroup")[1]!).getAllByRole("row");

describe("ui_9 category tabs", () => {
  it("ui_9 tabs show counts and a definition, and filter the table", () => {
    render(<Harness />);
    const dex = screen.getByRole("button", { name: /Spot DEX/ });
    expect(dex.textContent).toContain("1");
    expect(screen.getByRole("button", { name: /^All/ }).textContent).toContain("2");
    fireEvent.click(dex);
    expect(dex.getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByText(/Swap tokens/)).toBeTruthy();
    expect(bodyRows()).toHaveLength(1);
  });

  it("ui_9 hui_1 the chain selector offers all chains, Solana and Hyperliquid", () => {
    let view = DEFAULT_VIEW;
    const { rerender } = render(<ChainSelectorView view={view} update={(f) => (view = f(view))} />);
    const chain = screen.getByRole("group", { name: "Chain" });
    const items = within(chain).getAllByRole("button");
    expect(items.map((o) => [o.textContent?.trim(), o.getAttribute("aria-pressed")])).toEqual([
      ["All chains", "true"],
      ["Solana", "false"],
      ["Hyperliquid", "false"],
    ]);
    fireEvent.click(items[2]!);
    expect(view.chain).toBe("hyperliquid");
    expect(serializeView(view)).toBe("chain=hyperliquid");
    rerender(<ChainSelectorView view={view} update={() => {}} />);
    expect(within(chain).getByRole("button", { name: "Hyperliquid" }).getAttribute("aria-pressed")).toBe("true");
  });

  it("hui_1 choosing Hyperliquid keeps only Hyperliquid rows and counts them", () => {
    const rows = [
      row("sol-a", "Sol A"),
      row("hl-a", "HL A", { chain: "hyperliquid", category: "liquid-staking" }),
      row("hl-b", "HL B", { chain: "hyperliquid" }),
    ];
    const view = parseView(new URLSearchParams("chain=hyperliquid"));
    render(
      <ProtocolTableView rows={rows} counts={categoryCounts(rows)} view={view} update={() => {}} />,
    );
    expect(bodyRows()).toHaveLength(2);
    expect(screen.getByRole("button", { name: /^All/ }).textContent).toContain("2");
    expect(screen.getByRole("button", { name: /Liquid staking/ }).textContent).toContain("1");
  });
});

describe("ui_10 protocol table", () => {
  it("ui_10 every column header is a sort button that flips direction", () => {
    let url = "";
    render(<Harness onUrl={(q) => (url = q)} />);
    const headers = screen.getAllByRole("columnheader");
    expect(headers).toHaveLength(COLUMNS.length);
    for (const h of headers) expect(within(h).getByRole("button")).toBeTruthy();
    const name = within(headers[1]!).getByRole("button");
    fireEvent.click(name);
    expect(url).toBe("sort=name%3Aasc");
    fireEvent.click(name);
    expect(url).toBe("sort=name%3Adesc");
    expect(screen.getAllByRole("columnheader")[1]!.getAttribute("aria-sort")).toBe("descending");
    expect(bodyRows()[0]!.getAttribute("data-protocol")).toBe("pump");
  });

  it("ui_10 shows rank, controller summary, TVL and activity with 7-day change", () => {
    render(<Harness />);
    const first = bodyRows()[0]!;
    expect(first.textContent).toContain("1");
    expect(first.textContent).toContain("Kamino Lend");
    expect(first.textContent).toContain("Squads 5 of 10 · 24 h delay");
    expect(first.textContent).toContain("$1M");
    expect(first.textContent).toContain("+5.0%");
    expect(first.textContent).toContain("$20K");
    expect(first.textContent).toContain("−3.0%");
    expect(first.textContent).toContain("None recorded");
    expect(within(first).getByRole("link", { name: "Kamino Lend" }).getAttribute("href")).toMatch(/^\/protocols\/kamino-lend\/?$/);
  });

  it("ui_10 the Stage tooltip opens on keyboard focus, lists what is missing and closes on Escape", () => {
    render(<Harness />);
    const stage = within(bodyRows()[0]!).getByRole("button", { name: "Stage 0" });
    const tip = document.getElementById(stage.getAttribute("aria-describedby")!)!;
    expect(tip.hidden).toBe(true);
    fireEvent.focus(stage);
    expect(tip.hidden).toBe(false);
    expect(tip.textContent).toContain("Missing for Stage 1");
    expect(tip.textContent).toContain("Upgrades wait at least 7 days");
      expect(tip.textContent).toContain("It is not a smart-contract audit, a measure of code safety, or investment advice.");
    fireEvent.keyDown(stage, { key: "Escape" });
    expect(tip.hidden).toBe(true);
  });

  it("ui_10 a click or tap opens the tooltip and leaves it open", () => {
    render(<Harness />);
    const stage = within(bodyRows()[0]!).getByRole("button", { name: "Stage 0" });
    const tip = document.getElementById(stage.getAttribute("aria-describedby")!)!;
    fireEvent.focus(stage);
    fireEvent.click(stage);
    expect(tip.hidden).toBe(false);
  });

  it("ui_10 the risk strip names all six ratings, one labelled cell each, and each tooltip opens by keyboard", () => {
    render(<Harness />);
    const strip = within(bodyRows()[0]!).getByRole("group", { name: /^Risk:/ });
    expect(strip.getAttribute("aria-label")).toBe(
      "Risk: Upgrade control Medium, Upgrade delay Medium, Admin powers High, Code transparency Low, Oracle dependency Medium, Dependencies Unknown",
    );
    const cells = within(strip).getAllByRole("link");
    expect(cells[0]!.getAttribute("href")).toContain("detail=kamino-lend.upgrade_control");
    expect(cells.map((c) => c.getAttribute("aria-label"))).toEqual([
      "UC Upgrade control: Medium",
      "UD Upgrade delay: Medium",
      "AP Admin powers: High",
      "CT Code transparency: Low",
      "OD Oracle dependency: Medium",
      "DP Dependencies: Unknown",
    ]);
    expect(cells.map((c) => c.textContent)).toEqual(["UC", "UD", "AP", "CT", "OD", "DP"]);
    fireEvent.focus(cells[0]!);
    const tip = document.getElementById(cells[0]!.getAttribute("aria-describedby")!)!;
    expect(tip.hidden).toBe(false);
    expect(tip.textContent).toContain("Upgrade control");
    expect(tip.textContent).toContain("Squads 5 of 10.");
    fireEvent.keyDown(cells[0]!, { key: "Escape" });
    expect(tip.hidden).toBe(true);
  });
});

describe("ui_11 column picker and filters", () => {
  it("ui_11 hiding a column and filtering to single key survive a reload via the URL", () => {
    let url = "";
    const { unmount } = render(<Harness onUrl={(q) => (url = q)} />);
    fireEvent.click(screen.getByRole("checkbox", { name: "Category" }));
    fireEvent.click(screen.getByRole("checkbox", { name: /Single key/ }));
    expect(url).toBe("hide=category&controller=single_key");
    unmount();

    // A reload: the page reads the same URL back.
    render(<Harness initial={parseView(new URLSearchParams(url))} />);
    expect(screen.queryByRole("columnheader", { name: /Category/ })).toBeNull();
    expect(bodyRows().map((r) => r.getAttribute("data-protocol"))).toEqual(["pump"]);
    expect(screen.getByText("Filters (1)")).toBeTruthy();
  });

  it("ui_11 the protocol name column cannot be hidden", () => {
    render(<Harness />);
    expect(screen.queryByRole("checkbox", { name: "Protocol" })).toBeNull();
  });

  it("ui_11 an empty result says so and offers to show everything", () => {
    render(<Harness initial={{ ...DEFAULT_VIEW, filters: { stage: ["stage_2"], controller: [], rating: {} } }} />);
    expect(screen.getByText(/No protocols match/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Show all" }));
    expect(bodyRows()).toHaveLength(2);
  });
});

function day(date: string, tvl: number): SummaryDay {
  return {
    date,
    protocols: 2,
    total_tvl_usd: tvl,
    value_by_control: {
      classes: { single_key: 0, unrecognized: 0, multisig_without_delay: 0, multisig_with_delay: tvl, immutable: 0 },
      total: tvl,
    },
    stages: { unrated: 0, stage_0: 2, stage_1: 0, stage_2: 0 },
    methodology_version: "0.1",
  };
}

describe("ui_8 summary charts", () => {
  const days = Array.from({ length: 12 }, (_, i) => day(`2026-10-${String(i + 1).padStart(2, "0")}`, 1e9 + i * 1e7));

  it("ui_8 charts wait for two snapshots", () => {
    render(<SummaryChartsView days={days.slice(0, 1)} view={DEFAULT_VIEW} setRange={() => {}} />);
    expect(screen.getByText(/two daily snapshots/)).toBeTruthy();
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("ui_8 three charts with legends and table views, and the range switches without a reload", () => {
    function Ranged() {
      const [view, setView] = useState(DEFAULT_VIEW);
      return <SummaryChartsView days={days} view={view} setRange={(range) => setView({ ...view, range })} />;
    }
    render(<Ranged />);
    expect(screen.getAllByRole("img")).toHaveLength(3);
    expect(screen.getAllByRole("list", { name: "Legend" })).toHaveLength(2);
    const tables = screen.getAllByRole("table", { hidden: true });
    expect(within(tables[0]!).getAllByRole("row", { hidden: true })).toHaveLength(13);
    fireEvent.click(screen.getByRole("button", { name: "7 days" }));
    expect(screen.getByRole("button", { name: "7 days" }).getAttribute("aria-pressed")).toBe("true");
    expect(within(screen.getAllByRole("table", { hidden: true })[0]!).getAllByRole("row", { hidden: true })).toHaveLength(8);
  });

  it("ui_8 the crosshair reads each day by keyboard", () => {
    render(<SummaryChartsView days={days} view={DEFAULT_VIEW} setRange={() => {}} />);
    const chart = screen.getAllByRole("img")[0]!;
    fireEvent.keyDown(chart, { key: "ArrowRight" });
    expect(screen.getByRole("status").textContent).toContain("2026-10-01");
    fireEvent.keyDown(chart, { key: "ArrowRight" });
    expect(screen.getByRole("status").textContent).toContain("2026-10-02");
    fireEvent.keyDown(chart, { key: "Escape" });
    expect(screen.queryByRole("status")).toBeNull();
  });
});
