import fs from 'node:fs';
import crypto from 'node:crypto';
import ts from '../site/node_modules/typescript/lib/typescript.js';
const code=ts.transpileModule(fs.readFileSync('site/src/lib/scoreReview.ts','utf8'),{compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}).outputText;
const {reviewCriteria}=await import('data:text/javascript;base64,'+Buffer.from(code).toString('base64'));
const root='output/reviews/score-review-2026-10-26', docs='docs/research/score-review';
fs.mkdirSync(docs,{recursive:true});
const read=p=>JSON.parse(fs.readFileSync(p));
const stable=x=>Array.isArray(x)?x.map(stable):x&&typeof x==='object'?Object.fromEntries(Object.keys(x).sort().map(k=>[k,stable(x[k])])):x;
const equal=(a,b)=>JSON.stringify(stable(a))===JSON.stringify(stable(b));
const sha=x=>crypto.createHash('sha256').update(JSON.stringify(stable(x))).digest('hex');
const summary=read(`${root}/hosted/data/v2/summary.json`), replay=read(`${root}/replay/scores.json`);
const protocols=[],anomalies=[],evidence_index={};
for(const row of summary.protocols.filter(p=>p.chain==='solana').sort((a,b)=>a.id<b.id?-1:1)){
 const p=read(`${root}/hosted/data/v2/protocols/${row.id}.json`),score=p.score;
 const all=reviewCriteria(p.facts,score);
 const relevant=all.filter(c=>c.requirement.stage===score.stage.next);
 const criteria=relevant.map(c=>{
   for(const e of c.evidence)evidence_index[sha(e)]=e;
   if(c.uncertain)anomalies.push({protocol_id:p.id,criterion_id:c.requirement.id,code:'CAUSE_CLASSIFICATION_UNRESOLVED',reason:'No source-supported cause can be assigned; no guessed catch-all.'});
   return {criterion_id:c.requirement.id,criterion_text:c.requirement.description,canonical_source_path:'crates/celebtir-score/src/stage.rs',canonical_position:score.stage.requirements.findIndex(r=>r.id===c.requirement.id),evaluator_result:c.requirement.met,audit_status:c.status,classification_uncertain:c.uncertain,cause_codes:c.causeCodes,evidence_ids:c.evidence.map(sha),source_account_ids:[...new Set(c.evidence.map(e=>e.address))].sort(),actual_observation:c.requirement.detail,required_value:c.requirement.description,causal_paths:c.causes.map(x=>({...x,evidence_ids:x.evidence.map(sha)}))};
 });
 const published_output_match=equal(score,replay.protocols[p.id]);
 if(!published_output_match)anomalies.push({protocol_id:p.id,code:'SOURCE_OUTPUT_MISMATCH',fields:Object.keys(score).filter(k=>!equal(score[k],replay.protocols[p.id]?.[k]))});
 const v=criteria.some(c=>c.audit_status==='UNMET_VERIFIED'),u=criteria.some(c=>c.audit_status==='UNRESOLVED_EVIDENCE');
 protocols.push({protocol_id:p.id,name:p.facts.name,domain:'solana',category:p.facts.category,snapshot_id:p.snapshot_date,methodology_version:p.methodology_version,published_stage:score.stage.stage,published_next_stage:score.stage.next,next_stage:score.stage.stage==='unrated'?null:score.stage.next,core_program_ids:Object.entries(p.facts.programs).filter(([,x])=>x.role==='core').map(([a])=>a),authority_roots:Object.entries(p.facts.programs).filter(([,x])=>x.role==='core').map(([a,x])=>({program:a,controller:x.upgrade_control})),factors:Object.entries(score.factors).map(([factor_id,f])=>{for(const e of f.evidence)evidence_index[sha(e)]=e;return {factor_id,published_rating:f.rating,reason:f.reason,evidence_ids:f.evidence.map(sha)};}),criteria,published_output_match,determination:v&&u?'MIXED':v?'VERIFIED_CONTROL_BOTTLENECK':u?'EVIDENCE_LIMITED':'UNDETERMINED'});
}
const distribution={};for(const p of protocols)distribution[p.published_stage]=(distribution[p.published_stage]??0)+1;
const criterionIds=[...new Set(protocols.flatMap(p=>p.criteria.map(c=>c.criterion_id)))];
const criteria_counts=criterionIds.map(id=>{const cs=protocols.flatMap(p=>p.criteria.filter(c=>c.criterion_id===id));return {criterion_id:id,eligible:cs.filter(c=>c.audit_status!=='NOT_APPLICABLE').length,verified:cs.filter(c=>c.audit_status==='UNMET_VERIFIED').length,unresolved:cs.filter(c=>c.audit_status==='UNRESOLVED_EVIDENCE').length};});
const cause_counts=[...new Set(protocols.flatMap(p=>p.criteria.flatMap(c=>c.cause_codes)))].sort().map(cause_code=>({cause_code,protocol_ids:protocols.filter(p=>p.criteria.some(c=>c.cause_codes.includes(cause_code))).map(p=>p.protocol_id)})).map(c=>({...c,count:c.protocol_ids.length}));
const report={contract_version:'1.1.1',snapshot_id:summary.snapshot_date,methodology_version:summary.methodology_version,scope:'solana_published_protocols_only',protocols,distribution,criteria_counts,cause_counts,evidence_index,anomalies,limitations:['Hosted v0.2 differs from the owner-expected v0.3 candidate.','Replay reconstructs only publicly exported facts; null metrics are absent, not zero. HIP-3 raw JSON is unavailable.','Criterion classifications are review annotations, never alternate Stage assignments.','Missing binary verification is unresolved evidence, not proof of a binary mismatch.','Unrated review next_stage is null per contract; published_next_stage retains Stage 0. Stage 0 entry criteria remain visible.','Counterfactual overrides: BLOCKED_UNSUPPORTED; no safe criterion-override replay API. Structural frequencies follow instead.']};
fs.writeFileSync(`${root}/stage-diagnostics.json`,JSON.stringify(stable(report),null,2)+'\n');
const percent=(n,d)=>d?((BigInt(n)*10000n*2n+BigInt(d))/(2n*BigInt(d))).toString().padStart(3,'0').replace(/(..)$/,'.$1')+'%':'N/A';
let md=`# Stage diagnostics\n\nHosted snapshot ${summary.snapshot_date}; methodology ${summary.methodology_version}. All ${protocols.length} Solana records are included once. Distribution: ${JSON.stringify(distribution)}. Native HyperEVM and HIP-3 are excluded.\n\nCurrent scorer replay agrees with ${protocols.filter(p=>p.published_output_match).length}/${protocols.length} complete published protocol score objects.\n\n| Protocol | Stage | Verified unmet | Unresolved | Determination |\n|---|---|---:|---:|---|\n`;
for(const p of protocols)md+=`| ${p.protocol_id} | ${p.published_stage} | ${p.criteria.filter(c=>c.audit_status==='UNMET_VERIFIED').length} | ${p.criteria.filter(c=>c.audit_status==='UNRESOLVED_EVIDENCE').length} | ${p.determination} |\n`;
md+='\n## Criterion frequencies\n\nMultiple blockers can occur per protocol: percentages can sum above 100%. Denominators include unresolved eligible protocols and exclude explicit exemptions. Percentages use integer half-up rounding to two decimal places.\n\n| Criterion | Verified / eligible | Unresolved / eligible |\n|---|---|---|\n';
for(const c of criteria_counts)md+=`| ${c.criterion_id} | ${c.verified}/${c.eligible} (${percent(c.verified,c.eligible)}) | ${c.unresolved}/${c.eligible} (${percent(c.unresolved,c.eligible)}) |\n`;
md+='\n## Source traces\n';for(const p of protocols){md+=`\n### ${p.name}\n\n${p.factors.map(f=>`${f.factor_id}: **${f.published_rating}** — ${f.reason}`).join('\n\n')}\n`;for(const c of p.criteria)md+=`\n- **${c.audit_status}** ${c.criterion_text}: ${c.actual_observation}. Causes: ${c.cause_codes.join(', ')||'none assigned'}. ${c.evidence_ids.map(id=>{const e=evidence_index[id];return `[${e.field}, ${e.slot}](${e.explorer_url})`;}).join(' ')}\n`;}
md+='\n## Separate blocker analyses\n\n### Verified blockers\n\n'+criteria_counts.filter(c=>c.verified).sort((a,b)=>b.verified-a.verified||a.criterion_id.localeCompare(b.criterion_id)).map(c=>`- ${c.criterion_id}: ${c.verified}/${c.eligible}; protocols: ${protocols.filter(p=>p.criteria.some(x=>x.criterion_id===c.criterion_id&&x.audit_status==='UNMET_VERIFIED')).map(p=>p.protocol_id).join(', ')}`).join('\n');
md+='\n\n### Unresolved evidence\n\n'+criteria_counts.filter(c=>c.unresolved).sort((a,b)=>b.unresolved-a.unresolved||a.criterion_id.localeCompare(b.criterion_id)).map(c=>`- ${c.criterion_id}: ${c.unresolved}/${c.eligible}; protocols: ${protocols.filter(p=>p.criteria.some(x=>x.criterion_id===c.criterion_id&&x.audit_status==='UNRESOLVED_EVIDENCE')).map(p=>p.protocol_id).join(', ')}`).join('\n');
md+='\n\n### Mixed cases\n\n'+protocols.filter(p=>p.determination==='MIXED').map(p=>`- ${p.protocol_id}: both verified unmet and unresolved next-level criteria (see source trace).`).join('\n');
md+='\n\n## All status counts\n\n| Protocol | Factor state counts | Criterion status counts |\n|---|---|---|\n';
const counts=xs=>xs.reduce((a,k)=>(a[k]=(a[k]??0)+1,a),{});
for(const p of protocols)md+=`| ${p.protocol_id} | ${JSON.stringify(counts(p.factors.map(f=>f.published_rating)))} | ${JSON.stringify(counts(p.criteria.map(c=>c.audit_status)))} |\n`;
md+='\n## Distinct-protocol cause counts and representative paths\n\n';
for(const c of cause_counts){const p=protocols.find(p=>p.criteria.some(x=>x.cause_codes.includes(c.cause_code)));const criterion=p.criteria.find(x=>x.cause_codes.includes(c.cause_code));const path=criterion.causal_paths.find(x=>x.code===c.cause_code);md+=`- **${c.cause_code}**: ${c.count} distinct protocols (${c.protocol_ids.join(', ')}). Example ${p.protocol_id}/${criterion.criterion_id}, account ${path.account}: ${path.observation}. Evidence IDs: ${path.evidence_ids.join(', ')}. Full controller tree/voting permissions/delay/configuration path is in this protocol's authority_roots in the diagnostic JSON.\n`;}
md+='\nThe current canonical Stage rubric has no per-criterion exemption field; factor N/A is not a criterion exemption. NOT_APPLICABLE criterion cases are therefore NOT_OBSERVED, not fabricated. Stage 1 and Stage 2 counts are zero. Counterfactual sensitivity: BLOCKED_UNSUPPORTED; the structural criterion frequencies above are the supported alternative. Native HyperEVM and HIP-3 do not enter these denominators.\n';
md+='\n## Limitations and anomalies\n\n'+report.limitations.map(x=>'- '+x).join('\n')+'\n\n'+anomalies.map(a=>`- ${a.protocol_id} ${a.criterion_id??''}: ${a.code}; ${a.reason??a.fields.join(', ')}`).join('\n')+'\n';
fs.writeFileSync(`${docs}/stage-diagnostics.md`,md);
console.log(JSON.stringify({distribution,matches:protocols.filter(p=>p.published_output_match).length,verified:criteria_counts.reduce((n,c)=>n+c.verified,0),unresolved:criteria_counts.reduce((n,c)=>n+c.unresolved,0),anomalies}));
