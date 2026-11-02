import { expect, test } from "@playwright/test";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { IS_DEMO } from "./snapshot";

/**
 * The demo run's finish time. The freshness bar warns once data is over 36 h
 * old in the browser's clock (UI-2), which shifts the page; pixel checks pin
 * the clock an hour after the run so they don't depend on today's date.
 */
const DEMO_FINISHED_AT: string = JSON.parse(
  readFileSync(path.join(__dirname, "../../fixtures/site-data/latest/run.json"), "utf8"),
).finished_at;

const STATES = ["unknown", "unrated", "not_applicable", "unavailable"] as const;

// QA-4 / UI-7: the four data states keep one visual everywhere.
test.describe("qa_4 visual regression of the four data states", () => {
  test("qa_4 each state keeps its label and look (checked on every platform)", async ({ page }) => {
    await page.goto("/methodology/#missing-data");
    for (const s of STATES) {
      const el = page.locator(`[data-testid="data-states"] [data-state="${s}"]`);
      await expect(el).toBeVisible();
      const look = await el.evaluate((e) => {
        const c = getComputedStyle(e);
        return { color: c.color, borderStyle: c.borderTopStyle, borderColor: c.borderTopColor, borderWidth: c.borderTopWidth, display: c.display };
      });
      // ink-muted text, a 1 px dashed neutral border: tokens.ts (Highwatch, ADR 023).
      expect(look, s).toEqual({
        color: "rgb(152, 166, 186)",
        borderStyle: "dashed",
        borderColor: "rgb(152, 166, 186)",
        borderWidth: "1px",
        display: "inline-block",
      });
    }
    await expect(page.locator('[data-testid="data-states"] [data-state]')).toHaveText(["Unknown", "Unrated", "Not applicable", "Data unavailable"]);
  });

  test("qa_4 the four states match their screenshot", async ({ page }, info) => {
    // Pixel baselines are per OS (fonts render differently). A platform
    // without one is seeded with `pnpm e2e:update` on that platform.
    const baseline = path.join(__dirname, "__screenshots__", process.platform, "data-states.spec.ts", `data-states-${info.project.name}.png`);
    const updating = ["all", "changed"].includes(info.config.updateSnapshots);
    test.skip(!existsSync(baseline) && !updating, `no ${process.platform} baseline yet: run pnpm e2e:update there`);
    // Baselines show the demo set at a pinned time; a live build differs by design.
    test.skip(!IS_DEMO, "pixel baselines are taken on the demo dataset (CELEBTIR_DATA=fixtures/site-data)");
    await page.clock.setFixedTime(Date.parse(DEMO_FINISHED_AT) + 3_600_000);
    await page.goto("/methodology/#missing-data");
    await expect(page.locator('[data-testid="data-states"]')).toHaveScreenshot(`data-states-${info.project.name}.png`);
  });
});
