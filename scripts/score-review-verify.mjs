// Offline, read-only checks of preserved inputs and generated review artifacts.
import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
const root='output/reviews/score-review-2026-10-26';
const read=p=>JSON.parse(fs.readFileSync(p,'utf8'));
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const walk=p=>fs.readdirSync(p,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(`${p}/${e.name}`):[`${p}/${e.name}`]);
const stable=x=>Array.isArray(x)?x.map(stable):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,stable(x[k])])):x;
const equal=(a,b)=>JSON.stringify(stable(a))===JSON.stringify(stable(b));
const bag=x=>Array.isArray(x)?x.map(bag).sort((a,b)=>JSON.stringify(a).localeCompare(JSON.stringify(b))):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,bag(x[k])])):x;
const before=read(`${root}/protected-before.json`), changed=Object.keys(before).filter(p=>!fs.existsSync(p)||hash(p)!==before[p]);
assert.deepEqual(changed,[],'Protected data/registry/methodology/scorer changed');
const diagnostic=`${root}/stage-diagnostics.json`, initial=hash(diagnostic);
execFileSync(process.execPath,['scripts/score-review-report.mjs']);
assert.equal(hash(diagnostic),initial,'Diagnostic output must be byte deterministic');
const d=read(diagnostic);
assert.equal(d.protocols.length,12);assert.equal(new Set(d.protocols.map(p=>p.protocol_id)).size,12);
for(const p of d.protocols){
 assert.equal(p.published_output_match,true);
 for(const c of p.criteria){
  assert.deepEqual(c.cause_codes,[...new Set(c.cause_codes)].sort());
  if(c.classification_uncertain)assert(d.anomalies.some(a=>a.protocol_id===p.protocol_id&&a.criterion_id===c.criterion_id&&a.code==='CAUSE_CLASSIFICATION_UNRESOLVED'));
 }
}
for(const c of d.cause_counts)assert.equal(c.count,new Set(d.protocols.filter(p=>p.criteria.some(x=>x.cause_codes.includes(c.cause_code))).map(p=>p.protocol_id)).size);
const original=read(`${root}/hosted/data/v2/protocols/orca.json`).score, changedFixture=structuredClone(original);
changedFixture.stage.stage='stage_2';assert(!equal(original,changedFixture),'Differential comparison must detect a changed Stage');
assert(equal(original,read(`${root}/replay/scores.json`).protocols.orca));
const comparisons=walk(`${root}/hosted/data/v2`).filter(p=>p.endsWith('.json')).flatMap(p=>{
 const relative=p.slice(`${root}/hosted/`.length), local=`${root}/local-export/${relative}`;
 if(!fs.existsSync(local))return [];
 const a=read(p),b=read(local),values_equal_ignoring_array_order=equal(bag(a),bag(b));
 return [{path:relative,ordered_json_equal:equal(a,b),values_equal_ignoring_array_order,array_order_only_difference:values_equal_ignoring_array_order&&!equal(a,b)}];
});
fs.writeFileSync(`${root}/artifact-equivalence.json`,JSON.stringify({note:'Object keys normalized. Ordered comparison preserves arrays; multiset comparison isolates order-only changes and never declares different values equivalent.',comparisons},null,2)+'\n');
const apiChanges=walk(`${root}/local-export/data/v2`).filter(p=>p.endsWith('.json')).filter(p=>{const output=p.replace(`${root}/local-export`,'site/out');return !fs.existsSync(output)||!equal(read(p),read(output));});
assert.deepEqual(apiChanges,[],'Review changed candidate published API values');
const reports=[...walk('docs/research/score-review'),...walk(root).filter(p=>/\.(json|log|md|txt)$/.test(p))];
// High-confidence credential patterns only: never print matching values.
const secretPatterns=[/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,/\bAKIA[0-9A-Z]{16}\b/,/\bgh[pousr]_[A-Za-z0-9]{30,}\b/,/https?:\/\/[^\s"<>]+[?&](?:api[_-]?key|token|secret)=[A-Za-z0-9_-]{16,}/i];
const flagged=reports.filter(p=>secretPatterns.some(re=>re.test(fs.readFileSync(p,'utf8'))));
const result={executed_at_utc:new Date().toISOString(),protected_files_checked:Object.keys(before).length,protected_changed:changed,diagnostic_sha256:initial,byte_deterministic:true,protocol_score_matches:12,cause_deduplication:'PASS',mismatch_detection_fixture:'PASS',candidate_api_unchanged:true,secret_scan:{files_checked:reports.length,status:flagged.length?'BLOCKED':'PASS',flagged_file_count:flagged.length,scope:'New research reports, public capture text and log/JSON metadata; high-confidence key and credential URL patterns. PNG binary pixels are not an OCR scan.'}};
fs.writeFileSync(`${root}/verification.json`,JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result));assert.equal(flagged.length,0,'Credential scan requires review; values withheld');
