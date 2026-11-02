import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChainScoped } from "./ChainScoped";
import { ScopeEmpty } from "./ScopeEmpty";
import { ChainSelector } from "../shell/ChainSelector";
import { MainNavigation } from "../shell/MainNavigation";

const nav = vi.hoisted(() => ({ search: "", path: "/" }));
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  usePathname: () => nav.path,
  useSearchParams: () => new URLSearchParams(nav.search),
}));

// next/link drops the trailing slash under jsdom; the export adds it back.
const href = (a: HTMLElement) => a.getAttribute("href")!.replace(/\/(\?|$)/, "$1");

afterEach(() => {
  cleanup();
  nav.search = "";
  nav.path = "/";
});

const variants = { all: <p>every chain</p>, solana: <p>only solana</p>, hyperliquid: <p>only hyperliquid</p> };

describe("hui_1 ChainScoped shows the variant the picker chose", () => {
  for (const [search, text] of [
    ["", "every chain"],
    ["chain=solana", "only solana"],
    ["chain=hyperliquid", "only hyperliquid"],
    ["chain=bogus", "every chain"],
  ] as const) {
    it(`hui_1 ?${search || "(none)"} shows "${text}"`, () => {
      nav.search = search;
      render(<ChainScoped variants={variants} />);
      expect(screen.getByText(text)).toBeTruthy();
      expect(document.body.textContent).toBe(text);
    });
  }
});

describe("hui_1 ScopeEmpty", () => {
  it("hui_1 names the chain and links Hyperliquid to the HIP-3 page", () => {
    render(<ScopeEmpty chain="hyperliquid" kind="protocols" />);
    expect(screen.getByText(/No Hyperliquid protocols in this snapshot yet/)).toBeTruthy();
    expect(href(screen.getByRole("link", { name: "HIP-3 DEXs" }))).toBe("/hip3");
    cleanup();
    render(<ScopeEmpty chain="solana" kind="tokens" />);
    expect(screen.getByText(/No Solana tokens in this snapshot yet/)).toBeTruthy();
    expect(screen.queryByRole("link")).toBeNull();
  });
});

describe("hui_1 the picker and the nav", () => {
  it("hui_1 a protocol page fixes the picker to its chain, with a hint", () => {
    nav.path = "/protocols/kamino-lend/";
    nav.search = "chain=hyperliquid";
    render(<ChainSelector protocolChains={{ "kamino-lend": "solana" }} />);
    const group = screen.getByRole("group", { name: "Chain" });
    const solana = within(group).getByRole("button", { name: "Solana" });
    expect(solana.getAttribute("aria-pressed")).toBe("true");
    expect(solana.getAttribute("aria-disabled")).toBe("true");
    expect(group.getAttribute("data-fixed")).toBe("solana");
    expect(screen.getByText(/one Solana entity, so the chain is fixed/)).toBeTruthy();
  });

  it("hui_1 other pages follow the URL", () => {
    nav.path = "/risk/";
    nav.search = "chain=hyperliquid";
    render(<ChainSelector />);
    expect(screen.getByRole("button", { name: "Hyperliquid" }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("group", { name: "Chain" }).getAttribute("data-fixed")).toBeNull();
  });

  it("hui_1 nav links keep the chain, and HIP-3 hides under Solana", () => {
    render(<MainNavigation chain="hyperliquid" />);
    const links = within(screen.getByRole("navigation", { name: "Main" })).getAllByRole("link");
    expect(href(links.find((a) => a.textContent === "Risk Analysis")!)).toBe("/risk?chain=hyperliquid");
    expect(links.some((a) => a.textContent === "HIP-3")).toBe(true);
    cleanup();
    render(<MainNavigation chain="solana" />);
    const sol = within(screen.getByRole("navigation", { name: "Main" })).getAllByRole("link");
    expect(sol.some((a) => a.textContent === "HIP-3")).toBe(false);
    cleanup();
    render(<MainNavigation />);
    const all = within(screen.getByRole("navigation", { name: "Main" })).getAllByRole("link");
    expect(href(all.find((a) => a.textContent === "Risk Analysis")!)).toBe("/risk");
  });
});
