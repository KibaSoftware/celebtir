import { test, expect } from "@playwright/test";
import { readdirSync } from "node:fs";
import path from "node:path";

const out = path.resolve(__dirname, "../out");
const routes = readdirSync(out, { recursive: true, withFileTypes: true })
  .filter(e => e.isFile() && e.name === "index.html")
  .map(e => "/" + path.relative(out, e.parentPath).replaceAll("\\", "/") + "/")
  .map(r => r === "//" ? "/" : r)
  .filter(r => !r.includes("_not-found"));

for (const chain of ["all", "solana", "hyperliquid"] as const) {
  test(`hui_1 ui_9 qa_all_exported_routes under ${chain}`, async ({ page, baseURL }) => {
    test.setTimeout(240_000);
    const errors: string[] = [];
    page.on("pageerror", e => errors.push(e.message));
    const origin = new URL(baseURL!).origin;
    page.on("console", message => { if (message.type() === "error") errors.push(message.text()); });
    page.on("response", r => { if (r.status() >= 400 && new URL(r.url()).origin === origin) errors.push(`${r.status()} ${r.url()}`); });
    page.on("requestfailed", request => {
      const failure = request.failure()?.errorText;
      if (new URL(request.url()).origin === origin && failure !== "net::ERR_ABORTED") errors.push(`${failure}: ${request.url()}`);
    });
    for (const route of routes) {
      const response = await page.goto(route + (chain === "all" ? "" : `?chain=${chain}`));
      expect(response?.status(), route).toBe(200);
      await expect(page.locator("main")).toBeVisible();
      const picker = page.getByRole("group", { name: "Chain" });
      await expect(picker).toBeVisible();
      if (!await picker.getAttribute("data-fixed")) {
        await expect(picker.getByRole("button", { name: { all: "All chains", solana: "Solana", hyperliquid: "Hyperliquid" }[chain], exact: true })).toHaveAttribute("aria-pressed", "true");
      }
    }
    expect(errors).toEqual([]);
  });
}

test("ui_9 qa_scoped_link_navigation_has_no_missing_route_payloads", async ({ page }) => {
  const errors: string[] = [];
  page.on("response", r => { if (r.status() >= 400) errors.push(`${r.status()} ${r.url()}`); });
  await page.goto("/value/?chain=solana");
  await page.waitForLoadState("networkidle");
  await page.locator("main a[href^='/protocols/']").first().click();
  await expect(page).toHaveURL(/protocols\/.*chain=solana/);
  await expect(page.locator("#risk-analysis")).toBeVisible();
  expect(errors).toEqual([]);
});
