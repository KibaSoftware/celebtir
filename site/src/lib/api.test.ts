// @vitest-environment node
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import http from "node:http";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { checkApi } from "../../scripts/check-api.mjs";

const dir = mkdtempSync(path.join(tmpdir(), "celebtir-api-"));
const servers: http.Server[] = [];

/** A static server over `dir`, sending the CORS header like GitHub Pages, or not. */
function serve(cors: boolean): Promise<string> {
  const server = http.createServer((req, res) => {
    try {
      const body = readFileSync(path.join(dir, decodeURIComponent(new URL(req.url!, "http://x").pathname)));
      res.writeHead(200, { "content-type": "application/json", ...(cors ? { "access-control-allow-origin": "*" } : {}) });
      res.end(body);
    } catch {
      res.writeHead(404).end();
    }
  });
  servers.push(server);
  return new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve(`http://127.0.0.1:${(server.address() as { port: number }).port}`)));
}

beforeAll(() => {
  mkdirSync(path.join(dir, "data/v2/schemas"), { recursive: true });
  const licence = { name: "CC BY 4.0", url: "https://creativecommons.org/licenses/by/4.0/", attribution: "Data: Celebtir (CC BY 4.0)" };
  writeFileSync(path.join(dir, "data/v2/summary.json"), JSON.stringify({ licence, protocols: [] }));
  writeFileSync(path.join(dir, "data/v2/events.json"), JSON.stringify({ licence, events: [] }));
  writeFileSync(path.join(dir, "data/v2/schemas/summary.schema.json"), JSON.stringify({ type: "object" }));
});

afterAll(() => servers.forEach((s) => s.close()));

describe("api_3 readable from any website", () => {
  it("api_3 passes when the files are served with Access-Control-Allow-Origin: *", async () => {
    expect(await checkApi(await serve(true))).toEqual([]);
  });

  it("api_3 fails when a file is not readable cross-origin", async () => {
    const problems: string[] = await checkApi(await serve(false));
    expect(problems.some((p) => p.includes("not readable cross-origin"))).toBe(true);
  });

  it("api_5 fails when the summary has no licence field", async () => {
    writeFileSync(path.join(dir, "data/v2/summary.json"), JSON.stringify({ protocols: [] }));
    const problems: string[] = await checkApi(await serve(true));
    expect(problems.some((p) => p.includes("licence"))).toBe(true);
  });
});
