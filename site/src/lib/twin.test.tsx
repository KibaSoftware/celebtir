import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { GET as llmsGet } from "@/app/llms.txt/route";
import { GET as twinGet, generateStaticParams } from "@/app/twins/[file]/route";
import { OgCard } from "@/components/OgCard";
import { loadHistory, loadSnapshot } from "./data";
import { llmsTxt } from "./llms";
import { buildProtocolPage } from "./protocol";
import { protocolMarkdown } from "./twin";
import { FACTORS, type ChangeEvent, type ProtocolHistoryFile } from "./types";
import { ProtocolSections, SECTIONS } from "@/components/protocol/Sections";
import { Footer } from "@/components/reference/Footer";

afterEach(cleanup);

const s = loadSnapshot();
const page = (id: string) =>
  buildProtocolPage(s, id, loadHistory<ProtocolHistoryFile>(id), loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? []);

describe("ui_14 Markdown twin", () => {
  const md = protocolMarkdown(page("kamino-lend"), { snapshotDate: "2026-10-17", methodology: "0.1" });

  it("ui_14 has every section of the protocol page, in order", () => {
    const headings = [...md.matchAll(/^## (.+)$/gm)].map((m) => m[1]);
    expect(headings).toEqual(SECTIONS.map((x) => x.title));
    expect(md.startsWith("# Kamino Lend\n")).toBe(true);
  });

  it("ui_14 carries the ratings with reasons and Solscan evidence, and the notices", () => {
    expect(md).toContain("- Upgrade control: **Medium**");
    expect(md).toMatch(/https:\/\/solscan\.io\/account\/[1-9A-HJ-NP-Za-km-z]{32,44}/);
    expect(md).toContain("Not investment advice.");
    expect(md).toContain("CC BY 4.0");
    expect(md).toContain("Snapshot 2026-10-17, methodology v0.1");
  });

  it("ui_14 is deterministic and says why an empty section is empty", () => {
    expect(protocolMarkdown(page("kamino-lend"), { snapshotDate: "2026-10-17", methodology: "0.1" })).toBe(md);
    const empty = { ...page("kamino-lend"), events: [] };
    expect(protocolMarkdown(empty, { snapshotDate: "2026-10-17", methodology: "0.1" })).toContain("No changes have been recorded for this protocol since tracking began.");
  });

  it("ui_14 is generated at build as /protocols/<id>.md for every protocol", async () => {
    const params = generateStaticParams();
    expect(params.map((p) => p.file)).toEqual(Object.keys(s.facts.protocols).sort().map((id) => `${id}.md`));
    const res = await twinGet(new Request("http://x/"), { params: Promise.resolve({ file: "kamino-lend.md" }) });
    expect(res.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(await res.text()).toContain("# Kamino Lend");
  });

  it("ui_14 llms.txt lists every protocol's Markdown twin and the reference pages", async () => {
    const txt = llmsTxt(s);
    expect(txt.startsWith("# Celebtir\n\n> ")).toBe(true);
    for (const id of Object.keys(s.scores.protocols)) expect(txt).toContain(`(/protocols/${id}.md)`);
    for (const p of ["/methodology/", "/glossary/", "/about/", "/api/"]) expect(txt).toContain(`(${p})`);
    const res = await llmsGet();
    expect(res.headers.get("content-type")).toBe("text/plain; charset=utf-8");
    expect(await res.text()).toBe(txt);
  });
});

describe("ui_14 links", () => {
  it("ui_14 the protocol page links its Markdown twin at the bottom, and the site footer links llms.txt", () => {
    render(<ProtocolSections page={page("kamino-lend")} />);
    expect(screen.getByRole("link", { name: "Markdown version" }).getAttribute("href")).toBe("/protocols/kamino-lend.md");
    cleanup();
    render(<Footer />);
    expect(screen.getByRole("link", { name: "llms.txt" }).getAttribute("href")).toBe("/llms.txt");
  });
});

describe("ui_15 Open Graph card", () => {
  it("ui_15 shows the name, Stage and the six factor ratings, each in words", () => {
    const p = page("kamino-lend");
    render(<OgCard page={p} />);
    expect(screen.getByText("Kamino Lend")).toBeTruthy();
    expect(screen.getByText("Stage 0")).toBeTruthy();
    for (const f of FACTORS) expect(document.querySelector(`[data-factor="${f}"]`)?.textContent).toMatch(/Low|Medium|High|Unknown|Not applicable/);
  });
});
