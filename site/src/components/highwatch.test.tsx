import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EvidenceList } from "./Evidence";
import { StackedBar } from "./ui/Charts";
import type { Evidence } from "@/lib/types";

vi.mock("next/navigation", () => ({ useSearchParams: () => new URLSearchParams("chain=hyperliquid") }));
afterEach(cleanup);

describe("Highwatch evidence integrity", () => {
  it("preserves distinct Solana, HyperEVM and HIP-3 observation anchors", () => {
    const evidence: Evidence[] = ["solana", "hyper_evm", "hyper_core"].map(chain => ({
      chain, address: "recorded-account", slot: 123456, field: "authority",
      explorer_url: `https://example.org/${chain}/recorded-account`,
    }));
    render(<EvidenceList evidence={evidence} />);
    expect(screen.getByText("Solana slot 123,456")).toBeTruthy();
    expect(screen.getByText("HyperEVM block 123,456")).toBeTruthy();
    expect(screen.getByText("HyperEVM anchor block 123,456")).toBeTruthy();
    expect(screen.getAllByRole("link").map(a => a.getAttribute("href"))).toEqual(evidence.map(e => e.explorer_url));
  });

  it("exposure links preserve controller, chain and the leaderboard anchor", () => {
    render(<StackedBar unit="share" label="Exposure" segments={[
      { key: "immutable", label: "Immutable", value: 25, color: "#fff", href: "/?controller=immutable#protocols-heading" },
      { key: "single_key", label: "Single key", value: 75, color: "#aaa", href: "/?controller=single_key#protocols-heading" },
    ]} />);
    const link = screen.getByRole("link", { name: "Immutable: 25, 25%. Explore protocols." });
    expect(link.getAttribute("href")).toBe("/?controller=immutable&chain=hyperliquid#protocols-heading");
    expect(screen.getByRole("link", { name: "Immutable" })).toBeTruthy();
  });
});
