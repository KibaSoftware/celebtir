import { defineConfig, devices } from "@playwright/test";

// QA-4: end-to-end, visual and accessibility tests against the static export
// (run `pnpm build` first). They drive the installed Chrome (`channel:
// "chrome"`), so no browser download is needed here or on CI runners.
const PORT = Number(process.env.CELEBTIR_TEST_PORT || 3210);
if (!Number.isInteger(PORT) || PORT < 1024 || PORT > 65535) throw new Error('Invalid CELEBTIR_TEST_PORT');

export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  // Bound Chrome/axe memory pressure consistently on developer machines and CI.
  workers: 4,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  // Pixel baselines differ per OS; each platform keeps its own.
  snapshotPathTemplate: "{testDir}/__screenshots__/{platform}/{testFilePath}/{arg}{ext}",
  expect: { toHaveScreenshot: { maxDiffPixelRatio: 0.01 } },
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    channel: "chrome",
    colorScheme: "dark",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"], channel: "chrome", viewport: { width: 1440, height: 900 } } },
    { name: "phone", use: { ...devices["Pixel 7"], channel: "chrome", viewport: { width: 375, height: 812 } } },
  ],
  webServer: {
    command: `node scripts/serve-out.mjs ${PORT}`,
    url: `http://127.0.0.1:${PORT}/`,
    reuseExistingServer: !process.env.CI,
  },
});
