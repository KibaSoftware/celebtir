// Runtime storage policy. Never run this against the development repository.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export function collectionStatus(file) {
  const report = JSON.parse(fs.readFileSync(file, 'utf8'));
  if (!Array.isArray(report.failures)) throw new Error('Invalid collection report');
  return report.failures.length || report.budget_exceeded ? 2 : 0;
}

export function assertExportable(file) {
  const facts = JSON.parse(fs.readFileSync(file, 'utf8'));
  for (const kind of ['protocols', 'tokens']) {
    if (!facts[kind] || typeof facts[kind] !== 'object' || !Object.keys(facts[kind]).length) {
      throw new Error(`Refusing publication: no ${kind} collected. Inspect collection failures and RPC authentication; the current static export requires nonempty entity routes.`);
    }
  }
}

export function removeWithin(parent, target) {
  const base = fs.realpathSync(parent);
  const absolute = path.resolve(target);
  if (path.dirname(absolute) !== base) throw new Error(`Refusing removal outside ${base}: ${absolute}`);
  if (!fs.existsSync(absolute)) return;
  const stat = fs.lstatSync(absolute);
  if (stat.isSymbolicLink() || fs.realpathSync(absolute) !== absolute) throw new Error(`Refusing linked deletion: ${absolute}`);
  fs.rmSync(absolute, { recursive: true, force: false });
}

export function trimData(data) {
  const root = fs.realpathSync(data);
  // Raw HIP-3 transport responses have already been decoded into evidenced facts.
  const snapshots = path.join(root, 'snapshots');
  if (fs.existsSync(snapshots)) {
    for (const name of fs.readdirSync(snapshots)) {
      if (!/^\d{4}-\d{2}-\d{2}$/.test(name)) continue;
      const dir = path.join(snapshots, name);
      if (fs.lstatSync(dir).isSymbolicLink()) throw new Error('Linked snapshot directory');
      removeWithin(dir, path.join(dir, 'raw'));
    }
  }
  const cache = path.join(root, 'cache', 'tx-cache.json');
  if (fs.existsSync(cache)) {
    if (fs.lstatSync(path.dirname(cache)).isSymbolicLink() || fs.lstatSync(cache).isSymbolicLink()) throw new Error('Linked cache');
    const parsed = JSON.parse(fs.readFileSync(cache, 'utf8'));
    // Upgrade history and scanned-signature progress drive UI coverage and must
    // survive. Recent dependency callees are already in facts; refetch next run.
    parsed.by_signature = {};
    fs.writeFileSync(cache, JSON.stringify(parsed) + '\n');
  }
}

export function pruneChildren(parent, keep) {
  const base = fs.realpathSync(parent);
  const protectedPaths = new Set(keep.map(p => fs.realpathSync(p)));
  if (!protectedPaths.size || [...protectedPaths].some(p => path.dirname(p) !== base)) throw new Error('Missing or escaped protected generation');
  for (const name of fs.readdirSync(base)) {
    // Delete only directories created by this deployment's known naming scheme.
    if (!/^(?:seed-)?\d{8}T\d{6}Z-(?:[a-f0-9]{8}|\d+)$/.test(name)) continue;
    const child = path.join(base, name);
    if (protectedPaths.has(child)) continue;
    removeWithin(base, child);
  }
}

export function pruneRuntime(root, sources = false) {
  const base = fs.realpathSync(root);
  if (fs.existsSync(path.join(base, 'shared/publication.pending'))) throw new Error('Publication verification pending; refusing to prune');
  const currentData = fs.realpathSync(path.join(base, 'shared/data'));
  const currentWeb = fs.realpathSync(path.join(base, 'current'));
  // Resolve all protected links before any deletion; missing links fail closed.
  if (path.dirname(currentData) !== path.join(base, 'shared/generations') || path.dirname(currentWeb) !== path.join(base, 'web')) throw new Error('Unexpected active paths');
  const currentSource = sources ? fs.realpathSync(path.join(base, 'source')) : null;
  if (sources && path.dirname(currentSource) !== path.join(base, 'releases')) throw new Error('Unexpected source path');
  pruneChildren(path.join(base, 'shared/generations'), [currentData]);
  pruneChildren(path.join(base, 'web'), [currentWeb]);
  if (sources) pruneChildren(path.join(base, 'releases'), [currentSource]);
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  const [command, argument] = process.argv.slice(2);
  const root = '/opt/celebtir';
  if (command === 'check-export') {
    assertExportable(argument);
  } else if (command === 'status') {
    console.log(collectionStatus(argument));
  } else if (command === 'trim') {
    const candidate = fs.realpathSync(argument);
    if (path.dirname(candidate) !== `${root}/shared/generations`) throw new Error('Not a runtime candidate');
    trimData(candidate);
  } else if (command === 'prune') {
    pruneRuntime(root, argument === 'sources');
  } else if (command === 'discard') {
    removeWithin(`${root}/shared/generations`, path.resolve(argument));
  } else if (command === 'discard-web') {
    removeWithin(`${root}/web`, path.resolve(argument));
  } else throw new Error('usage: storage.mjs trim <candidate> | prune [sources] | discard <candidate> | discard-web <export>');
}
