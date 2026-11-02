// A small static server for out/ (the static export), as GitHub Pages serves
// it: directories map to index.html, unknown paths are 404s. Used by the
// end-to-end tests and Lighthouse. Usage: node scripts/serve-out.mjs [port]
import { createReadStream, existsSync, statSync } from "node:fs";
import http from "node:http";
import path from "node:path";

const root = path.resolve(process.cwd(), "out");
const port = Number(process.argv[2] ?? 3210);
const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".txt": "text/plain; charset=utf-8",
  ".md": "text/markdown; charset=utf-8",
  ".atom": "application/atom+xml",
  ".woff2": "font/woff2",
};

http
  .createServer((req, res) => {
    const url = new URL(req.url ?? "/", "http://localhost");
    let file = path.join(root, decodeURIComponent(url.pathname));
    if (!file.startsWith(root)) {
      res.writeHead(403).end();
      return;
    }
    if (existsSync(file) && statSync(file).isDirectory()) file = path.join(file, "index.html");
    if (!existsSync(file)) {
      const notFound = path.join(root, "404.html");
      res.writeHead(404, { "content-type": TYPES[".html"] });
      if (existsSync(notFound)) createReadStream(notFound).pipe(res);
      else res.end("not found");
      return;
    }
    res.writeHead(200, {
      "content-type": TYPES[path.extname(file)] ?? "application/octet-stream",
      "access-control-allow-origin": "*",
    });
    createReadStream(file).pipe(res);
  })
  .listen(port, "127.0.0.1", () => console.log(`serve-out: http://127.0.0.1:${port}`));
