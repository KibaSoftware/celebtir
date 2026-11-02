import { expect, test } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { buildProtocolPage } from "../src/lib/protocol";
import { loadHistory, loadSnapshot } from "../src/lib/data";
import type { ChangeEvent, ProtocolHistoryFile } from "../src/lib/types";
import { eventAnchor } from "../src/lib/changes";
import { PEER } from "./snapshot";

const snapshot = loadSnapshot();
const events = loadHistory<{events: ChangeEvent[]}>("events")?.events ?? [];
const kamino = buildProtocolPage(snapshot, "kamino-lend", loadHistory<ProtocolHistoryFile>("kamino-lend"), events);
const output = path.resolve(__dirname, "../../output/playwright/clarity-polish");

test("clarity preserves every permission, factor and graph record behind accessible disclosure", async ({page}) => {
  await page.goto("/protocols/kamino-lend/");
  await page.waitForLoadState("networkidle");
  const holders = page.getByRole("list", {name:"permission holders",exact:true});
  await expect(holders.locator(":scope > li:not([hidden]) article")).toHaveCount(Math.min(10, kamino.actors.length));
  if (kamino.actors.length > 10) await holders.getByRole("button", {name:`Show all ${kamino.actors.length} permission holders`,exact:true}).click();
  await expect(holders.locator("article")).toHaveCount(kamino.actors.length);
  for (const actor of kamino.actors) {
    const record = page.locator(`#actor-${actor.id}`).locator("..");
    await record.locator("details > summary").first().click();
    const reveal = record.getByRole("button", {name:/^Show all \d+ permissions$/});
    if (await reveal.count()) await reveal.click();
    await expect(record.locator("tbody > tr:not(:has(button[aria-expanded]))")).toHaveCount(actor.grants.length);
  }
  const graphList = page.getByRole("list", {name:"Control graph as a list",exact:true});
  if (kamino.graph.nodes.length > 10) await graphList.getByRole("button", {name:`Show all ${kamino.graph.nodes.length} Control graph as a list`,exact:true}).click();
  await expect(graphList.locator(":scope > li[id]")).toHaveCount(kamino.graph.nodes.length);
  // Every edge occurs once at each endpoint, including records still collapsed.
  const edges = await graphList.locator("details a[href^='#node-']").count();
  expect(edges).toBe(kamino.graph.edges.length * 2);
  for (const [factor, score] of Object.entries(kamino.factors!)) {
    const card = page.locator(`#factor-${factor}`).locator("xpath=ancestor::article");
    await expect(card).toHaveAttribute("data-rating", score.rating);
    for (const evidence of score.evidence) expect(await card.locator(`a[href="${evidence.explorer_url}"]`).count()).toBeGreaterThan(0);
  }
});

test("clarity deep links reveal hidden permission holders and history records", async ({page}) => {
  const actor = kamino.actors.at(-1)!;
  await page.goto(`/protocols/kamino-lend/#actor-${actor.id}`);
  await expect(page.locator(`#actor-${actor.id}`)).toBeVisible();
  await expect(page.locator(`#actor-${actor.id}`)).toBeFocused();
  await page.locator('nav[aria-label="Sections"] a[href="#summary"]').click();
  await page.goBack();
  await expect(page.locator(`#actor-${actor.id}`)).toBeFocused();
  const event = events.at(-1)!;
  await page.goto(`/changelog/#${eventAnchor(event.id)}`);
  const target = page.locator(`[id="${eventAnchor(event.id)}"]`);
  await expect(target).toBeVisible();
  await expect(target).toBeFocused();
  await target.locator("details > summary").click();
  await expect(target.locator("details")).toHaveAttribute("open", "");
});

test("clarity history preview and keyboard disclosure retain all filtered events", async ({page, isMobile}) => {
  await page.goto("/changelog/");
  const list = page.getByRole("list", {name:"Events, newest first",exact:true});
  await expect(list.locator(":scope > li[id]:visible")).toHaveCount(Math.min(events.length,10));
  const button = list.getByRole("button", {name:/^Show all/});
  if (events.length <= 10) {
    await expect(button).toHaveCount(0);
    return;
  }
  await button.focus(); await page.keyboard.press("Enter");
  await expect(list.locator(":scope > li[id]:visible")).toHaveCount(events.length);
  await expect(list.getByRole("button", {name:/^Show fewer/})).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(list.locator(":scope > li[id]:visible")).toHaveCount(10);
  await expect(button).toBeFocused();
  if (isMobile) {
    await button.tap();
    await expect(list.locator(":scope > li[id]:visible")).toHaveCount(events.length);
    await list.getByRole("button", {name:/^Show fewer/}).tap();
    await expect(list.locator(":scope > li[id]:visible")).toHaveCount(10);
  }
});

test("clarity token copy states the signer boundary without changing ratings", async ({page}) => {
  const mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
  await page.goto(`/tokens/${mint}/`);
  await expect(page.getByText(/An authority address does not establish one human signer/)).toBeVisible();
  await expect(page.getByText(/Issuer-controlled mint and freeze powers can be intentional/)).toBeVisible();
  for (const [factor, score] of Object.entries(snapshot.scores.tokens[mint].factors)) {
    const card = page.locator(`#tf-${factor}`).locator("..");
    await expect(card.locator("[data-rating]").first()).toHaveAttribute("data-rating", score.rating);
  }
  expect(await page.locator("main").innerText()).not.toMatch(/single key/i);
});

test("clarity review gallery and reflow", async ({page}, info) => {
  test.skip(info.project.name !== "desktop", "This gallery sets each review viewport explicitly.");
  test.setTimeout(240_000);
  mkdirSync(output,{recursive:true});
  const programToken = Object.entries(snapshot.facts.tokens).find(([,t]) => t.mint_authority.value.kind === "program_controlled")?.[0];
  const peer = snapshot.facts.protocols["jupiter-lend"] ? "jupiter-lend" : PEER;
  const routes = ["/protocols/kamino-lend/", `/protocols/${peer}/`, "/tokens/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/", "/tokens/", `/compare/?p=kamino-lend,${peer}`, "/changelog/", "/controllers/", "/", "/hip3/", "/methodology/", "/risk/", "/value/", "/activity/", "/hyperevm/", "/api/", "/about/", "/glossary/", "/archived/", ...(programToken ? [`/tokens/${programToken}/`] : [])];
  const metrics = [];
  for (const [width,height] of [[1440,900],[768,1024],[375,812],[720,450]]) {
    await page.setViewportSize({width,height});
    for (const [i,route] of routes.entries()) {
      await page.goto(route); await page.waitForLoadState("networkidle"); await page.evaluate(() => document.fonts.ready); await page.evaluate(() => window.scrollTo(0,0));
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1),route).toBe(true);
      const clipped = await page.locator("main header dl dd").evaluateAll(elements => elements.filter(e => e.scrollWidth > e.clientWidth+1).map(e=>e.textContent));
      expect(clipped,route).toEqual([]);
      await page.screenshot({path:path.join(output,`after-${i}-${width}.png`)});
      metrics.push(await page.evaluate(() => ({route:location.pathname,viewport:innerWidth,height:document.documentElement.scrollHeight,width:document.documentElement.scrollWidth})));
    }
  }
  writeFileSync(path.join(output,"metrics.json"),JSON.stringify(metrics,null,2));
});
