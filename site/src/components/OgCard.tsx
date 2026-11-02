import { FACTOR_LABELS, STAGE_COLORS, STAGE_LABELS } from "@/lib/labels";
import type { ProtocolPage } from "@/lib/protocol";
import { ink, ratingStyle, surfaces, type Rating } from "@/lib/tokens";
import { FACTORS } from "@/lib/types";
import { BRAND_HEADLINE, BRAND_SUPPORTING_LINE } from "@/lib/site";

export const OG_SIZE = { width: 1200, height: 630 };

/**
 * A drawn shape per rating, so the card reads without colour (UI-24): Low a
 * circle, Medium a diamond, High a square, Unknown a dashed square, Not
 * applicable a bar. Shapes, not glyphs: the bundled font has no ✓ or ✕, and
 * a missing glyph would make next/og fetch a font.
 */
function Mark({ rating }: { rating: Rating }) {
  const color = ratingStyle[rating].color;
  const box = { display: "flex", width: 34, height: 34, marginRight: 22 } as const;
  switch (rating) {
    case "low":
      return <div style={{ ...box, borderRadius: 17, background: color }} />;
    case "medium":
      return <div style={{ ...box, width: 28, height: 28, margin: "3px 25px 3px 3px", background: color, transform: "rotate(45deg)" }} />;
    case "high":
      return <div style={{ ...box, borderRadius: 4, background: color }} />;
    case "unknown":
      return <div style={{ ...box, borderRadius: 4, border: `4px dashed ${color}` }} />;
    case "not_applicable":
      return <div style={{ ...box, height: 6, margin: "14px 22px 14px 0", background: color }} />;
  }
}

/**
 * UI-15: the protocol's share card (1200 × 630): name, Stage and the six
 * factor ratings. Inline styles and flex only, as `next/og` requires.
 */
export function OgCard({ page }: { page: ProtocolPage }) {
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        background: surfaces.page,
        color: ink.primary,
        padding: "56px 64px",
        fontSize: 28,
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", color: ink.muted, fontSize: 26 }}>
        <span>Celebtir</span>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end" }}>
          <span style={{ color: ink.secondary, fontWeight: 700 }}>{BRAND_HEADLINE}</span>
          <span style={{ fontSize: 20, marginTop: 6 }}>{BRAND_SUPPORTING_LINE}</span>
        </div>
      </div>
      <div style={{ display: "flex", fontSize: 76, fontWeight: 700, marginTop: 28 }}>{page.name}</div>
      {page.stage && page.factors ? <div style={{ display: "flex", flexDirection: "column" }}>
      <div style={{ display: "flex", alignItems: "center", marginTop: 12, fontSize: 36, color: ink.secondary }}>
        <div style={{ display: "flex", width: 26, height: 26, borderRadius: 4, background: STAGE_COLORS[page.stage], marginRight: 14 }} />
        <span>{STAGE_LABELS[page.stage]}</span>
      </div>
      <div style={{ display: "flex", flexWrap: "wrap", marginTop: 40 }}>
        {FACTORS.map((f) => {
          const r = page.factors![f].rating;
          return (
            <div
              key={f}
              data-factor={f}
              style={{
                display: "flex",
                alignItems: "center",
                width: "50%",
                padding: "12px 0",
                fontSize: 30,
              }}
            >
              <Mark rating={r} />
              <span style={{ color: ink.secondary, marginRight: 12 }}>{FACTOR_LABELS[f]}</span>
              <span style={{ fontWeight: 700 }}>{ratingStyle[r].label}</span>
            </div>
          );
        })}
      </div>
      </div> : (
        // F5: no Stage or ratings for an unassessed protocol.
        <div style={{ display: "flex", flexDirection: "column", marginTop: 24, fontSize: 32, color: ink.secondary }}>
          <span>Unassessed: contract facts listed, not rated</span>
          <span style={{ marginTop: 12, fontSize: 26, color: ink.muted }}>{page.unassessedReason ?? ""}</span>
        </div>
      )}
      <div style={{ display: "flex", marginTop: "auto", color: ink.muted, fontSize: 24 }}>
        Every rating links to the on-chain account it was read from. Not investment advice.
      </div>
    </div>
  );
}
