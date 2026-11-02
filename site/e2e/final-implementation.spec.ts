import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { mkdirSync } from "node:fs";
import path from "node:path";
import { facts } from "./snapshot";

const output = process.env.CELEBTIR_REVIEW_SCREENSHOTS
  ? path.resolve(process.env.CELEBTIR_REVIEW_SCREENSHOTS)
  : path.resolve(__dirname, "../../output/playwright/final-implementation");
const routes = ["/", "/risk/", "/value/", "/activity/", "/tokens/", "/compare/?p=kamino-lend,jito", "/hyperevm/", "/hip3/", "/changelog/", "/methodology/", "/protocols/kamino-lend/", "/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/"];
const sizes = [[1440,900],[1280,800],[1024,768],[768,1024],[390,844],[360,780]];

for (const [width,height] of sizes) test(`final responsive gallery ${width}x${height}`, async ({page}, info) => {
  test.skip(info.project.name !== "desktop", "The gallery explicitly sets all six viewports.");
  test.setTimeout(180_000);
  mkdirSync(output,{recursive:true});
  await page.setViewportSize({width,height});
  for (const route of routes) {
    await page.goto(route); await page.waitForLoadState("networkidle");
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth+1), route).toBe(true);
    await page.screenshot({path:path.join(output,`${width}-${route.split("?")[0].replaceAll("/","_") || "summary"}.png`),fullPage:true});
  }
});

test("final risk inspector keyboard, focus return and accessibility", async ({page}, info) => {
  await page.goto("/risk/");
  const trigger=page.getByRole("button",{name:/Kamino Lend, Upgrade delay:.*Open the evidence/});
  await trigger.focus(); await page.keyboard.press("Enter");
  const dialog=page.getByRole("dialog"); await expect(dialog).toBeVisible();
  await expect(dialog.getByText("Observed configuration",{exact:true})).toBeVisible();
  expect(page.frames()).toHaveLength(1);
  const audit=await new AxeBuilder({page}).setLegacyMode(true).withTags(["wcag2a","wcag2aa","wcag21aa","wcag22aa"]).analyze();
  expect(audit.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>n.target)}))).toEqual([]);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth+1)).toBe(true);
  await page.screenshot({path:path.join(output,`${info.project.name}-risk-inspector.png`)});
  await page.keyboard.press("Escape"); await expect(dialog).toHaveCount(0); await expect(trigger).toBeFocused();
});

test("final token grouping and factor deep link survives reload", async ({page}, info) => {
  await page.goto("/tokens/");
  await page.getByLabel("Group tokens").selectOption("flat");
  await page.getByRole("button",{name:/All tokens.*tokens/}).click();
  await expect(page.locator("tr[data-mint]:visible")).toHaveCount(0);
  await expect(page).toHaveURL(/tgroups=/);
  await page.reload(); await expect(page.locator("tr[data-mint]:visible")).toHaveCount(0);
  await page.getByRole("button",{name:/All tokens.*tokens/}).click();
  await page.getByRole("button",{name:/USDC, Supply control:.*open evidence/}).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page).toHaveURL(/tdetail=/);
  await page.reload(); await expect(page.getByRole("dialog")).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth+1)).toBe(true);
  await page.screenshot({path:path.join(output,`${info.project.name}-token-inspector.png`)});
  await page.keyboard.press("Escape"); await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("final typed economic-table sorting persists in URL", async ({page}) => {
  for(const route of ["/value/","/activity/"]) {
    await page.goto(route);
    const table=page.getByRole("region",{name:route === "/value/" ? "Protocol value" : "Protocol activity",exact:true});
    await table.getByRole("button",{name:/^Protocol/}).click();
    await expect(page).toHaveURL(/sort=column/);
    const before=await table.locator("tbody tr").evaluateAll(rows=>rows.map(r=>r.getAttribute("data-sort-id")));
    await page.reload();
    await expect.poll(()=>table.locator("tbody tr").evaluateAll(rows=>rows.map(r=>r.getAttribute("data-sort-id")))).toEqual(before);
  }
});

test("final provenance, assets, native coverage and command search", async ({page}) => {
  await page.goto("/");
  await expect(page.getByRole("heading",{name:"Who controls DeFi?"})).toHaveCount(0);
  await expect(page.getByRole("heading",{name:"Recent observations"})).toBeVisible();
  for(const asset of ["/art/observatory.webp","/art/archive.webp","/brand/celebtir-insignia-64.png", ...["risk","value","activity","tokens","compare","hip3","changes","methodology","hyperevm"].map(name=>`/art/${name}.webp`)]) expect((await page.request.get(asset)).ok()).toBe(true);
  await page.keyboard.press("Control+k"); await expect(page.getByRole("combobox")).toBeFocused();
  const count = Object.values(facts.protocols).filter(p => p.chain === "hyper_evm").length;
  await page.goto("/hyperevm/"); await expect(page.getByRole("heading",{name:count ? `${count} protocols with recorded contract facts` : "Coverage required"})).toBeVisible();
});

test("final 200 percent equivalent reflow for key routes", async ({page}, info) => {
  test.skip(info.project.name !== "desktop", "Explicit zoom and viewport below.");
  for (const route of ["/", "/risk/", "/tokens/", "/compare/?p=kamino-lend,jito"]) {
    // 1440x900 at 200% browser zoom has a 720x450 CSS layout viewport.
    await page.setViewportSize({width:720,height:450});
    await page.goto(route); await page.waitForLoadState("networkidle");
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth+1), route).toBe(true);
    await page.screenshot({path:path.join(output,`zoom200-${route.split("?")[0].replaceAll("/","_")}.png`),fullPage:true});
  }
});
