import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import AboutPage from "@/app/about/page";
import ApiPage from "@/app/api/page";
import ControllersPage from "@/app/controllers/page";
import TokenPage from "@/app/tokens/[mint]/page";
import ValuePage from "@/app/value/page";
import { ProtocolSections } from "@/components/protocol/Sections";
import { loadSnapshot } from "@/lib/data";
import { nameOf, peerOfKamino } from "@/lib/testSnapshot";
import { buildProtocolPage } from "@/lib/protocol";
import GlossaryPage from "@/app/glossary/page";
import MethodologyPage from "@/app/methodology/page";
import { ChangelogBody } from "./ChangelogView";
import { Footer } from "./Footer";
import { serializeLog, type LogView } from "@/lib/changelog";
import { GLOSSARY } from "@/lib/glossary";
import { loadMethodology } from "@/lib/methodology";
import { formatDuration } from "@/lib/format";
import type { ChangeEvent } from "@/lib/types";

// The Value page keeps its chart range in the URL; the app router is not mounted in tests.
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => new URLSearchParams(),
}));

afterEach(cleanup);

/** Text a reader sees: tooltips' hidden definitions left out. */
function visible(selector = "body"): string {
  return [...document.querySelectorAll(selector)]
    .map((el) => {
      const c = el.cloneNode(true) as HTMLElement;
      c.querySelectorAll('[role="tooltip"]').forEach((t) => t.remove());
      return c.textContent ?? "";
    })
    .join(" ");
}

describe("lch_2 notices on every route", () => {
  it("lch_2 the footer says: not investment advice, configuration only, may contain errors, how to report one", () => {
    render(<Footer />);
    const footer = screen.getByRole("contentinfo");
    expect(footer.textContent).toContain("Not investment advice.");
    expect(footer.textContent).toContain("Ratings describe on-chain configuration only");
    expect(footer.textContent).toContain("may contain errors");
    expect(within(footer).getByRole("link", { name: "How to report one" }).getAttribute("href")).toMatch(/^\/about\/?#corrections$/);
  });

  it("lch_2 the About page states the same notice", () => {
    render(<AboutPage />);
    expect(visible()).toContain("Not investment advice. Ratings describe on-chain configuration only and may contain errors.");
  });
});

describe("lch_4 corrections policy", () => {
  it("lch_4 the About page publishes 72-hour acknowledgement, 7-day resolution and a public log", () => {
    render(<AboutPage />);
    const policy = screen.getByRole("heading", { name: "Corrections policy" }).parentElement!;
    expect(policy.textContent).toContain("acknowledged within 72 hours");
    expect(policy.textContent).toContain("resolved within 7 days");
    expect(policy.textContent).toContain("logged publicly");
  });
});

describe("lch_6 privacy", () => {
  it("lch_6 the About page says there are no cookies and no personal data", () => {
    render(<AboutPage />);
    expect(visible()).toContain("No cookies, no analytics, no accounts and no personal data.");
  });
});

describe("ui_6 glossary", () => {
  it("ui_6 the glossary page lists every term with an anchor", () => {
    render(<GlossaryPage />);
    for (const t of GLOSSARY) expect(document.getElementById(t.slug), t.slug).not.toBeNull();
  });

  it("ui_6 on the methodology page every glossary term used in its prose is linked once, with a definition", () => {
    render(<MethodologyPage />);
    const prose = visible("p, li, td");
    for (const t of GLOSSARY) {
      const words = [t.term, ...(t.aliases ?? [])];
      const used = words.some((w) => new RegExp(`\\b${w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "i").test(prose));
      const links = document.querySelectorAll(`a:is([href="/glossary/#${t.slug}"], [href="/glossary#${t.slug}"])`);
      if (used) {
        expect(links.length, `${t.term} is used but not linked`).toBe(1);
        const tip = document.getElementById(links[0]!.getAttribute("aria-describedby")!)!;
        expect(tip.textContent).toBe(t.definition);
      }
    }
  });

  it("ui_6 a term's definition opens on keyboard focus", () => {
    render(<AboutPage />);
    const link = document.querySelector('a:is([href="/glossary/#multisig"], [href="/glossary#multisig"])') as HTMLElement;
    const tip = document.getElementById(link.getAttribute("aria-describedby")!)!;
    expect(tip.hidden).toBe(true);
    fireEvent.focus(link);
    expect(tip.hidden).toBe(false);
  });
});

describe("methodology page", () => {
  it("shows the thresholds from the methodology file and its changelog", () => {
    const m = loadMethodology("0.1");
    render(<MethodologyPage />);
    const text = visible();
    expect(text).toContain(`${m.upgrade_control.low_min_voters}+ signers at a ${m.upgrade_control.low_min_threshold_pct}%+ threshold`);
    expect(text).toContain(`${formatDuration(m.upgrade_delay.low_min_secs)} or more`);
    expect(screen.getByRole("heading", { name: "Changelog" })).toBeTruthy();
    expect(text).toContain("First published version.");
    expect(text).toContain("cannot return an account's past state");
    expect(text).toContain("simplification");
  });
});

describe("api page", () => {
  it("documents every data file, the versioning rule, CORS and the licence", () => {
    render(<ApiPage />);
    const text = document.body.textContent!;
    for (const p of ["/data/v2/summary.json", "/data/v2/protocols/<id>.json", "/data/v2/tokens/<mint>.json", "/data/v2/events.json"]) {
      expect(text).toContain(p);
    }
    expect(text).not.toContain("/data/v1/");
    expect(text).toContain("assessment");
    expect(text).toContain("Access-Control-Allow-Origin");
    expect(text).toContain("CC BY 4.0");
  });
});

describe("change log", () => {
  const ev = (id: string, severity: ChangeEvent["severity"], protocol: string): ChangeEvent => ({
    id,
    date: "2026-10-17",
    detected_within_days: 2,
    protocol,
    subject: "x",
    kind: "controller_changed",
    severity,
    source: "on_chain",
    summary: `summary ${id}`,
    signature: "5sig",
  });
  const events = [ev("a", "critical", "kamino-lend"), ev("b", "info", "spl-stake-pool")];
  const names = { "kamino-lend": "Kamino Lend", "spl-stake-pool": "SPL Stake Pool" };

  function Harness({ onUrl }: { onUrl: (q: string) => void }) {
    const [view, setView] = useState<LogView>({ severity: [], source: [], protocol: null });
    return (
      <ChangelogBody
        events={events}
        names={names}
        feeds={["kamino-lend"]}
        view={view}
        update={(v) => {
          onUrl(serializeLog(v));
          setView(v);
        }}
      />
    );
  }

  it("lists events newest first with protocol, severity, source, detection window and transaction", () => {
    render(<Harness onUrl={() => {}} />);
    const list = screen.getByRole("list", { name: "Events, newest first" });
    expect(within(list).getAllByRole("listitem")).toHaveLength(2);
    expect(list.textContent).toContain("detected within 2 days");
    expect(within(list).getAllByRole("link", { name: "transaction" })[0]!.getAttribute("href")).toBe("https://solscan.io/tx/5sig");
  });

  it("filters by severity and protocol in the URL, and offers that protocol's feed", () => {
    let url = "";
    render(<Harness onUrl={(q) => (url = q)} />);
    fireEvent.click(screen.getByRole("checkbox", { name: "Critical" }));
    expect(url).toBe("severity=critical");
    expect(screen.getAllByRole("listitem")).toHaveLength(1);
    fireEvent.change(screen.getByRole("combobox", { name: "Protocol" }), { target: { value: "kamino-lend" } });
    expect(url).toBe("severity=critical&protocol=kamino-lend");
    expect(screen.getByRole("link", { name: /Subscribe with Atom/ }).getAttribute("href")).toBe("/feeds/kamino-lend.atom");
  });

  it("says so when there are no events yet", () => {
    render(<ChangelogBody events={[]} names={names} feeds={[]} view={{ severity: [], source: [], protocol: null }} update={() => {}} />);
    expect(screen.getByText(/No transition records available/)).toBeTruthy();
  });
});

describe("ui_6 glossary terms across pages", () => {
  /**
   * Glossary terms used in a page's prose that are not linked exactly once.
   * Prose is paragraphs and sentence lists (risk summaries); data lines such
   * as evidence rows, badges, stats and table cells are not prose (UI-6).
   */
  // Next Link normalizes trailing slashes in jsdom; both spellings retain the exact glossary target.
  function unlinked(): string[] {
    const prose = visible("main p:not([data-empty]), main [data-prose] li");
    return GLOSSARY.filter((t) => {
      const used = [t.term, ...(t.aliases ?? [])].some((w) => new RegExp(`\\b${w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "i").test(prose));
      return used && document.querySelectorAll(`main a:is([href="/glossary/#${t.slug}"], [href="/glossary#${t.slug}"])`).length !== 1;
    }).map((t) => t.term);
  }

  const pages: [string, () => Promise<React.ReactNode> | React.ReactNode][] = [
    ["about", () => <AboutPage />],
    ["methodology", () => <MethodologyPage />],
    ["value", () => <ValuePage />],
    ["controllers", () => <ControllersPage />],
    ["token USDC", async () => TokenPage({ params: Promise.resolve({ mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" }) })],
    ["protocol Kamino", () => <ProtocolSections page={buildProtocolPage(loadSnapshot(), "kamino-lend", undefined, [])} />],
    [
      `protocol ${nameOf(loadSnapshot(), peerOfKamino(loadSnapshot()))}`,
      () => <ProtocolSections page={buildProtocolPage(loadSnapshot(), peerOfKamino(loadSnapshot()), undefined, [])} />,
    ],
  ];

  for (const [name, make] of pages) {
    it(`ui_6 every glossary term in the ${name} page's prose is linked once`, async () => {
      render(<main>{await make()}</main>);
      expect(unlinked()).toEqual([]);
    });
  }
});

describe("hui_5 methodology covers Hyperliquid", () => {
  it("hui_5 the page explains HyperEVM sources, the HIP-3 anchor block and why HIP-3 is not rated", () => {
    render(<MethodologyPage />);
    for (const id of ["hyperliquid", "hyperevm-sources", "hip3-anchor", "hip3-unrated"]) {
      expect(document.getElementById(id), id).not.toBeNull();
    }
    const text = visible();
    expect(text).toContain("EIP-1967 admin slot");
    expect(text).toContain("anchored to the HyperEVM block");
    expect(text).toContain("without ratings or a Stage");
  });
});
