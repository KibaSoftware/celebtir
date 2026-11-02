// Read-only public HTTP reconnaissance; writes only the isolated review packet.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';

const root = process.cwd();
const out = 'output/reviews/score-review-2026-10-26';
if (fs.existsSync(`${out}/baseline-manifest.json`)) throw new Error('Baseline exists; preserve the original capture.');
fs.mkdirSync(`${out}/logs`, { recursive: true });
const sha = x => crypto.createHash('sha256').update(x).digest('hex');
const stable = x => Array.isArray(x) ? x.map(stable) : x && typeof x === 'object' ? Object.fromEntries(Object.keys(x).sort().map(k => [k, stable(x[k])])) : x;
const write = (p, v) => { fs.mkdirSync(path.dirname(p), {recursive:true}); fs.writeFileSync(p, typeof v === 'string' || Buffer.isBuffer(v) ? v : JSON.stringify(v,null,2)+'\n'); };
const files = p => fs.readdirSync(p,{withFileTypes:true}).flatMap(e=>e.isDirectory()?files(`${p}/${e.name}`):[`${p}/${e.name}`]);
const git = (...args) => execFileSync('git',args,{encoding:'utf8'}).trim();
const initial = { timestamp_utc:new Date().toISOString(), cwd:root, platform:process.platform, node:process.version, head:git('rev-parse','HEAD'), branch:git('branch','--show-current'), status:git('status','--short') };
write(`${out}/logs/reconnaissance.json`,initial);
const protectedFiles = ['data','output/qa-v02-data','output/qa-v03-data','methodology','registry','fixtures','crates/celebtir-score'].flatMap(files);
write(`${out}/protected-before.json`,Object.fromEntries(protectedFiles.map(p=>[p,sha(fs.readFileSync(p))])));
const localSummary=JSON.parse(fs.readFileSync('site/out/data/v2/summary.json'));
const checks=[];
const captures=[];
const targets=['/','/risk/','/value/','/activity/','/tokens/','/compare/','/changes/','/controllers/','/hyperevm/','/hip3/','/methodology/','/about/','/api/','/llms.txt','/manifest.webmanifest', ...files('site/out/data/v2').map(p=>p.replace('site/out',''))];
for(let n=0;n<targets.length;n+=4) await Promise.all(targets.slice(n,n+4).map(async route=>{
 const hosted_url=`https://celebtir.com${route}`;
 const relative=route==='/'?'index.html':route.endsWith('/')?`${route.slice(1)}index.html`:route.slice(1);
 const local_path=`site/out/${relative}`;
 let http_status=null;
 try {
  const response=await fetch(hosted_url,{signal:AbortSignal.timeout(30000)});http_status=response.status;
  const bytes=Buffer.from(await response.arrayBuffer());
  write(`${out}/hosted/${relative}`,bytes);
  captures.push({route,http_status,final_url:response.url,captured_at_utc:new Date().toISOString(),raw_sha256:sha(bytes),headers:Object.fromEntries(['content-type','last-modified','access-control-allow-origin','etag'].map(k=>[k,response.headers.get(k)]))});
  if(!response.ok) { checks.push({local_path,hosted_url,http_status,logical_hash:null,raw_sha256:sha(bytes),result:'NOT_FETCHED',difference:`HTTP ${http_status}`});return; }
  if(fs.existsSync(local_path)) {
   const local=fs.readFileSync(local_path);write(`${out}/local-export/${relative}`,local);
   if(route.endsWith('.json')) {
    const h=JSON.parse(bytes),l=JSON.parse(local),hs=JSON.stringify(stable(h)),ls=JSON.stringify(stable(l));
    const match=hs===ls;
    checks.push({local_path,hosted_url,http_status,logical_hash:sha(hs),local_logical_hash:sha(ls),raw_sha256:sha(bytes),local_raw_sha256:sha(local),result:match?'MATCH':'MISMATCH',difference:match?null:'JSON values or original array order differ; no fields or array ordering normalized away.',array_order_preserved:true});
   }
  }
 }catch(e){checks.push({local_path,hosted_url,http_status,logical_hash:null,raw_sha256:null,result:'NOT_FETCHED',difference:String(e.cause?.message??e.message)});}
}));
checks.sort((a,b)=>a.hosted_url.localeCompare(b.hosted_url));captures.sort((a,b)=>a.route.localeCompare(b.route));
write(`${out}/http-captures.json`,captures);
const summaryPath=`${out}/hosted/data/v2/summary.json`;
const hosted=fs.existsSync(summaryPath)?JSON.parse(fs.readFileSync(summaryPath)):null;
const counts={};for(const p of hosted?.protocols??[]) {const k=`${p.chain}:${p.assessment}:${p.stage??'no_stage'}`;counts[k]=(counts[k]??0)+1;}
const version=hosted?.methodology_version??null;
const manifest={contract_version:'1.1.1',review_date:'2026-10-27',repository_head:initial.head,repository_dirty_before_review:!!initial.status,hosted_base_url:'https://celebtir.com/',hosted_fetch_status:!hosted?'BLOCKED_NETWORK':checks.some(c=>c.result==='MISMATCH')?'MISMATCH':'PASS',hosted_build_id:null,local_build_id:fs.existsSync('site/.next/BUILD_ID')?fs.readFileSync('site/.next/BUILD_ID','utf8').trim():null,snapshot_id:hosted?.snapshot_date??null,snapshot_date_utc:hosted?.snapshot_date??null,methodology_version:version,methodology_hash:version?sha(fs.readFileSync(`methodology/v${version}.toml`)):null,source_data_directory:`${out}/hosted/data/v2`,local_source_data_directory:'output/qa-v03-data',local_methodology_version:localSummary.methodology_version,domain_counts:counts,artifact_checks:checks,discrepancies:hosted&&version!==localSummary.methodology_version?[`Hosted methodology ${version}; local candidate ${localSummary.methodology_version}. These are distinct baselines; do not rescore hosted history.`]:[],blocking_findings:['Hosted build commit and workflow run are NOT_EXPOSED.','Full hosted raw snapshot/run and HIP-3 JSON are not exposed by the discovered public API; HTML observations remain separate.'],capture_timestamp_utc:initial.timestamp_utc};
write(`${out}/baseline-manifest.json`,manifest);
console.log(JSON.stringify({hosted_fetch_status:manifest.hosted_fetch_status,methodology_version:version,counts,artifacts:checks.length,mismatches:checks.filter(c=>c.result==='MISMATCH').length}));
