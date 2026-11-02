import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { trimData, pruneChildren, removeWithin, pruneRuntime, collectionStatus, assertExportable } from '../deploy/storage.mjs';

test('a chain outage cannot publish an empty token or protocol route set', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-export-'));
  const file = path.join(root, 'facts.json');
  for (const facts of [{protocols: {p: {}}, tokens: {}}, {protocols: {}, tokens: {t: {}}}, {}]) {
    fs.writeFileSync(file, JSON.stringify(facts));
    assert.throws(() => assertExportable(file), /Refusing publication/);
  }
  fs.writeFileSync(file, JSON.stringify({protocols: {p: {}}, tokens: {t: {}}}));
  assert.doesNotThrow(() => assertExportable(file));
});

test('same-day rebuild retains partial or budget failure and rejects missing reports', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-status-'));
  const file = path.join(root, 'run.json');
  assert.throws(() => collectionStatus(file));
  for (const [report, expected] of [[{failures: []}, 0], [{failures: ['unavailable']}, 2], [{failures: [], budget_exceeded: true}, 2]]) {
    fs.writeFileSync(file, JSON.stringify(report));
    assert.equal(collectionStatus(file), expected);
  }
  fs.writeFileSync(file, '{}');
  assert.throws(() => collectionStatus(file), /Invalid/);
});

test('discard transport data and dependency cache while preserving report facts and upgrade progress', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-storage-'));
  const snapshot = path.join(root, 'snapshots/2026-10-23');
  fs.mkdirSync(path.join(snapshot, 'raw'), { recursive: true });
  fs.mkdirSync(path.join(root, 'cache'));
  fs.writeFileSync(path.join(snapshot, 'raw/perpDexs.json'), '{"transport":true}');
  fs.writeFileSync(path.join(snapshot, 'facts.json'), '{"evidence":"preserved"}\n');
  const upgrades = { 'program:signature': [{ slot: 42 }], 'program:scanned-empty': [] };
  fs.writeFileSync(path.join(root, 'cache/tx-cache.json'), JSON.stringify({ by_signature: { tx: ['callee'] }, upgrades }));
  trimData(root);
  assert.equal(fs.existsSync(path.join(snapshot, 'raw')), false);
  assert.equal(fs.readFileSync(path.join(snapshot, 'facts.json'), 'utf8'), '{"evidence":"preserved"}\n');
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(root, 'cache/tx-cache.json'))), { by_signature: {}, upgrades });
});

test('prune only recognized inactive deployment directories and reject escaped paths', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-prune-'));
  const keep = path.join(root, '20261023T120000Z-123');
  const old = path.join(root, '20261022T120000Z-456');
  const unknown = path.join(root, 'owner-backup');
  for (const dir of [keep, old, unknown]) fs.mkdirSync(dir);
  pruneChildren(root, [keep]);
  assert.equal(fs.existsSync(keep), true);
  assert.equal(fs.existsSync(old), false);
  assert.equal(fs.existsSync(unknown), true);
  assert.throws(() => removeWithin(root, path.dirname(root)), /Refusing/);
});

test('repeated updates retain exactly one known generation', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-growth-'));
  for (let i = 1; i <= 30; i++) {
    const current = path.join(root, `20261023T120000Z-${i}`);
    fs.mkdirSync(current);
    fs.writeFileSync(path.join(current, 'report.json'), '{}');
    pruneChildren(root, [current]);
    assert.deepEqual(fs.readdirSync(root), [path.basename(current)]);
  }
});

test('missing or outside active generation fails before deleting anything', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-protection-'));
  const old = path.join(root, '20261023T120000Z-1');
  fs.mkdirSync(old);
  assert.throws(() => pruneChildren(root, []), /protected generation/);
  assert.throws(() => pruneChildren(root, [path.dirname(root)]), /protected generation/);
  assert.throws(() => pruneChildren(root, [path.join(root, 'missing')]));
  assert.equal(fs.existsSync(old), true);
});

test('unverified publication prevents all pruning', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-pending-'));
  fs.mkdirSync(path.join(root, 'shared'));
  fs.writeFileSync(path.join(root, 'shared/publication.pending'), 'pending');
  assert.throws(() => pruneRuntime(root, true), /verification pending/);
});

test('linked deletion targets are rejected without changing their destination', t => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-links-'));
  const outside = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-outside-'));
  fs.writeFileSync(path.join(outside, 'evidence.json'), '{}');
  const link = path.join(root, '20261023T120000Z-1');
  try { fs.symlinkSync(outside, link, process.platform === 'win32' ? 'junction' : 'dir'); }
  catch (error) { if (error.code === 'EPERM') { t.skip('Native links unavailable'); return; } throw error; }
  assert.throws(() => removeWithin(root, link), /linked deletion/);
  assert.equal(fs.readFileSync(path.join(outside, 'evidence.json'), 'utf8'), '{}');
});

test('invalid cache fails rather than deleting upgrade progress', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'celebtir-invalid-cache-'));
  fs.mkdirSync(path.join(root, 'cache'));
  const cache = path.join(root, 'cache/tx-cache.json');
  fs.writeFileSync(cache, 'not json');
  assert.throws(() => trimData(root));
  assert.equal(fs.readFileSync(cache, 'utf8'), 'not json');
});
