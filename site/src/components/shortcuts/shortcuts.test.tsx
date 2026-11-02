import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import ActivityPage from "@/app/activity/page";
import TokensPage from "@/app/tokens/page";
import ValuePage from "@/app/value/page";
import { Address, EvidenceList, TxLink } from "@/components/Evidence";
import { CompareTray } from "@/components/compare/CompareTray";
import { RiskViewBody } from "@/components/page/RiskView";
import { ProtocolTableView } from "@/components/ProtocolTable";
import { Shortcuts } from "./Shortcuts";
import { SiteSearch } from "./SiteSearch";
import { searchIndex, type SearchItem } from "@/lib/search";
import { loadHistory, loadSnapshot } from "@/lib/data";
import { factorRows } from "@/lib/pages";
import { accountUrl, txUrl } from "@/lib/solscan";
import { buildRows, categoryCounts } from "@/lib/summary";
import { FACTORS, type ChangeEvent, type Factor, type ProtocolHistoryFile } from "@/lib/types";
import { DEFAULT_VIEW } from "@/lib/viewState";
import { peerOfKamino } from "@/lib/testSnapshot";

// The protocol compared with Kamino Lend in the snapshot being tested.
const PEER = peerOfKamino(loadSnapshot());


// The app router is not mounted in tests; SiteSearch only needs push.
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => new URLSearchParams(),
}));

const ADDR = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

beforeEach(() => window.localStorage.clear());
afterEach(() => {
  cleanup();
  window.localStorage.clear();
});

const press = (key: string, init: KeyboardEventInit = {}, target: Element = document.body) =>
  act(() => {
    fireEvent.keyDown(target, { key, ...init });
  });

const s = loadSnapshot();
const ids = Object.keys(s.scores.protocols);
const rows = buildRows(
  s,
  Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])),
  loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [],
);
const items: SearchItem[] = [
  ...rows.map((r) => ({ label: r.name, href: `/protocols/${r.id}/`, kind: "Protocol" as const })),
  { label: "USDC", href: "/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/", kind: "Token", detail: "Circle", keywords: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" },
];

function Chrome({ children }: { children: React.ReactNode }) {
  return (
    <>
      <SiteSearch items={items} />
      <Shortcuts />
      <main>{children}</main>
      <CompareTray names={Object.fromEntries(rows.map((r) => [r.id, r.name]))} />
    </>
  );
}

describe("ui_3 keyboard shortcuts", () => {
  it("ui_3 / focuses the page's own search, F opens its filters", () => {
    render(
      <Chrome>
        <ProtocolTableView rows={rows} counts={categoryCounts(rows)} view={DEFAULT_VIEW} update={() => {}} />
      </Chrome>,
    );
    press("/");
    expect(document.activeElement).toBe(screen.getByRole("searchbox", { name: "Search" }));
    (document.activeElement as HTMLElement).blur();
    press("f");
    const filters = document.querySelector('[data-shortcut="filters"]') as HTMLDetailsElement;
    expect(filters.open).toBe(true);
    expect(document.activeElement).toBe(filters.querySelector("summary"));
  });

  it("ui_3 shortcuts are ignored while typing or with a modifier key", () => {
    render(
      <Chrome>
        <ProtocolTableView rows={rows} counts={categoryCounts(rows)} view={DEFAULT_VIEW} update={() => {}} />
      </Chrome>,
    );
    const search = screen.getByRole("searchbox", { name: "Search" });
    search.focus();
    press("f", {}, search);
    expect((document.querySelector('[data-shortcut="filters"]') as HTMLDetailsElement).open).toBe(false);
    search.blur();
    press("f", { ctrlKey: true });
    expect((document.querySelector('[data-shortcut="filters"]') as HTMLDetailsElement).open).toBe(false);
  });

  it("ui_3 C goes to the compare tray, or says it is empty", () => {
    render(<Chrome>{null}</Chrome>);
    press("c");
    expect(screen.getByRole("status").textContent).toMatch(/compare tray is empty/i);
    cleanup();
    window.localStorage.setItem("celebtir.compare", JSON.stringify(["kamino-lend", PEER]));
    render(<Chrome>{null}</Chrome>);
    press("c");
    expect(document.activeElement).toBe(screen.getByRole("link", { name: "Compare 2" }));
  });

  it("ui_3 ? lists every shortcut, and Escape closes the list", () => {
    render(<Chrome>{null}</Chrome>);
    press("?", { shiftKey: true });
    const dialog = screen.getByRole("dialog", { name: "Keyboard shortcuts" });
    for (const k of ["/", "F", "C", "?"]) expect(dialog.textContent).toContain(k);
    press("Escape", {}, dialog);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("ui_3 Shift+/ opens the list even where it reports \"/\"", () => {
    render(<Chrome>{null}</Chrome>);
    press("/", { shiftKey: true });
    expect(screen.getByRole("dialog", { name: "Keyboard shortcuts" })).toBeTruthy();
  });

  it("ui_3 each shortcut works on every page with a table", async () => {
    const details = Object.fromEntries(FACTORS.map((f) => [f, factorRows(s, f)])) as Record<Factor, ReturnType<typeof factorRows>>;
    const pages: [string, React.ReactNode][] = [
      ["summary", <ProtocolTableView key="s" rows={rows} counts={categoryCounts(rows)} view={DEFAULT_VIEW} update={() => {}} />],
      ["risk", <RiskViewBody key="r" rows={rows} details={details} view={DEFAULT_VIEW} update={() => {}} />],
      ["value", <ValuePage key="v" />],
      ["activity", <ActivityPage key="a" />],
      ["tokens", <TokensPage key="t" />],
    ];
    for (const [name, page] of pages) {
      cleanup();
      render(<Chrome>{page}</Chrome>);
      press("/");
      expect(document.activeElement?.getAttribute("type"), `${name}: /`).toBe("search");
      (document.activeElement as HTMLElement).blur();
      press("f");
      const target = document.querySelector('[data-shortcut="filters"]');
      if (target) expect(target.contains(document.activeElement), `${name}: F`).toBe(true);
      else expect(screen.getByRole("status").textContent, `${name}: F`).toMatch(/no filters/i);
      press("c");
      expect(screen.getByRole("status").textContent, `${name}: C`).toMatch(/compare tray/i);
      press("?", { shiftKey: true });
      expect(screen.getByRole("dialog", { name: "Keyboard shortcuts" }), `${name}: ?`).toBeTruthy();
      press("Escape", {}, screen.getByRole("dialog"));
    }
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });
});

describe("site search", () => {
  it("ui_3 finds protocols and tokens by name and opens the first match on Enter", () => {
    const go = vi.fn();
    render(<SiteSearch items={items} navigate={go} />);
    const input = screen.getByRole("combobox", { name: "Search protocols, tokens and controllers" });
    fireEvent.change(input, { target: { value: "usd" } });
    const option = screen.getByRole("option", { name: /USDC/ });
    expect(option.getAttribute("data-href")).toMatch(/^\/tokens\/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\/?$/);
    expect(input.getAttribute("aria-activedescendant")).toBe(option.id);
    fireEvent.keyDown(input, { key: "Enter" });
    expect(go).toHaveBeenCalledWith("/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/");
  });

  it("ui_3 arrow keys move through grouped results and Escape clears", () => {
    const go = vi.fn();
    render(<SiteSearch items={searchIndex(s)} navigate={go} />);
    const input = screen.getByRole("combobox");
    fireEvent.change(input, { target: { value: "s" } });
    const options = screen.getAllByRole("option");
    expect(options.length).toBeGreaterThan(1);
    expect(screen.getAllByRole("group").map((g) => g.getAttribute("aria-label"))[0]).toBe("Protocols");
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(input.getAttribute("aria-activedescendant")).toBe(options[1]!.id);
    expect(options[1]!.getAttribute("aria-selected")).toBe("true");
    fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(input.getAttribute("aria-activedescendant")).toBe(options[0]!.id);
    fireEvent.keyDown(input, { key: "Escape" });
    expect((input as HTMLInputElement).value).toBe("");
    expect(input.getAttribute("aria-expanded")).toBe("false");
  });

  it("ui_3 the index finds a controller by address and says when nothing matches", () => {
    const index = searchIndex(s);
    const controller = index.find((i) => i.kind === "Controller");
    expect(controller).toBeTruthy();
    render(<SiteSearch items={index} navigate={vi.fn()} />);
    const input = screen.getByRole("combobox");
    fireEvent.change(input, { target: { value: controller!.keywords!.slice(0, 8) } });
    expect(screen.getAllByRole("option").some((o) => o.getAttribute("data-href") === controller!.href)).toBe(true);
    fireEvent.change(input, { target: { value: "zzzz-not-a-protocol" } });
    expect(screen.queryAllByRole("option")).toHaveLength(0);
    expect(screen.getByText(/Nothing in this snapshot matches/)).toBeTruthy();
  });
});

describe("ui_5 addresses and evidence links", () => {
  it("ui_5 an address shows shortened, its full value on hover, links to Solscan and copies", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    render(<Address address={ADDR} />);
    const link = screen.getByRole("link", { name: "KLen…gmjD" });
    expect(link.getAttribute("title")).toBe(ADDR);
    expect(link.getAttribute("href")).toBe(`https://solscan.io/account/${ADDR}`);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: `Copy ${ADDR}` }));
    });
    expect(writeText).toHaveBeenCalledWith(ADDR);
    expect(screen.getByRole("button", { name: `Copy ${ADDR}` }).textContent).toBe("Copied");
  });

  it("ui_5 a blocked clipboard says so instead of failing silently", async () => {
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText: vi.fn().mockRejectedValue(new Error("denied")) },
      configurable: true,
    });
    const exec = vi.fn().mockReturnValue(false);
    Object.defineProperty(document, "execCommand", { value: exec, configurable: true });
    render(<Address address={ADDR} />);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: `Copy ${ADDR}` }));
    });
    expect(exec).toHaveBeenCalledWith("copy");
    expect(screen.getByRole("button", { name: `Copy ${ADDR}` }).textContent).toBe("Copy blocked");
  });

  it("ui_5 transaction links open Solscan's transaction page and copy the signature", () => {
    render(<TxLink signature="5sigABCDEFGHIJKLMNOP" />);
    expect(screen.getByRole("link").getAttribute("href")).toBe("https://solscan.io/tx/5sigABCDEFGHIJKLMNOP");
    expect(screen.getByRole("button", { name: "Copy 5sigABCDEFGHIJKLMNOP" })).toBeTruthy();
  });

  it("ui_5 every evidence link in the published data resolves to the right Solscan account page", () => {
    const all: { address: string; explorer_url: string; chain?: string }[] = [];
    const walk = (v: unknown) => {
      if (Array.isArray(v)) v.forEach(walk);
      else if (v && typeof v === "object") {
        const o = v as Record<string, unknown>;
        if (typeof o.explorer_url === "string" && typeof o.address === "string") all.push(o as never);
        Object.values(o).forEach(walk);
      }
    };
    walk(s.facts);
    walk(s.scores);
    expect(all.length).toBeGreaterThan(20);
    for (const e of all) expect(e.explorer_url, e.address).toBe(accountUrl(e.address, e.chain));
    expect(txUrl("abc")).toBe("https://solscan.io/tx/abc");
  });

  it("ui_5 evidence lists render each account through the copyable address", () => {
    render(<EvidenceList evidence={[{ chain: "solana", address: ADDR, slot: 1, field: "owner", explorer_url: accountUrl(ADDR) }]} />);
    expect(screen.getByRole("button", { name: `Copy ${ADDR}` })).toBeTruthy();
  });
});
