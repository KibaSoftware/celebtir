import { expect, test, type Page } from "@playwright/test";
import { namesOn } from "./snapshot";

// HUI-1 / UI-9: every route under each chain choice shows only that chain,
// the shared empty state when the chain has nothing, and keeps the chain in
// the nav. Expectations come from the data the site was built with.

const ROUTES = ["/", "/risk/", "/value/", "/activity/", "/tokens/", "/compare/", "/changelog/", "/controllers/", "/methodology/"];
const SCOPED_LISTS = ["/", "/risk/", "/value/", "/activity/", "/tokens/"];
const CHAINS = ["all", "solana", "hyperliquid"] as const;
const other = { solana: "hyperliquid", hyperliquid: "solana" } as const;

async function open(page: Page, route: string, chain: (typeof CHAINS)[number]) {
  await page.goto(chain === "all" ? route : `${route}?chain=${chain}`);
  await page.waitForLoadState("networkidle");
}

for (const chain of CHAINS) {
  for (const route of ROUTES) {
    test(`hui_1 ${route} under ${chain} shows only that chain`, async ({ page }) => {
      const errors: string[] = [];
      page.on("pageerror", (e) => errors.push(e.message));
      await open(page, route, chain);
      const picker = page.getByRole("group", { name: "Chain" });
      const label = { all: "All chains", solana: "Solana", hyperliquid: "Hyperliquid" }[chain];
      await expect(picker.getByRole("button", { name: label })).toHaveAttribute("aria-pressed", "true");

      if (chain !== "all") {
        const main = page.locator("main");
        // Route pages never name a protocol from the other chain (search
        // results and the compare tray live outside <main>).
        if (route !== "/methodology/") {
          for (const name of namesOn(other[chain])) await expect(main).not.toContainText(name);
        }
        if (SCOPED_LISTS.includes(route) && namesOn(chain).length === 0) {
          await expect(main.locator(`[data-scope-empty="${chain}"]`).first()).toBeVisible();
        }
        // The nav carries the chain to the next page.
        // (On a phone the links sit in the folded menu, so match them hidden too.)
        const risk = page.locator("#main-nav-list a").filter({ hasText: "Risk Analysis" });
        await expect(risk).toHaveAttribute("href", new RegExp(`chain=${chain}`));
      }
      expect(errors).toEqual([]);
    });
  }
}

test("hui_1 the chain choice survives a nav click", async ({ page }, info) => {
  test.skip(info.project.name === "phone", "the folded menu is covered on desktop");
  await open(page, "/", "hyperliquid");
  await page.getByRole("navigation", { name: "Main" }).getByRole("link", { name: "Risk Analysis" }).click();
  await expect(page).toHaveURL(/\/risk\/\?chain=hyperliquid/);
  await expect(page.getByRole("group", { name: "Chain" }).getByRole("button", { name: "Hyperliquid" })).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("group", { name: "Chain" }).getByRole("button", { name: "All chains" }).click();
  await expect(page).not.toHaveURL(/chain=/);
});

test("hui_1 a protocol page shows its chain as fixed", async ({ page }) => {
  await open(page, "/protocols/kamino-lend/", "hyperliquid");
  const picker = page.getByRole("group", { name: "Chain" });
  await expect(picker).toHaveAttribute("data-fixed", "solana");
  await expect(picker.getByRole("button", { name: "Solana" })).toHaveAttribute("aria-pressed", "true");
});

// B1: the header never overlaps itself on a desktop width.
for (const width of [1024, 1280, 1440]) {
  test(`hui_1 header at ${width} px: nav, chain picker and search do not overlap`, async ({ page }, info) => {
    test.skip(info.project.name === "phone", "desktop widths");
    await page.setViewportSize({ width, height: 900 });
    await open(page, "/", "all");
    const boxes = [];
    for (const l of [page.locator("#main-nav-list"), page.getByRole("group", { name: "Chain" }), page.getByRole("search").first()]) {
      if (await l.isVisible()) boxes.push((await l.boundingBox())!);
    }
    expect(boxes.length).toBeGreaterThanOrEqual(2);
    for (let i = 0; i < boxes.length; i++) {
      for (let j = i + 1; j < boxes.length; j++) {
        const a = boxes[i]!;
        const b = boxes[j]!;
        const overlap = a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
        expect(overlap, `boxes ${i} and ${j} overlap at ${width} px`).toBe(false);
      }
    }
    // The search placeholder is not cut off.
    const search = page.getByRole("searchbox").first();
    const fits = await search.evaluate((e: HTMLInputElement) => {
      const c = document.createElement("canvas").getContext("2d")!;
      c.font = getComputedStyle(e).font;
      return c.measureText(e.placeholder).width <= e.clientWidth;
    });
    expect(fits).toBe(true);
  });
}
