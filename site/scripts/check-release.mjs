// Release configuration gate (H1): validate the configured repository URL and
// rendered corrections link. Public availability is a separate owner check.
// Usage: node site/scripts/check-release.mjs [out-dir]
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const REPO = /^https:\/\/github\.com\/[A-Za-z0-9-]+\/[A-Za-z0-9._-]+$/;

/** Problems with the release configuration and the built About page; empty when it checks out. */
export function checkRelease(env = process.env, out = path.join(process.cwd(), "out")) {
  const problems = [];
  const repo = env.CELEBTIR_REPO_URL ?? "";
  if (!repo) problems.push("CELEBTIR_REPO_URL is not set: the About page has no corrections channel (H1)");
  else if (!REPO.test(repo)) problems.push(`CELEBTIR_REPO_URL must be https://github.com/<owner>/<repo>, got ${repo}`);
  const about = path.join(out, "about", "index.html");
  if (!existsSync(about)) {
    problems.push(`${about} is missing: build the site first`);
  } else {
    const html = readFileSync(about, "utf8");
    if (!html.includes('id="corrections"')) problems.push("/about/ has no #corrections anchor");
    if (repo && !html.includes(`href="${repo}/issues/new"`)) problems.push(`/about/ does not link ${repo}/issues/new: rebuild with CELEBTIR_REPO_URL set`);
  }
  return problems;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const problems = checkRelease(process.env, process.argv[2] ? path.resolve(process.argv[2]) : undefined);
  for (const p of problems) console.error(`check-release: ${p}`);
  if (problems.length) process.exit(1);
  console.log("check-release: ok");
}
