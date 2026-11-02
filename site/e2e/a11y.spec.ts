import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const ROUTES = [
  "/",
  "/risk/",
  "/risk/?factor=admin_powers",
  "/value/",
  "/activity/",
  "/tokens/",
  "/hyperevm/",
  "/hip3/",
  "/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/",
  "/protocols/kamino-lend/",
  "/compare/?p=kamino-lend,spl-stake-pool",
  "/changelog/",
  "/methodology/",
  "/glossary/",
  "/about/",
  "/api/",
  "/controllers/",
  "/archived/",
];

// QA-4 / UI-23 / UI-24: automated accessibility scans, colour contrast included.
for (const route of ROUTES) {
  test(`qa_4 axe finds no violations on ${route}`, async ({ page }) => {
    await page.goto(route);
    await page.waitForLoadState("networkidle");
    // The app has no frames. Run axe in-page to avoid the adapter's large
    // JSON transfer to a blank page (which corrupts results on Windows Chrome).
    // Fail explicitly if frames are introduced, so coverage cannot silently shrink.
    expect(page.frames()).toHaveLength(1);
    const result = await new AxeBuilder({ page }).setLegacyMode(true).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]).analyze();
    const found = result.violations.map((v) => `${v.id} (${v.impact}): ${v.help} → ${v.nodes.slice(0, 3).map((n) => n.target.join(" ")).join(" | ")}`);
    expect(found).toEqual([]);
  });
}
