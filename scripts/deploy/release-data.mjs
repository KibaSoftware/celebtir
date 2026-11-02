// Deployment-only revision handling. Published generations are never edited.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {pathToFileURL} from 'node:url';

export function latestMethodology(source) {
  const versions=fs.readdirSync(path.join(source,'methodology')).flatMap(name=>{
    const m=/^v(\d+)\.(\d+)(?:\.(\d+))?\.toml$/.exec(name);
    return m?[{name,parts:m.slice(1).map(n=>Number(n??0))}]:[];
  }).sort((a,b)=>a.parts[0]-b.parts[0]||a.parts[1]-b.parts[1]||a.parts[2]-b.parts[2]);
  const latest=versions.at(-1);
  if(!latest)throw new Error('No versioned methodology found');
  const file=path.join(source,'methodology',latest.name), bytes=fs.readFileSync(file);
  const version=/^version\s*=\s*"([0-9.]+)"\s*$/m.exec(bytes.toString())?.[1];
  if(`v${version}.toml`!==latest.name)throw new Error('Methodology filename/version mismatch');
  const bundled=fs.readFileSync(path.join(source,'crates/celebtir-score/methodology',latest.name));
  if(!bytes.equals(bundled))throw new Error('Published and bundled methodology differ');
  return {file,version};
}

function regularTree(dir) {
  if(fs.lstatSync(dir).isSymbolicLink())throw new Error('Linked snapshot/archive path');
  return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>{
    const p=path.join(dir,e.name);
    if(e.isSymbolicLink())throw new Error('Linked snapshot content');
    if(e.isDirectory())return regularTree(p);
    if(!e.isFile())throw new Error('Non-regular snapshot content');
    return [p];
  });
}

export function archiveSameDay(data,date,release) {
  if(!/^\d{4}-\d{2}-\d{2}$/.test(date)||!/^\d{8}T\d{6}Z-\d+$/.test(release))throw new Error('Invalid revision identity');
  const root=fs.realpathSync(data), snapshots=path.join(root,'snapshots');
  if(!fs.existsSync(snapshots))return null;
  if(fs.lstatSync(snapshots).isSymbolicLink())throw new Error('Linked snapshot root');
  const snapshot=path.join(snapshots,date);
  if(!fs.existsSync(snapshot))return null;
  const files=regularTree(snapshot);
  for(const name of ['facts.json','scores.json','run.json'])if(!files.includes(path.join(snapshot,name)))throw new Error(`Incomplete old snapshot: ${name}`);
  const archiveRoot=path.join(root,'snapshot-revisions');
  if(fs.existsSync(archiveRoot)&&fs.lstatSync(archiveRoot).isSymbolicLink())throw new Error('Linked archive root');
  fs.mkdirSync(archiveRoot,{recursive:true});
  const target=path.join(archiveRoot,`${date}-${release}`);
  if(fs.existsSync(target))throw new Error('Revision already exists');
  const hashes=Object.fromEntries(files.map(p=>[path.relative(snapshot,p).replaceAll('\\','/'),crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex')]));
  fs.mkdirSync(target);
  fs.writeFileSync(path.join(target,'revision.json'),JSON.stringify({original_snapshot_date:date,superseded_by_release:release,reason:'Explicit fresh deployment; original snapshot preserved',sha256:hashes},null,2)+'\n');
  fs.renameSync(snapshot,path.join(target,'snapshot'));
  return target;
}

export function assertCurrentRelease(data,source,date,exportDir) {
  const {version}=latestMethodology(source);
  const scores=JSON.parse(fs.readFileSync(path.join(data,'latest/scores.json')));
  const summary=JSON.parse(fs.readFileSync(path.join(exportDir,'data/v2/summary.json')));
  if(scores.methodology_version!==version||summary.methodology_version!==version)throw new Error(`Release must publish methodology ${version}; refusing stale scores`);
  if(summary.snapshot_date!==date)throw new Error(`Release must publish snapshot ${date}`);
  return {snapshot_date:date,methodology_version:version};
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
  const [command,...args]=process.argv.slice(2);
  if(command==='methodology')console.log(latestMethodology(args[0]).file);
  else if(command==='archive'){
    const candidate=fs.realpathSync(args[0]);
    if(path.dirname(candidate)!=='/opt/celebtir/shared/generations')throw new Error('Not an isolated runtime generation');
    const archive=archiveSameDay(...args);if(archive)console.log(`Preserved previous snapshot: ${path.basename(archive)}`);
  }else if(command==='check')console.log(JSON.stringify(assertCurrentRelease(...args)));
  else throw new Error('Expected methodology, archive or check');
}
