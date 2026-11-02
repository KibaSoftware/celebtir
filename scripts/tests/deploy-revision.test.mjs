// Optional real CLI integration, fully offline. No host provisioning/publication.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {archiveSameDay} from '../deploy/release-data.mjs';

test('real offline redeployment archives v0.2 and collects/scores fresh v0.3 on the same date', {skip:!process.env.CELEBTIR_TEST_BIN}, t=>{
 const data=fs.mkdtempSync(path.join(os.tmpdir(),'celebtir-revision-integration-'));
 t.after(()=>fs.rmSync(data,{recursive:true,force:true}));
 const slash=p=>path.resolve(p).replaceAll('\\','/');
 function run(version){
  const args=['scripts/daily.sh','--date','2026-10-28','--data',slash(data),'--registry',slash('fixtures/golden/registry'),'--bin',slash(process.env.CELEBTIR_TEST_BIN),'--methodology',slash(`methodology/v${version}.toml`),'--offline',slash('fixtures/golden/replay'),'--tx-sample','5','--history-limit','5'];
  const result=spawnSync(process.env.CELEBTIR_TEST_BASH??'bash',args,{encoding:'utf8',env:{...process.env,CELEBTIR_RPC_URL:''}});
  assert.equal(result.status,0,result.stderr||result.error?.message||result.stdout);
 }
 run('0.2');
 const original=fs.readFileSync(path.join(data,'snapshots/2026-10-28/scores.json'));
 const archive=archiveSameDay(data,'2026-10-28','20261028T120000Z-123');
 run('0.3');
 assert.equal(JSON.parse(fs.readFileSync(path.join(data,'latest/scores.json'))).methodology_version,'0.3');
 assert(fs.readFileSync(path.join(archive,'snapshot/scores.json')).equals(original));
 assert.equal(JSON.parse(original).methodology_version,'0.2');
 const revision=JSON.parse(fs.readFileSync(path.join(archive,'revision.json')));
 assert(revision.sha256['facts.json']);assert(revision.sha256['scores.json']);
});
