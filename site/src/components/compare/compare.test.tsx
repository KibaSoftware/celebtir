import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { CompareBody } from "./CompareView";
import { CompareToggle } from "./CompareToggle";
import { CompareTray } from "./CompareTray";
import { RiskViewBody } from "@/components/page/RiskView";
import { ProtocolTableView } from "@/components/ProtocolTable";
import { compareItems, type CompareItem } from "@/lib/compare";
import { loadHistory, loadSnapshot } from "@/lib/data";
import { factorRows } from "@/lib/pages";
import { buildRows, categoryCounts } from "@/lib/summary";
import { FACTORS, type ChangeEvent, type Factor, type ProtocolHistoryFile } from "@/lib/types";
import { DEFAULT_VIEW, parseView, serializeView, type ViewState } from "@/lib/viewState";
import { nameOf, peerOfKamino } from "@/lib/testSnapshot";

// The protocol compared with Kamino Lend in the snapshot being tested.
const PEER = peerOfKamino(loadSnapshot());
const PEER_NAME = nameOf(loadSnapshot(), PEER);


beforeEach(() => window.localStorage.clear());
afterEach(() => {
  cleanup();
  window.localStorage.clear();
});

const s = loadSnapshot();
const ids = Object.keys(s.scores.protocols);
const histories = Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)]));
const rows = buildRows(s, histories, loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? []);
const names = Object.fromEntries(rows.map((r) => [r.id, r.name]));
const items = compareItems(s, histories);

describe("ui_4 compare tray", () => {
  it("ui_4 adding protocols from Summary and Risk lands both in one compare view", () => {
    const details = Object.fromEntries(FACTORS.map((f) => [f, factorRows(s, f)])) as Record<Factor, ReturnType<typeof factorRows>>;
    const summary = render(<ProtocolTableView rows={rows} counts={categoryCounts(rows)} view={DEFAULT_VIEW} update={() => {}} />);
    fireEvent.click(screen.getByRole("checkbox", { name: "Compare Kamino Lend" }));
    summary.unmount();

    // Another page: the Risk table.
    const risk = render(<RiskViewBody rows={rows} details={details} view={DEFAULT_VIEW} update={() => {}} />);
    expect((screen.getByRole("checkbox", { name: "Compare Kamino Lend" }) as HTMLInputElement).checked).toBe(true);
    fireEvent.click(screen.getByRole("checkbox", { name: `Compare ${PEER_NAME}` }));
    risk.unmount();

    render(<CompareTray names={names} />);
    const tray = screen.getByRole("complementary", { name: "Compare tray" });
    expect(tray.textContent).toContain("Kamino Lend");
    expect(tray.textContent).toContain(PEER_NAME);
    expect(within(tray).getByRole("link", { name: "Compare 2" }).getAttribute("href")).toMatch(new RegExp(`^/compare/?\\?p=kamino-lend,${PEER}$`));
  });

  it("ui_4 the tray persists until cleared, and holds at most 4", () => {
    const four = ["a", "b", "c", "d"];
    const all = { ...names, a: "A", b: "B", c: "C", d: "D", e: "E" };
    render(
      <>
        {[...four, "e"].map((id) => (
          <CompareToggle key={id} id={id} name={all[id as keyof typeof all]} />
        ))}
        <CompareTray names={all} />
      </>,
    );
    for (const id of four) fireEvent.click(screen.getByRole("checkbox", { name: `Compare ${all[id as keyof typeof all]}` }));
    expect((screen.getByRole("checkbox", { name: "Compare E" }) as HTMLInputElement).disabled).toBe(true);
    expect(screen.getByText("Compare (4 of 4)")).toBeTruthy();
    cleanup();

    // A new page load reads the same tray back.
    render(<CompareTray names={all} />);
    expect(screen.getByText("Compare (4 of 4)")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    expect(screen.queryByRole("complementary", { name: "Compare tray" })).toBeNull();
  });

  it("ui_4 removing one protocol from the tray", () => {
    window.localStorage.setItem("celebtir.compare", JSON.stringify(["kamino-lend", PEER]));
    render(<CompareTray names={names} />);
    fireEvent.click(screen.getByRole("button", { name: "Remove Kamino Lend from the compare tray" }));
    expect(screen.getByText("Add one more to compare")).toBeTruthy();
  });
});

function Harness({ start, all = items, tray = [] }: { start: Partial<ViewState>; all?: Record<string, CompareItem>; tray?: string[] }) {
  const [view, setView] = useState<ViewState>({ ...DEFAULT_VIEW, ...start });
  return (
    <>
      <output data-testid="url">{serializeView(view)}</output>
      <CompareBody items={all} view={view} fromTray={tray} update={(next) => setView(parseView(new URLSearchParams(serializeView(next(view)))))} />
    </>
  );
}

const url = () => screen.getByTestId("url").textContent;
const comparison = () => screen.getByRole("table", { name: /^Comparison of/ });

describe("ui_16 compare page", () => {
  it("ui_16 asks for 2 to 4 protocols until two are chosen", () => {
    render(<Harness start={{ compare: ["kamino-lend"] }} />);
    expect(screen.getByText(/Choose 2 to 4 protocols/)).toBeTruthy();
  });

  it("ui_16 search adds a protocol to the comparison, kept in the URL", () => {
    render(<Harness start={{ compare: ["kamino-lend"] }} />);
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: PEER_NAME.slice(0, 4).toLowerCase() } });
    fireEvent.click(within(screen.getByRole("list", { name: "Matching protocols" })).getByRole("button", { name: PEER_NAME }));
    expect(url()).toBe(`p=kamino-lend%2C${PEER}`);
    const table = comparison();
    expect(within(table).getAllByRole("columnheader").map((h) => within(h).getAllByRole("link")[0]!.textContent)).toEqual(["Kamino Lend", PEER_NAME]);
  });

  it("ui_16 the tray's selection can be loaded", () => {
    render(<Harness start={{}} tray={[PEER, "kamino-lend"]} />);
    fireEvent.click(screen.getByRole("button", { name: "Use the compare tray (2)" }));
    expect(url()).toBe(`p=${PEER}%2Ckamino-lend`);
  });

  it("ui_16 four protocols side by side, every factor with its reason", () => {
    const k = items["kamino-lend"]!;
    const four = { ...items, c: { ...k, id: "c", name: "C" }, d: { ...k, id: "d", name: "D" } };
    render(<Harness all={four} start={{ compare: ["kamino-lend", PEER, "c", "d"] }} />);
    const table = comparison();
    expect(within(table).getAllByRole("columnheader")).toHaveLength(4);
    const row = within(table)
      .getAllByRole("rowheader")
      .find((h) => h.textContent === "RiskUpgrade control")!
      .closest("tr")!;
    expect(row.querySelectorAll("[data-rating]")).toHaveLength(4);
    expect((screen.getByRole("searchbox") as HTMLInputElement).disabled).toBe(true);
  });
});

describe("ui_17 differences only", () => {
  it("ui_17 two identical setups with the toggle on show an empty-state message", () => {
    const k = items["kamino-lend"]!;
    render(<Harness all={{ ...items, twin: { ...k, id: "twin", name: "Twin" } }} start={{ compare: ["kamino-lend", "twin"] }} />);
    fireEvent.click(screen.getByRole("checkbox", { name: "Show differences only" }));
    expect(url()).toContain("diff=1");
    expect(screen.getByText(/match on every row/)).toBeTruthy();
    expect(within(comparison()).queryAllByRole("row")).toHaveLength(1);
  });

  it("ui_17 the toggle keeps only rows that differ", () => {
    render(<Harness start={{ compare: ["kamino-lend", PEER], diff: true }} />);
    const labels = within(comparison())
      .getAllByRole("rowheader")
      .map((h) => h.textContent);
    expect(labels.some((l) => l?.includes("Upgrade controller"))).toBe(true);
    expect(screen.queryByText(/match on every row/)).toBeNull();
  });
});

describe("ui_18 overlay charts", () => {
  // Three days of TVL, so the chart shows however much history the snapshot has.
  const withTvl = (c: CompareItem): CompareItem => ({
    ...c,
    tvlDays: ["2026-10-01", "2026-10-02", "2026-10-03"].map((date, i) => ({ date, usd: 1_000_000 * (i + 1) })),
  });

  it("ui_18 TVL and activity of the compared protocols on one chart each, lines named and dashed", () => {
    const k = withTvl(items["kamino-lend"]!);
    const other: CompareItem = { ...k, id: "other", name: "Other", tvlDays: k.tvlDays.map((d) => ({ ...d, usd: d.usd / 2 })) };
    render(<Harness all={{ ...items, "kamino-lend": k, other }} start={{ compare: ["kamino-lend", "other"] }} />);
    const charts = screen.getAllByRole("figure");
    expect(charts).toHaveLength(2);
    for (const c of charts) {
      expect(within(c).getByRole("list", { name: "Legend" }).textContent).toContain("Kamino Lend");
      const labels = [...c.querySelectorAll("svg text")].map((t) => t.textContent);
      expect(labels).toEqual(expect.arrayContaining(["Kamino Lend", "Other"]));
    }
    expect(within(charts[0]!).getByRole("list", { name: "Legend" }).textContent).not.toContain("no data");
    const dashes = [...charts[0]!.querySelectorAll("svg path[stroke]")].map((p) => p.getAttribute("stroke-dasharray"));
    expect(new Set(dashes).size).toBe(2);
  });

  it("ui_18 the tooltip reads each protocol's value for a day by keyboard", () => {
    const k = items["kamino-lend"]!;
    render(<Harness all={{ ...items, other: { ...k, id: "other", name: "Other" } }} start={{ compare: ["kamino-lend", "other"] }} />);
    const figure = screen.getAllByRole("figure")[0]!;
    act(() => {
      fireEvent.keyDown(within(figure).getByRole("img"), { key: "ArrowRight" });
    });
    const tip = within(figure).getByRole("status");
    expect(tip.textContent).toContain("Kamino Lend");
    expect(tip.textContent).toContain("Other");
  });

  it("ui_18 a protocol without data is named as such in the legend", () => {
    const k = withTvl(items["kamino-lend"]!);
    render(
      <Harness all={{ ...items, "kamino-lend": k, empty: { ...k, id: "empty", name: "Empty", tvlDays: [] } }} start={{ compare: ["kamino-lend", "empty"] }} />,
    );
    expect(within(screen.getAllByRole("figure")[0]!).getByRole("list", { name: "Legend" }).textContent).toContain("Empty (no data)");
  });
});
