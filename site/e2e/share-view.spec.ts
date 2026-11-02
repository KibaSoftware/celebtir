import { expect, test } from "@playwright/test";
import { withControlClass } from "./snapshot";

// QA-4 / UI-1: a filtered, column-trimmed view survives being shared as a link.
test("qa_4 share a filtered view: a copied URL reproduces it in a fresh browser", async ({ page, browser }) => {
  test.skip(test.info().project.name === "phone", "the filter menus are covered on desktop");
  await page.goto("/");
  // The static fallback is inert until the table is bound to the URL.
  await page.waitForLoadState("networkidle");
  await page.locator('[data-view="live"]').waitFor();
  await page.getByText("Filters", { exact: true }).click();
  await page.getByRole("checkbox", { name: "Multisig, delay 24 h+" }).check();
  await page.getByText("Columns", { exact: true }).click();
  await page.getByRole("checkbox", { name: "Category" }).uncheck();
  await expect(page).toHaveURL(/controller=multisig_with_delay/);
  await expect(page).toHaveURL(/hide=category/);
  const url = page.url();

  const fresh = await browser.newContext();
  const other = await fresh.newPage();
  await other.goto(url);
  const table = other.getByRole("table").filter({ has: other.getByRole("columnheader", { name: /Protocol/ }) });
  const expected = withControlClass("multisig_with_delay");
  await expect(table.locator("tbody tr")).toHaveCount(expected.length);
  const shown = await table.locator("tbody tr").evaluateAll((rows) => rows.map((r) => r.getAttribute("data-protocol")));
  expect([...shown].sort()).toEqual(expected);
  await expect(table.getByRole("columnheader", { name: /Category/ })).toHaveCount(0);
  await expect(other.getByText("Filters (1)")).toBeVisible();
  await fresh.close();
});
