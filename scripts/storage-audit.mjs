// Read original inputs; validate disposable copies only. No network collection.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { trimData, removeWithin } from './deploy/storage.mjs';

const root = process.cwd();
const binary = path.resolve(process.argv[2] || 'output/predeployment/celebtir.exe');
const output = path.resolve(process.argv[3] || 'output/storage-audit');
fs.mkdirSync(output, { recursive: true });
const bytes = new Map();
let unreadable = 0;
function files(dir, visit) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const file = path.join(dir, entry.name);
    if (entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) files(file, visit);
    else visit(file);
  }
}
files(root, file => {
  try {
    const size = fs.statSync(file).size;
    const parts = path.relative(root, file).split(path.sep);
    for (let depth = 1; depth <= Math.min(3, parts.length - 1); depth++) {
      const name = parts.slice(0, depth).join('/');
      bytes.set(name, (bytes.get(name) || 0) + size);
    }
  } catch { unreadable++; }
});
function manifest(dir) {
  const hashes = {};
  files(dir, file => { hashes[path.relative(dir, file).replaceAll('\\', '/')] = crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'); });
  return hashes;
}
function selectedManifest(dir) {
  const result = {};
  for (const part of ['latest', 'history', 'api', 'feeds']) {
    for (const [name, hash] of Object.entries(manifest(path.join(dir, part)))) result[`${part}/${name}`] = hash;
  }
  return result;
}
const protectedDirs = ['data', 'output/qa-v02-data', 'registry', 'methodology'];
const before = Object.fromEntries(protectedDirs.map(dir => [dir, manifest(path.join(root, dir))]));
const validations = [];
for (const input of ['data', 'output/qa-v02-data']) {
  const work = fs.mkdtempSync(path.join(output, 'validation-'));
  const baseline = path.join(work, 'baseline');
  const reduced = path.join(work, 'reduced');
  fs.cpSync(path.join(root, input), baseline, { recursive: true });
  fs.cpSync(path.join(root, input), reduced, { recursive: true });
  trimData(reduced);
  const date = fs.readdirSync(path.join(baseline, 'snapshots')).filter(n => /^\d{4}-\d{2}-\d{2}$/.test(n)).sort().at(-1);
  for (const dir of [baseline, reduced]) {
    const run = spawnSync(binary, ['build-site-data', '--data', dir, '--as-of', date], {
      cwd: root, encoding: 'utf8', env: { ...process.env, CELEBTIR_SITE_URL: 'https://celebtir.com' }
    });
    fs.writeFileSync(path.join(work, `${path.basename(dir)}.log`), (run.stdout || '') + (run.stderr || ''));
    if (run.error || run.status !== 0) throw new Error(`Report regeneration failed: ${run.error || run.stderr}`);
  }
  const a = selectedManifest(baseline), b = selectedManifest(reduced);
  const changed = [...new Set([...Object.keys(a), ...Object.keys(b)])].filter(key => a[key] !== b[key]);
  validations.push({ input, reportFiles: Object.keys(a).length, byteIdentical: changed.length === 0, changed });
  fs.writeFileSync(path.join(work, 'report-hashes.json'), JSON.stringify({ baseline: a, reduced: b }, null, 2));
  removeWithin(work, baseline);
  removeWithin(work, reduced);
}
const originalsUnchanged = protectedDirs.every(dir => JSON.stringify(before[dir]) === JSON.stringify(manifest(path.join(root, dir))));
const inventory = [...bytes].sort((a, b) => b[1] - a[1]);
const result = { inventory, unreadable, validations, originalsUnchanged, binary, binarySha256: crypto.createHash('sha256').update(fs.readFileSync(binary)).digest('hex') };
fs.writeFileSync(path.join(output, 'validation.json'), JSON.stringify(result, null, 2) + '\n');
const selected = ['target', 'target/debug', 'target/qa', '.claude', 'site/node_modules', 'site/.next', 'site/out', 'data', 'output/qa-v02-data'];
const rows = selected.map(dir => `| ${dir} | ${(bytes.get(dir) || 0).toLocaleString('en-US')} | ${((bytes.get(dir) || 0) / 1048576).toFixed(2)} |`).join('\n');
fs.writeFileSync(path.join(output, 'REPORT.md'), `# Storage validation\n\nMeasured file lengths (not allocated disk blocks; linked files are skipped). No source data was deleted.\n\n| Directory | Bytes | MiB |\n| --- | ---: | ---: |\n${rows}\n\n${validations.map(v => `- ${v.input}: ${v.reportFiles} generated report inputs/API/feed files compared; byte-identical: ${v.byteIdentical}.`).join('\n')}\n- Original data, QA candidate, registry and methodology unchanged: ${originalsUnchanged}.\n- Inventory unreadable files: ${unreadable}.\n\n## Required information vs disposable storage\n\n- Keep current facts/scores and evidence, report histories/events/API/feeds, and the snapshot inputs currently required to regenerate those histories. Deleting old snapshots with the current implementation would erase chart history on the next build.\n- Keep decoded upgrade events and scanned-signature progress: they drive the UI upgrade history and allow bounded catch-up. They are not raw RPC responses.\n- Discard raw HIP-3 response files after decoding, and dependency transaction-cache entries after their results reach facts. This increases later RPC reads; existing request budgets remain in force.\n- Keep one active runtime matrix, source release and site export. Inactive/failed generations and compiler/Next build intermediates are disposable. Runtime/toolchain dependencies remain necessary to execute and rebuild.\n- target/ is Cargo compiler output, not collected data; it is not uploaded. The separate .claude worktree was only inventoried, not modified.\n\nThis validates the inspected fixtures and generated outputs, not a global mathematical minimum or a live collection. A full replacement of historical snapshots by an incremental report matrix requires a separate persistence migration with history/event parity tests.\n`);
console.log(JSON.stringify({ output, validations, originalsUnchanged, targetGiB: bytes.get('target') / 1073741824, dataMiB: bytes.get('data') / 1048576 }, null, 2));
if (!originalsUnchanged || validations.some(v => !v.byteIdentical) || unreadable) process.exitCode = 1;
