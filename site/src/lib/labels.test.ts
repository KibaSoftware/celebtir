import { describe, expect, it } from "vitest";
import { formatChange, formatChangeBp, formatUsd } from "./format";
import { OVERLAY } from "./compare";
import { CONTROL_COLORS, SERIES_COLOR, SERIES_COLORS, STAGE_COLORS } from "./labels";
import { contrast, status, surfaces } from "./tokens";

describe("ui_8 chart colors", () => {
  const chartColors = [...Object.values(CONTROL_COLORS), ...Object.values(STAGE_COLORS), SERIES_COLOR, ...OVERLAY.map((o) => o.color), ...SERIES_COLORS];

  it("ui_8 chart colors never reuse a status color", () => {
    const reserved = [status.good, status.warning, status.serious, status.critical];
    for (const c of chartColors) expect(reserved).not.toContain(c);
  });

  it("ui_24 Stage colors are never status colors, so Stage 0 cannot read as High", () => {
    for (const c of Object.values(STAGE_COLORS)) expect([status.good, status.warning, status.serious, status.critical]).not.toContain(c);
  });

  it("ui_8 every chart color clears 2:1 against the surface", () => {
    for (const c of chartColors) expect(contrast(c, surfaces.surface)).toBeGreaterThanOrEqual(2);
  });
});

describe("ui_10 number formats", () => {
  it("ui_10 dollars are compact, changes are signed", () => {
    expect(formatUsd(1_310_434_543)).toBe("$1.31B");
    expect(formatUsd(149_746)).toBe("$149.75K");
    expect(formatChange(0.052)).toBe("+5.2%");
    expect(formatChange(-0.1)).toBe("−10.0%");
    expect(formatChange(0)).toBe("0.0%");
    expect(formatChangeBp(-250)).toBe("−2.5%");
  });
});
