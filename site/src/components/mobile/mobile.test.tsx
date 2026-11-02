import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { readFileSync } from "node:fs";
import path from "node:path";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CompareBody } from "@/components/compare/CompareView";
import { TableLabels } from "./TableLabels";
import { compareItems, type CompareItem } from "@/lib/compare";
import { loadHistory, loadSnapshot } from "@/lib/data";
import type { ProtocolHistoryFile } from "@/lib/types";
import { DEFAULT_VIEW, parseView, serializeView, type ViewState } from "@/lib/viewState";
import { nameOf, peerOfKamino } from "@/lib/testSnapshot";

// The protocol compared with Kamino Lend in the snapshot being tested.
const PEER = peerOfKamino(loadSnapshot());
const PEER_NAME = nameOf(loadSnapshot(), PEER);


afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** jsdom has no layout: say whether the phone media query matches. */
function phone(matches: boolean) {
  const listeners = new Set<() => void>();
  vi.stubGlobal("matchMedia", (q: string) => ({
    matches: q.includes("max-width") ? matches : false,
    media: q,
    addEventListener: (_: string, l: () => void) => listeners.add(l),
    removeEventListener: (_: string, l: () => void) => listeners.delete(l),
  }));
}

describe("ui_25 tables collapse to cards", () => {
  it("ui_25 every body cell gets its column's name as a label for the card layout", async () => {
    render(
      <main>
        <TableLabels />
        <table>
          <thead>
            <tr>
              <th>Protocol</th>
              <th>
                <button>TVL ↕</button>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th>Kamino</th>
              <td>$1B</td>
            </tr>
          </tbody>
        </table>
      </main>,
    );
    const cells = screen.getAllByRole("row")[1]!.children;
    // Labelling runs once the page is idle, after hydration.
    await waitFor(() => expect(cells[0]!.getAttribute("data-label")).toBe("Protocol"));
    expect(cells[1]!.getAttribute("data-label")).toBe("TVL");
  });

  it("ui_25 rows added later get labels too", async () => {
    function Grow() {
      const [n, setN] = useState(1);
      return (
        <main>
          <TableLabels />
          <button onClick={() => setN(n + 1)}>more</button>
          <table>
            <thead>
              <tr>
                <th>Name</th>
              </tr>
            </thead>
            <tbody>
              {Array.from({ length: n }, (_, i) => (
                <tr key={i}>
                  <td>row {i}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </main>
      );
    }
    render(<Grow />);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "more" }));
    });
    await waitFor(() => expect(screen.getByText("row 1").getAttribute("data-label")).toBe("Name"));
  });

  it("ui_25 the stylesheet turns tables into cards below 640 px, keeping sort buttons visible", () => {
    const css = readFileSync(path.resolve("src/app/globals.css"), "utf8");
    const mobile = css.slice(css.indexOf("@media (max-width: 640px)"));
    expect(mobile).toContain('main table:not([data-cards="off"])');
    expect(mobile).toContain("attr(data-label)");
    expect(mobile).toMatch(/thead:has\(button\) tr \{[^}]*overflow-x: auto/);
  });
});

describe("ui_25 compare shows 2 protocols at a time on a phone", () => {
  const s = loadSnapshot();
  const ids = Object.keys(s.scores.protocols);
  const items = compareItems(s, Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])));
  const k = items["kamino-lend"]!;
  const four: Record<string, CompareItem> = { ...items, c: { ...k, id: "c", name: "C" }, d: { ...k, id: "d", name: "D" } };

  function Harness() {
    const [view, setView] = useState<ViewState>({ ...DEFAULT_VIEW, compare: ["kamino-lend", PEER, "c", "d"] });
    return <CompareBody items={four} view={view} fromTray={[]} update={(n) => setView(parseView(new URLSearchParams(serializeView(n(view)))))} />;
  }
  const headers = () =>
    within(screen.getByRole("table", { name: /^Comparison of/ }))
      .getAllByRole("columnheader")
      .map((h) => within(h).getAllByRole("link")[0]!.textContent);

  it("ui_25 on a phone, two columns with a pager", () => {
    phone(true);
    render(<Harness />);
    expect(headers()).toEqual(["Kamino Lend", PEER_NAME]);
    expect(screen.getByText("Showing 1–2 of 4")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Next two" }));
    expect(headers()).toEqual(["C", "D"]);
    expect((screen.getByRole("button", { name: "Next two" }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Previous two" }));
    expect(headers()).toEqual(["Kamino Lend", PEER_NAME]);
  });

  it("ui_25 differences-only still compares all four, not just the two shown", () => {
    phone(true);
    render(<Harness />);
    fireEvent.click(screen.getByRole("checkbox", { name: "Show differences only" }));
    // C and D copy Kamino, but the peer differs, so rows remain.
    expect(screen.queryByText(/match on every row/)).toBeNull();
  });

  it("ui_25 on a desktop, all four side by side and no pager", () => {
    phone(false);
    render(<Harness />);
    expect(headers()).toHaveLength(4);
    expect(screen.queryByText(/^Showing/)).toBeNull();
  });
});
