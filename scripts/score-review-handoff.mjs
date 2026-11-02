import fs from 'node:fs';
import crypto from 'node:crypto';
const root='output/reviews/score-review-2026-10-26';
const now=new Date().toISOString();
const write=(name,x)=>fs.writeFileSync(`${root}/${name}`,JSON.stringify(x,null,2)+'\n');
const gates=[
 ['G0','BLOCKED','Public HTTP captures and hosted route/API reconciliation','0.2','baseline-manifest.json','Owner expected v0.3; public artifact remains v0.2. Deployed source/run binding is not exposed. Browser/API are internally consistent.'],
 ['G1','BLOCKED','Rust v0.2 replay; node scripts/score-review-report.mjs; node scripts/score-review-verify.mjs','0.2','stage-diagnostics.json','12/12 exact score matches; three CAUSE_CLASSIFICATION_UNRESOLVED anomalies. No safe counterfactual override interface.'],
 ['G2','PASS','pnpm.cmd -C site test; pnpm.cmd -C site e2e (through gates.sh)','0.3','logs/gates-final.log','Exact copy, published-score preservation, Stage keyboard/axe/viewport checks pass. Native zoom/owner acceptance remain G5 limitations.'],
 ['G3','PASS','Review/protection unit cases and score-review.spec.ts (through gates.sh)','0.3','logs/gates-final.log','Three-valued filters, unknown-last stable sorting, scope/version separation, URL reload/reset and exact observation evidence pass.'],
 ['G4','PASS','Public primary-source GET and four bounded finalized getMultipleAccounts batches; node scripts/score-review-dossiers.mjs','0.2','research/chain-observations.json','Five required dossiers produced with gaps; two REASSESS_EXISTING, three DRAFT_RESEARCH_ONLY. No admissions or publication authorization. Additional Solstice capture in research-solstice/.'],
 ['G5','FAIL','bash scripts/gates.sh; opt-in hosted browser; pnpm.cmd -C site lighthouse; artifact verification','0.3','logs/lighthouse-final.log','Full local regression passes; existing Lighthouse performance thresholds fail. Native browser zoom and host publication/source/timer binding remain unverified. Hosted checks separately use v0.2.'],
 ['G6','BLOCKED','Code inspection and docs/research/score-review/ethereum-admission.md','not_applicable','gate-results.json','Prerequisites unresolved. NO_RATING_UNTIL_METHODOLOGY_REVIEW; no Ethereum implementation.'],
].map(([gate_id,status,command_or_check,methodology_version,log,reason])=>({gate_id,status,command_or_check,input_snapshot_id:'2026-10-26',methodology_version,executed_at_utc:now,log_path:`${root}/${log}`,reason}));
write('gate-results.json',{overall_status:'FAIL',execution_recorded_at_utc:now,note:'Individual capture timestamps and execution logs remain authoritative; this timestamp records gate reconciliation.',gates});
const paths={
 'AGENTS.md':'Current decision/status and blockers',
 'site/src/lib/scoreReview.ts':'Pure criterion review and shared Stage wording',
 'site/src/lib/protections.ts':'Evidence-derived, domain-scoped three-valued protection filtering',
 'site/src/lib/scoreReview.test.ts':'Review, filter, comparator and candidate-readiness regression cases',
 'site/src/lib/summary.ts':'Carry inspection from selected snapshot to summary/risk rows',
 'site/src/lib/protocol.ts':'Carry inspection to protocol view',
 'site/src/lib/compare.ts':'Carry inspection to comparable protocol records',
 'site/src/lib/viewState.ts':'Canonical protection filter/sort URL state',
 'site/src/lib/labels.ts':'Reuse exact Stage definition',
 'site/src/components/research/StageInspector.tsx':'Accessible source-backed Stage dialog',
 'site/src/components/research/ProtectionsBrowser.tsx':'Fact filters with exact observation evidence dialog',
 'site/src/components/research/ReviewEvidence.tsx':'Mount repeated source references on demand',
 'site/src/components/ProtocolBits.tsx':'Optional Stage inspection preserving original fallback',
 'site/src/components/ProtocolTable.tsx':'Connect protocol-row Stage inspection',
 'site/src/components/page/RiskView.tsx':'Connect Stage and protections research controls',
 'site/src/components/compare/CompareView.tsx':'Same-domain Stage inspection',
 'site/src/components/protocol/ProtocolHeader.tsx':'Profile Stage inspection',
 'site/src/components/protocol/Sections.tsx':'Criterion status copy and evidence expansion',
 'site/src/components/DataState.tsx':'Exact four-state explanation copy',
 'site/src/components/Tooltip.tsx':'Optional accessible dialog activation',
 'site/src/components/summary-page.test.tsx':'Required-copy assertion update',
 'site/src/components/__snapshots__/freshness-and-states.test.tsx.snap':'Owner-authorized new-version UI text reference',
 'site/src/app/page.tsx':'Neutral distribution context',
 'site/src/app/risk/page.tsx':'Construct scoped protection observations',
 'site/src/app/methodology/page.tsx':'Stage 0 definition and exact state explanations',
 'site/e2e/score-review.spec.ts':'Local keyboard, axe, reflow, filters and evidence checks',
 'site/e2e/score-review-hosted.spec.ts':'Opt-in read-only public HTTP/browser evidence',
};
for(const name of ['baseline','research','report','dossiers','verify','handoff'])paths[`scripts/score-review-${name}.mjs`]='Isolated review evidence/report workflow; no publication';
const walk=p=>fs.existsSync(p)?fs.readdirSync(p,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(`${p}/${e.name}`):[`${p}/${e.name}`]):[];
for(const p of walk('docs/research/score-review'))paths[p]='Required review finding/dossier and bounded evidence interpretation';
for(const p of walk(root).filter(p=>!p.endsWith('/changed-files.json')))paths[p]='Isolated capture, diagnostic, screenshot, validation log or source-binding evidence';
const digest=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const files=Object.entries(paths).map(([path,reason])=>({path,reason,sha256:digest(path)})).sort((a,b)=>a.path.localeCompare(b.path));
const generated=['site/out','site/public/data','site/public/feeds'].flatMap(walk).map(path=>({path,reason:'Regenerated disposable static build/copy artifact from the unchanged v0.3 candidate',sha256:digest(path)}));
write('changed-files.json',{recorded_at_utc:now,initial_worktree_dirty:true,note:'Explicit review file inventory, not a Git diff against the largely untracked starting tree. Pre-existing branding edits are preserved and not claimed as new review edits. This manifest omits its own hash.',files,generated_build_artifacts:generated,disposable_intermediates:['site/.next/','site/tsconfig.tsbuildinfo','site/test-results/','site/.lighthouseci/','target/review/'],intermediate_note:'Compiler/browser intermediates are not review source diffs. Selected final Lighthouse results and screenshots are copied into the isolated review output.'});
console.log(JSON.stringify({gates:gates.map(g=>[g.gate_id,g.status]),review_files:files.length,generated_build_artifacts:generated.length}));
