import { describe, expect, it } from "vitest";
import { contrast, ink, ratingStyle, surfaces } from "./tokens";

// UI-24: WCAG AA contrast throughout.
describe("ui_24 contrast", () => {
  for (const [bg, bgHex] of Object.entries(surfaces)) {
    for (const [name, hex] of Object.entries(ink)) {
      it(`ui_24 ${name} text on ${bg} meets AA (4.5:1)`, () => {
        expect(contrast(hex, bgHex)).toBeGreaterThanOrEqual(4.5);
      });
    }
    for (const [rating, style] of Object.entries(ratingStyle)) {
      it(`ui_24 ${rating} icon on ${bg} meets the 3:1 non-text minimum`, () => {
        expect(contrast(style.color, bgHex)).toBeGreaterThanOrEqual(3);
      });
    }
  }

  it("ui_24 contrast() matches known values", () => {
    expect(contrast("#ffffff", "#000000")).toBeCloseTo(21, 5);
    expect(contrast("#777777", "#ffffff")).toBeCloseTo(4.48, 2);
  });
});
