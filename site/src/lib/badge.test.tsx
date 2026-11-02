import { cleanup, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProtocolSections } from "@/components/protocol/Sections";
import { buildProtocolPage } from "./protocol";
import { GET, generateStaticParams } from "@/app/badges/[file]/route";
import { badgeSvg, embedSnippets } from "./badge";
import { loadSnapshot } from "./data";
import { STAGE_COLORS, STAGE_LABELS } from "./labels";
import { contrast } from "./tokens";
import type { Stage } from "./types";

const STAGES: Stage[] = ["unrated", "stage_0", "stage_1", "stage_2"];

describe("api_4 Stage badges", () => {
  it("api_4 a badge shows the protocol's Stage, readable and labelled", () => {
    const svg = badgeSvg("Kamino Lend", "stage_0");
    expect(svg.startsWith("<svg")).toBe(true);
    expect(svg).toContain('role="img"');
    expect(svg).toContain('aria-label="Kamino Lend on Celebtir: Stage 0"');
    expect(svg).toContain("<title>Kamino Lend on Celebtir: Stage 0</title>");
    expect(svg).toContain(">Stage 0<");
    expect(svg).toContain(STAGE_COLORS.stage_0);
  });

  it("api_4 the Stage text clears 4.5:1 on every Stage colour", () => {
    for (const st of STAGES) {
      const svg = badgeSvg("X", st);
      const fill = /<text[^>]*fill="(#[0-9a-f]{6})"[^>]*>[^<]*(Stage|Unrated)/i.exec(svg)![1]!;
      expect(contrast(fill, STAGE_COLORS[st]), st).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("api_4 the badge follows the latest snapshot: one per protocol, rebuilt every day", async () => {
    const s = loadSnapshot();
    expect(generateStaticParams().map((p) => p.file)).toEqual(Object.keys(s.facts.protocols).sort().map((id) => `${id}.svg`));
    const res = await GET(new Request("http://x/"), { params: Promise.resolve({ file: "kamino-lend.svg" }) });
    expect(res.headers.get("content-type")).toBe("image/svg+xml");
    const stage = s.scores.protocols["kamino-lend"]!.stage.stage;
    expect(await res.text()).toContain(`>${STAGE_LABELS[stage]}<`);
  });

  it("api_4 the embed snippet links the badge and the protocol page, as HTML and Markdown", () => {
    const e = embedSnippets("https://celebtir.example", "kamino-lend", "Kamino Lend", "stage_0");
    expect(e.html).toBe(
      '<a href="https://celebtir.example/protocols/kamino-lend/"><img src="https://celebtir.example/badges/kamino-lend.svg" alt="Kamino Lend on Celebtir: Stage 0"></a>',
    );
    expect(e.markdown).toBe(
      "[![Kamino Lend on Celebtir: Stage 0](https://celebtir.example/badges/kamino-lend.svg)](https://celebtir.example/protocols/kamino-lend/)",
    );
  });

  it("api_4 names are escaped inside the SVG", () => {
    expect(badgeSvg('A <b>&"', "stage_0")).toContain("A &lt;b&gt;&amp;&quot;");
  });

  it("api_4 the protocol page offers the badge with copyable HTML and Markdown snippets", () => {
    const s = loadSnapshot();
    const page = buildProtocolPage(s, "kamino-lend", undefined, []);
    render(<ProtocolSections page={page} siteUrl="https://celebtir.example" />);
    expect(screen.getByRole("img", { name: "Kamino Lend on Celebtir: Stage 0" }).getAttribute("src")).toBe("/badges/kamino-lend.svg");
    const e = embedSnippets("https://celebtir.example", "kamino-lend", "Kamino Lend", "stage_0");
    expect(screen.getByRole("button", { name: `Copy ${e.html}` })).toBeTruthy();
    expect(screen.getByRole("button", { name: `Copy ${e.markdown}` })).toBeTruthy();
    cleanup();
  });
});
