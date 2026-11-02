import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import ActivityPage from "@/app/activity/page";
import TokenPage from "@/app/tokens/[mint]/page";
import TokensPage from "@/app/tokens/page";
import ValuePage from "@/app/value/page";
import { ArchivedList } from "./ArchivedList";
import { RiskViewBody } from "./RiskView";
import { filterTokens, type TokenTableRow } from "./TokenTable";
import { loadHistory, loadSnapshot } from "@/lib/data";
import type { TokenHistoryFile } from "@/lib/types";
import { activityRows, factorRows } from "@/lib/pages";
import { buildRows } from "@/lib/summary";
import { FACTORS, type ChangeEvent, type Factor, type ProtocolHistoryFile } from "@/lib/types";
import { DEFAULT_VIEW, parseView, serializeView } from "@/lib/viewState";

// Value, Activity and Tokens keep their filters in the URL; the app router is not mounted in tests.
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => new URLSearchParams(),
}));

afterEach(cleanup);

const USDC = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

function RiskHarness({ onUrl, highlightFixture = false }: { onUrl: (q: string) => void; highlightFixture?: boolean }) {
  const s = structuredClone(loadSnapshot());
  const ids = Object.keys(s.scores.protocols);
  // Exercise both matching and dimmed cells independently of live ratings.
  if (highlightFixture) {
    s.scores.protocols[ids[0]!].factors.upgrade_delay.rating = "high";
    s.scores.protocols[ids[0]!].factors.upgrade_control.rating = "low";
  }
  const rows = buildRows(
    s,
    Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])),
    loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [],
  );
  const details = Object.fromEntries(FACTORS.map((f) => [f, factorRows(s, f)])) as Record<Factor, ReturnType<typeof factorRows>>;
  const [view, setView] = useState(DEFAULT_VIEW);
  return (
    <RiskViewBody
      rows={rows}
      details={details}
      view={view}
      update={(next) => {
        const q = serializeView(next(view));
        onUrl(q);
        setView(parseView(new URLSearchParams(q)));
      }}
    />
  );
}

describe("risk page", () => {
  it("native HyperEVM records never inherit the Solana matrix ratings", () => {
    const s = loadSnapshot();
    const row = buildRows(s, {}, [])[0]!;
    const details = Object.fromEntries(FACTORS.map(f => [f, factorRows(s, f)])) as Record<Factor, ReturnType<typeof factorRows>>;
    render(<RiskViewBody rows={[{...row, chain: "hyperliquid", assessment: "unassessed", stage: null, ratings: null, reasons: null, controlClass: null}]} details={details} view={DEFAULT_VIEW} update={() => {}} />);
    expect(screen.getByText(/Native HyperEVM assessment is Unassessed/)).toBeTruthy();
    expect(document.querySelector("tr[data-protocol]")).toBeNull();
    expect(screen.queryByRole("button", {name: /Open the evidence/})).toBeNull();
  });
  it("shows the protocols × six factors matrix, every rating in words", () => {
    render(<RiskHarness onUrl={() => {}} />);
    const table = screen.getByRole("table");
    expect(within(table).getAllByRole("columnheader")).toHaveLength(8);
    const row = table.querySelector('[data-protocol="kamino-lend"]')!;
    expect(row.querySelectorAll("[data-rating]")).toHaveLength(6);
  });

  it("a factor sub-tab shows reasons, next steps and evidence, and lives in the URL", () => {
    let url = "";
    render(<RiskHarness onUrl={(q) => (url = q)} />);
    fireEvent.click(within(screen.getByRole("group", { name: "View" })).getByRole("button", { name: "Upgrade control" }));
    expect(url).toBe("factor=upgrade_control");
    const table = screen.getByRole("table");
    expect(within(table).getByRole("columnheader", { name: "To improve" })).toBeTruthy();
    expect(within(table).getAllByRole("list", { name: "Evidence" }).length).toBeGreaterThan(0);
  });

  it("ui_1 a rating cell opens its detail panel, kept in the URL, observed facts apart from the criterion", () => {
    let url = "";
    render(<RiskHarness onUrl={(q) => (url = q)} />);
    fireEvent.click(screen.getByRole("button", { name: /^Kamino Lend, Upgrade control: \w+\. Open the evidence$/ }));
    expect(url).toBe("detail=kamino-lend.upgrade_control");
    const dialog = screen.getByRole("dialog", { name: "Upgrade control" });
    expect(within(dialog).getByRole("heading", { name: "Observed configuration" })).toBeTruthy();
    expect(within(dialog).getByRole("heading", { name: "Methodology criterion" })).toBeTruthy();
    expect(within(dialog).getByRole("heading", { name: "What would improve the rating" })).toBeTruthy();
    expect(within(dialog).getByRole("list", { name: "Evidence" })).toBeTruthy();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(url).toBe("");
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("ui_1 highlighting a rating dims the other cells and says how many protocols match", () => {
    let url = "";
    render(<RiskHarness highlightFixture onUrl={(q) => (url = q)} />);
    fireEvent.click(within(screen.getByRole("group", { name: "Highlight ratings" })).getByRole("button", { name: /High/ }));
    expect(url).toBe("hl=high");
    const highs = document.querySelectorAll("td[data-match]");
    expect(highs.length).toBeGreaterThan(0);
    for (const td of highs) expect(td.querySelector("[data-rating]")!.getAttribute("data-rating")).toBe("high");
    expect(screen.getByRole("status").textContent).toMatch(/of \d+ protocols match/);
  });

  it("matrix columns sort by risk", () => {
    let url = "";
    render(<RiskHarness onUrl={(q) => (url = q)} />);
    // The sub-tab buttons and the column headers share names; this is the header.
    fireEvent.click(within(screen.getByRole("table")).getByRole("button", { name: "Admin powers" }));
    expect(url).toContain("sort=admin_powers%3Adesc");
  });
});

describe("value page", () => {
  it("lists TVL by protocol, value by control per category and asset groups", () => {
    render(<ValuePage />);
    expect(screen.getByRole("heading", { name: "TVL by protocol" })).toBeTruthy();
    expect(screen.getByRole("row", { name: /Kamino Lend/ }).textContent).toContain("Multisig, delay 24 h+");
    expect(screen.getByRole("rowheader", { name: "All" })).toBeTruthy();
    expect(screen.getByRole("rowheader", { name: "Stablecoins" })).toBeTruthy();
  });
});

describe("activity page", () => {
  it("lists volume and fees per protocol with their 7-day change", () => {
    render(<ActivityPage />);
    const kamino = document.querySelector('[data-protocol="kamino-lend"]')!;
    const fees = activityRows(loadSnapshot()).find(r => r.id === "kamino-lend")!.fees;
    if (fees.kind === "usd") {
      expect(kamino.textContent).toMatch(/\$/);
      if (fees.change_7d_bp !== null) expect(kamino.textContent).toMatch(/[+−]\d+\.\d%/);
    } else {
      expect(kamino.textContent).toContain(fees.reason);
      expect(kamino.textContent).not.toMatch(/[+−]\d+\.\d%/);
    }
  });
});

describe("tok_6 token directory filters", () => {
  const rows: TokenTableRow[] = [
    { mint: "A1", symbol: "AAA", kind: "stablecoin", issuer: "Circle", program: "spl_token", provenance: "Native to Solana", ratings: { supply_control: "high", freeze_and_seize: "high", transfer_restrictions: "low", provenance: "low" }, reasons: { supply_control: "", freeze_and_seize: "", transfer_restrictions: "", provenance: "" }, exposure: [], risks: [] },
    { mint: "B2", symbol: "BBB", kind: "liquid-staking", issuer: "Jito", program: "spl_token", provenance: "Native to Solana", ratings: { supply_control: "low", freeze_and_seize: "low", transfer_restrictions: "low", provenance: "unknown" }, reasons: { supply_control: "", freeze_and_seize: "", transfer_restrictions: "", provenance: "" }, exposure: [], risks: [] },
  ];
  const factors = ["supply_control", "freeze_and_seize", "transfer_restrictions", "provenance"] as const;

  it("tok_6 search, type and rating filters come from the URL view", () => {
    expect(filterTokens(rows, factors, parseView(new URLSearchParams("q=jito"))).map((t) => t.symbol)).toEqual(["BBB"]);
    expect(filterTokens(rows, factors, parseView(new URLSearchParams("kind=stablecoin"))).map((t) => t.symbol)).toEqual(["AAA"]);
    expect(filterTokens(rows, factors, parseView(new URLSearchParams("trating=unknown"))).map((t) => t.symbol)).toEqual(["BBB"]);
    expect(filterTokens(rows, factors, DEFAULT_VIEW)).toHaveLength(2);
  });

  it("tok_6 a row expands to the issuer's powers and back", () => {
    render(<TokensPage />);
    const button = screen.getAllByRole("button", { name: /^Show .+ details$/ })[0]!;
    fireEvent.click(button);
    expect(button.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText(/^Issuer-side powers over/)).toBeTruthy();
    fireEvent.click(button);
    expect(screen.queryByText(/^Issuer-side powers over/)).toBeNull();
  });

  it("highwatch groups the directory by asset type without dropping token identities", () => {
    render(<TokensPage />);
    expect(screen.getByRole("navigation", { name: "Token asset groups" })).toBeTruthy();
    const rows = [...document.querySelectorAll("tr[data-mint]")];
    const mints = rows.map(r => r.getAttribute("data-mint"));
    expect(new Set(mints).size).toBe(Object.keys(loadSnapshot().facts.tokens).length);
    expect(document.querySelectorAll("tr[data-token-group]").length).toBeGreaterThan(1);
  });
});

describe("tok_6 token pages", () => {
  it("tok_6 the token table shows four ratings per token and who uses it", () => {
    render(<TokensPage />);
    const usdc = document.querySelector(`[data-mint="${USDC}"]`)!;
    expect(usdc.querySelectorAll("[data-rating]")).toHaveLength(4);
    expect(within(usdc as HTMLElement).getByRole("link", { name: "Kamino Lend" })).toBeTruthy();
  });

  it("tok_6 every token page shows four ratings with evidence", async () => {
    for (const mint of Object.keys(loadSnapshot().facts.tokens)) {
      cleanup();
      render(await TokenPage({ params: Promise.resolve({ mint }) }));
      const cards = screen.getAllByRole("article");
      expect(cards).toHaveLength(4);
      for (const c of cards) expect(within(c).getByRole("list", { name: "Evidence" })).toBeTruthy();
    }
  });

  it("tok_6 the token page names the issuer, its controllers and the risk summary", async () => {
    render(await TokenPage({ params: Promise.resolve({ mint: USDC }) }));
    expect(screen.getByRole("link", { name: "Circle" })).toBeTruthy();
    expect(screen.getByRole("rowheader", { name: "Freeze authority" })).toBeTruthy();
    expect(screen.getByText(/^Your balance can be frozen if/)).toBeTruthy();
  });

  it("tok_9 the supply chart waits for 30 snapshots and says how many exist", async () => {
    const days = loadHistory<TokenHistoryFile>(`tokens/${USDC}`)!.days.length;
    expect(days).toBeLessThan(30);
    render(await TokenPage({ params: Promise.resolve({ mint: USDC }) }));
    expect(screen.getByText(`once 30 daily snapshots exist (${days} so far)`, { exact: false })).toBeTruthy();
  });
});

describe("archived page", () => {
  it("shows each archived protocol's reason, date, source and last known ratings", () => {
    render(
      <ArchivedList
        archived={[
          {
            id: "old-dex",
            name: "Old DEX",
            archived_on: "2026-09-01",
            reason: "Shut down by its team.",
            source: "https://example.com/sunset",
            last_known: {
              date: "2026-08-31",
              tvl_usd: 1,
              fees_usd: null,
              volume_usd: null,
              stage: "stage_0",
              ratings: Object.fromEntries(FACTORS.map((f) => [f, "medium"])) as Record<Factor, "medium">,
              methodology_version: "0.1",
            },
          },
        ]}
      />,
    );
    const card = screen.getByRole("article", { name: "Old DEX" });
    expect(card.textContent).toContain("Archived 2026-09-01: Shut down by its team.");
    expect(card.textContent).toContain("Last rated 2026-08-31");
    expect(card.querySelectorAll("[data-rating]")).toHaveLength(6);
  });

  it("with nothing archived the page says so", () => {
    render(<ArchivedList archived={[]} />);
    expect(screen.getByText("No protocol has been archived yet.")).toBeTruthy();
  });
});
