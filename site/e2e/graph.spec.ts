import { expect, test } from "@playwright/test";

const SQUADS = "node-6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM";

// QA-4 / UI-20: clicking a node opens its side panel; signers expand on demand.
test("qa_4 graph node click opens the side panel with its facts", async ({ page }) => {
  await page.goto("/protocols/kamino-lend/");
  await page.waitForLoadState("networkidle");
  const node = page.locator(`.react-flow__node[data-id="${SQUADS}"]`);
  const panel = page.getByRole("complementary", { name: /Squads 5 of 10/ });
  // A large graph can leave the node outside the canvas; its hash pans to it
  // (UI-13). Close that panel so the click below is what opens it.
  await page.evaluate((id) => (window.location.hash = id), SQUADS);
  await expect(panel).toBeVisible();
  await panel.getByRole("button", { name: "Close" }).click();
  await expect(panel).toHaveCount(0);
  await node.scrollIntoViewIfNeeded();
  await node.click();
  await expect(panel).toBeVisible();
  await expect(panel.getByRole("link", { name: "View on Solscan" })).toHaveAttribute("href", /solscan\.io\/account\/6hhB/);
  const signers = panel.getByRole("button", { name: "10 signers" });
  await expect(signers).toHaveAttribute("aria-expanded", "false");
  await signers.click();
  await expect(panel.getByRole("list", { name: "Signers" }).getByRole("listitem")).toHaveCount(10);
  await page.keyboard.press("Escape");
  await expect(panel).toHaveCount(0);
});

// QA-4 / UI-13: an actor in Permissions focuses its node in the control graph.
test("qa_4 an actor's link focuses its node in the control graph", async ({ page }) => {
  await page.goto("/protocols/kamino-lend/");
  // The panel opens from client code; the static page is inert until hydrated.
  await page.waitForLoadState("networkidle");
  const reveal = page.locator("#permissions").getByRole("button", {name:/^Show all .* permission holders$/});
  if (await reveal.count()) await reveal.click();
  await page.locator(`#permissions a[href="#${SQUADS}"]`).click();
  await expect(page.locator(`#${SQUADS}`)).toBeFocused();
  await expect(page.getByRole("complementary", { name: /Squads 5 of 10/ })).toBeVisible();
});
