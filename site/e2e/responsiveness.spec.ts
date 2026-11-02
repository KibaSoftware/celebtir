import { test, expect } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const phase = process.env.CELEBTIR_LAYOUT_PHASE || "after";
const output = path.resolve(__dirname, "../../output/playwright/responsiveness", phase);
const sizes = [[3840,2160], [3440,1440], [2560,1440], [1920,1080], [1440,900], [1280,800], [768,1024], [390,844]];
const routes = ["/", "/risk/", "/value/", "/activity/", "/tokens/", "/compare/?p=kamino-lend,jito", "/protocols/kamino-lend/", "/methodology/"];

for (const [width, height] of sizes) {
  test(`responsive shell ${width}x${height}`, async ({ page }, info) => {
    test.skip(info.project.name !== "desktop", "Explicit CSS viewport matrix.");
    test.setTimeout(120000);
    mkdirSync(output, {recursive:true});
    await page.setViewportSize({width,height});
    const metrics = [];
    for (const route of routes) {
      const errors: string[] = [];
      page.on("pageerror", error => errors.push(error.message));
      await page.goto(route);
      await page.evaluate(() => document.fonts.ready);
      await page.waitForLoadState("networkidle");
      const measured = await page.evaluate(() => {
        const rect = (element: Element) => {
          const box = element.getBoundingClientRect();
          const css = getComputedStyle(element);
          return {left:box.left, right:box.right, width:box.width, contentLeft:box.left+parseFloat(css.paddingLeft), contentRight:box.right-parseFloat(css.paddingRight)};
        };
        return {viewport:innerWidth, height:innerHeight, dpr:devicePixelRatio, documentWidth:document.documentElement.scrollWidth,
          main:rect(document.querySelector("main")!), bar:rect(document.querySelector("header[data-research-shell] > div")!),
          footer:rect(document.querySelector("footer")!)};
      });
      metrics.push({route,...measured});
      const name = route.split("?")[0].replaceAll("/", "_");
      await page.screenshot({path:path.join(output, `${width}-${name}.png`), animations:"disabled"});
      expect(errors).toEqual([]);
      if (phase !== "before") {
        expect(measured.viewport).toBe(width);
        expect(measured.documentWidth, route).toBeLessThanOrEqual(width+1);
        const rail = width >= 1024 ? 208 : 0;
        expect(Math.abs((measured.main.left-rail)-(width-measured.main.right)), `center ${route}`).toBeLessThanOrEqual(2);
        expect(Math.abs(measured.main.contentLeft-measured.bar.contentLeft), `header ${route}`).toBeLessThanOrEqual(2);
        expect(Math.abs(measured.main.contentLeft-measured.footer.contentLeft), `footer ${route}`).toBeLessThanOrEqual(2);
        if (width >= 2560) expect(measured.main.width).toBeGreaterThan(2200);
        const nav = page.getByRole("navigation", {name:"Main"});
        if (width >= 1024) {
          let previousBottom = 0;
          for (const link of await nav.getByRole("link").all()) {
            const box = await link.boundingBox();
            expect(box).not.toBeNull();
            expect(box!.x).toBeGreaterThanOrEqual(0);
            expect(box!.x+box!.width).toBeLessThanOrEqual(209);
            expect(box!.y).toBeGreaterThanOrEqual(previousBottom);
            previousBottom = box!.y + box!.height;
          }
        }
      }
    }
    writeFileSync(path.join(output,`${width}-metrics.json`), JSON.stringify(metrics,null,2));
  });
}

test("responsive reflow at 200 and 400 percent equivalent CSS widths", async ({page}, info) => {
  test.skip(info.project.name !== "desktop" || phase === "before", "After-change reflow coverage.");
  for (const width of [720,360]) {
    await page.setViewportSize({width,height:450});
    for (const route of ["/", "/risk/", "/tokens/", "/compare/?p=kamino-lend,jito"]) {
      await page.goto(route);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth+1)).toBe(true);
      await page.screenshot({path:path.join(output, `reflow-${width}-${route.split("?")[0].replaceAll("/", "_")}.png`)});
      await page.getByRole("button", {name:"Menu",exact:true}).click();
      await expect(page.getByRole("navigation",{name:"Main"})).toBeVisible();
      await expect(page.getByRole("navigation",{name:"Main"}).getByRole("link",{name:"Risk Analysis",exact:true})).toBeVisible();
      await page.keyboard.press("Escape");
      await expect(page.getByRole("button",{name:"Menu",exact:true})).toBeFocused();
    }
  }
});

test("4K display scaling retains the effective CSS layout", async ({browser}, info) => {
  test.skip(info.project.name !== "desktop" || phase === "before", "Explicit scaling model.");
  const measurements = [];
  for (const [width,height,dpr] of [[2560,1440,1.5],[1920,1080,2]]) {
    const context = await browser.newContext({viewport:{width,height},deviceScaleFactor:dpr});
    const page = await context.newPage();
    try {
      await page.goto(`${info.project.use.baseURL}/risk/`);
      const dimensions = await page.evaluate(() => ({cssWidth:innerWidth,dpr:devicePixelRatio,documentWidth:document.documentElement.scrollWidth}));
      expect(dimensions.cssWidth).toBe(width);
      expect(dimensions.dpr).toBe(dpr);
      expect(dimensions.documentWidth).toBeLessThanOrEqual(width+1);
      measurements.push(dimensions);
      await page.screenshot({path:path.join(output, `scaled-${dpr}-risk.png`),scale:"css"});
    } finally { await context.close(); }
  }
  writeFileSync(path.join(output,"scaling-metrics.json"),JSON.stringify(measurements,null,2));
});
