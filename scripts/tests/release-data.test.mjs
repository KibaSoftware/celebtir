import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {archiveSameDay,latestMethodology,assertCurrentRelease} from '../deploy/release-data.mjs';
import {trimData} from '../deploy/storage.mjs';

function fixture(t){
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'celebtir-release-'));
 t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const write=(p,s)=>{const f=path.join(root,p);fs.mkdirSync(path.dirname(f),{recursive:true});fs.writeFileSync(f,s);};
 for(const v of ['0.2','0.3','0.10'])for(const prefix of ['methodology','crates/celebtir-score/methodology'])write(`${prefix}/v${v}.toml`,`version = "${v}"\n`);
 return {root,write};
}
test('latest methodology uses numeric versions and binds to bundled rules',t=>{
 const {root,write}=fixture(t);assert.equal(latestMethodology(root).version,'0.10');
 write('crates/celebtir-score/methodology/v0.10.toml','version = "0.3"');
 assert.throws(()=>latestMethodology(root),/differ/);
});
test('same-day fresh deployment archives every original byte and never replaces an archive',t=>{
 const {root,write}=fixture(t),data=path.join(root,'data');
 for(const name of ['facts.json','scores.json','run.json','events.json','raw/source.json'])write(`data/snapshots/2026-10-28/${name}`,`original ${name}\n`);
 const result=archiveSameDay(data,'2026-10-28','20261028T120000Z-123');
 assert(!fs.existsSync(path.join(data,'snapshots/2026-10-28')));
 for(const name of ['facts.json','scores.json','run.json','events.json','raw/source.json'])assert.equal(fs.readFileSync(path.join(result,'snapshot',name),'utf8'),`original ${name}\n`);
 const manifest=JSON.parse(fs.readFileSync(path.join(result,'revision.json')));assert.equal(Object.keys(manifest.sha256).length,5);
 trimData(data);assert(fs.existsSync(path.join(result,'snapshot/raw/source.json')));
 for(const name of ['facts.json','scores.json','run.json'])write(`data/snapshots/2026-10-28/${name}`,'new');
 assert.throws(()=>archiveSameDay(data,'2026-10-28','20261028T120000Z-123'),/already exists/);
 assert.throws(()=>archiveSameDay(data,'../escape','20261028T120000Z-123'),/identity/);
});
test('incomplete original snapshots fail closed; first-day installs need no QA seed',t=>{
 const {root,write}=fixture(t);assert.equal(archiveSameDay(root,'2026-10-28','20261028T120000Z-123'),null);
 write('snapshots/2026-10-28/facts.json','{}');
 assert.throws(()=>archiveSameDay(root,'2026-10-28','20261028T120000Z-123'),/Incomplete/);
 assert(fs.existsSync(path.join(root,'snapshots/2026-10-28/facts.json')));
});
test('release gate rejects stale methodology or snapshot even if source code was updated',t=>{
 const {root,write}=fixture(t);
 write('data/latest/scores.json',JSON.stringify({methodology_version:'0.2'}));
 write('out/data/v2/summary.json',JSON.stringify({methodology_version:'0.2',snapshot_date:'2026-10-28'}));
 const check=()=>assertCurrentRelease(path.join(root,'data'),root,'2026-10-28',path.join(root,'out'));
 assert.throws(check,/stale/);
 write('data/latest/scores.json',JSON.stringify({methodology_version:'0.10'}));
 write('out/data/v2/summary.json',JSON.stringify({methodology_version:'0.10',snapshot_date:'2026-10-27'}));
 assert.throws(check,/snapshot/);
 write('out/data/v2/summary.json',JSON.stringify({methodology_version:'0.10',snapshot_date:'2026-10-28'}));
 assert.deepEqual(check(),{methodology_version:'0.10',snapshot_date:'2026-10-28'});
});
