import { expect, test } from "@playwright/test";
import { PEER, PEER_NAME } from "./snapshot";

// QA-4 / UI-4 / UI-16: ticking rows on two pages lands both in one comparison.
test("qa_4 compare: rows ticked on Summary and Risk open one comparison", async ({ page }) => {
  await page.goto("/");
  // The static fallback is inert until hydration.
  await page.waitForLoadState("networkidle");
  await page.getByRole("checkbox", { name: "Compare Kamino Lend" }).check();
  await page.goto("/risk/");
  await page.waitForLoadState("networkidle");
  await page.getByRole("checkbox", { name: `Compare ${PEER_NAME}`, exact: true }).check();
  const tray = page.getByRole("complementary", { name: "Compare tray" });
  await expect(tray).toContainText("Kamino Lend");
  await expect(tray).toContainText(PEER_NAME);
  await tray.getByRole("link", { name: "Compare 2" }).click();
  await expect(page).toHaveURL(new RegExp(`/compare/\\?p=kamino-lend,${PEER}`));
  const table = page.getByRole("table", { name: /^Comparison of/ });
  await expect(table.getByRole("columnheader")).toHaveText([/Kamino Lend/, new RegExp(PEER_NAME)]);
  await expect(table.getByRole("rowheader", { name: /Upgrade controller/ })).toBeVisible();
});
