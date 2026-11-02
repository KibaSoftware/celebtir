import { describe,it,expect } from 'vitest';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { reviewCriteria,candidateDecision,stageInspection } from './scoreReview';
import { protectionRow,selectProtections,and,type ProtectionRow } from './protections';
import { parseView,serializeView } from './viewState';
import type { Controller,Requirement,FactsFile,ScoresFile } from './types';
// This scenario requires Orca's incomplete-build evidence. Use the immutable
// replay fixture, independent of CELEBTIR_DATA or ignored local QA candidates.
const replay=path.resolve(process.cwd(),'../fixtures/replay/v0.2/2026-10-21');
const snapshot={facts:JSON.parse(readFileSync(path.join(replay,'facts.json'),'utf8')) as FactsFile,scores:JSON.parse(readFileSync(path.join(replay,'scores.json'),'utf8')) as ScoresFile};
const original=()=>({facts:structuredClone(snapshot.facts.protocols.orca),score:structuredClone(snapshot.scores.protocols.orca)});
const req=(id:string,met=false):Requirement=>({id,met,stage:'stage_1',description:id,detail:''});
describe('score review annotations',()=>{
 it('srd_02 preserves all published values and identifies incomplete build evidence',()=>{
  const {facts,score}=original(),before=JSON.stringify({facts,score});
  const cs=reviewCriteria(facts,score);
  expect(cs.find(c=>c.requirement.id==='s1.verified_builds')?.status).toBe('UNRESOLVED_EVIDENCE');
  expect(JSON.stringify({facts,score})).toBe(before);
  expect(stageInspection('orca',facts,score,'0.3','2026-10-26').stage).toEqual(score.stage);
 });
 it('srd_03 retains simultaneous short-delay and bypass causes, deduplicated and sorted',()=>{
  const {facts,score}=original();const [a,p]=Object.entries(facts.programs).find(([,p])=>p.role==='core')!;
  p.upgrade_control.value={kind:'multisig',multisig_kind:'squads_v4',address:a,threshold:3,voters:5,time_lock_secs:86400,config_authority:{kind:'single_key',key:a}};
  p.upgrade_control.evidence.push(...p.upgrade_control.evidence);
  score.stage.requirements=[{...req('s1.upgrade_delay'),detail:`${a}: effective time lock 0 s; needs 5 days+`}];
  const c=reviewCriteria(facts,score)[0];
  expect(c.status).toBe('UNMET_VERIFIED');
  expect(c.causeCodes).toEqual(['CONFIG_AUTHORITY_BYPASS','DELAY_BELOW_REQUIRED','TIMELOCK_BYPASS']);
  expect(c.causes).toHaveLength(3);
 });
 it('srd_03 source-free failed criterion is uncertain and never invents a cause',()=>{
  const {facts,score}=original();score.stage.requirements=[req('unmapped')];
  expect(reviewCriteria(facts,score)[0]).toMatchObject({status:'UNRESOLVED_EVIDENCE',uncertain:true,causeCodes:[]});
 });
 it('srd_02 unknown upgrade remains Unrated, other unknowns remain unresolved with source causes',()=>{
  const {facts,score}=original();const p=Object.values(facts.programs).find(p=>p.role==='core')!;
  p.upgrade_control.value={kind:'unavailable',reason:'captured RPC read failed'};
  score.stage.stage='unrated';score.stage.next='stage_0';score.stage.requirements=[{...req('s0.upgrade_control_known'),stage:'stage_0'}];
  expect(reviewCriteria(facts,score)[0]).toMatchObject({status:'UNRESOLVED_EVIDENCE',causeCodes:['SOURCE_READ_FAILURE']});
  expect(score.stage.stage).toBe('unrated');
  score.stage.stage='stage_0';score.stage.requirements=[req('s1.no_unknown')];score.factors.upgrade_control.rating='unknown';
  expect(reviewCriteria(facts,score)[0].status).toBe('UNRESOLVED_EVIDENCE');expect(score.stage.stage).toBe('stage_0');
 });
 it('srd_02 a factor N/A is not a fabricated Stage exemption',()=>{
  const {facts,score}=original();score.factors.oracle_dependency.rating='not_applicable';
  score.stage.requirements=[req('s1.no_unknown',true)];
  Object.values(score.factors).forEach(f=>{if(f.rating==='unknown')f.rating='medium';});
  expect(reviewCriteria(facts,score)[0]).toMatchObject({status:'MET',causeCodes:[]});
 });
 it('srd_06 candidate readiness requires complete authority evidence AND independent human review',()=>{
  const c={active:false,outOfScope:false,identified:true,completeEvidence:true,independentHumanReview:true,actionable:true};
  expect(candidateDecision({...c,completeEvidence:false}).review_recommendation).toBe('DRAFT_RESEARCH_ONLY');
  expect(candidateDecision({...c,independentHumanReview:false}).review_recommendation).toBe('DRAFT_RESEARCH_ONLY');
  expect(candidateDecision(c)).toEqual({review_recommendation:'READY_FOR_OWNER_REVIEW',publication_authorized:false});
  expect(candidateDecision({...c,active:true}).review_recommendation).toBe('REASSESS_EXISTING');
  expect(candidateDecision({...c,identified:false}).review_recommendation).toBe('DEFER_INSUFFICIENT_EVIDENCE');
 });
});
describe('verified protections',()=>{
 const make=(id:string,c:Controller):ProtectionRow=>{const {facts,score}=original();const p=Object.values(facts.programs).find(p=>p.role==='core')!;p.upgrade_control.value=c;return protectionRow(id,facts,score,'0.3');};
 const multi=(delay:number):Controller=>({kind:'multisig',multisig_kind:'squads_v4',address:'fixture',threshold:3,voters:5,time_lock_secs:delay,config_authority:null});
 it('srd_05 uses three-valued AND and separates unresolved from matches',()=>{
  expect(and(['TRUE','UNKNOWN'])).toBe('UNKNOWN');expect(and(['FALSE','UNKNOWN'])).toBe('FALSE');
  const yes=make('yes',multi(86400)),no=make('no',multi(0)),unknown=make('unknown',{kind:'unavailable',reason:'missing'});
  const result=selectProtections([yes,no,unknown],['timelock'],true,'name','0.3');
  expect(result.matches.map(r=>r.id)).toEqual(['yes']);expect(result.unresolved.map(r=>r.id)).toEqual(['unknown']);
  expect(selectProtections([no],['timelock'],false,'name','0.3').matches).toEqual([]);
 });
 it('srd_05 effective delays respect bypass, unknown-last, both directions and stable ID ties',()=>{
  const rows=[make('z',multi(100)),make('b',multi(10)),make('a',multi(10)),make('u',{kind:'unavailable',reason:'missing'})];
  expect(selectProtections(rows,[],false,'delay_asc','0.3').matches.map(r=>r.id)).toEqual(['a','b','z','u']);
  expect(selectProtections(rows,[],false,'delay_desc','0.3').matches.map(r=>r.id)).toEqual(['z','a','b','u']);
  const controller=multi(86400);if(controller.kind==='multisig')controller.config_authority={kind:'single_key',key:'fixture'};
  expect(make('bypass',controller)).toMatchObject({delay:0,observations:{timelock:{value:'FALSE'},bypass:{value:'TRUE'}}});
  expect(make('immutable',{kind:'immutable'})).toMatchObject({delay:null,observations:{multi_actor:{value:'UNKNOWN'}}});
 });
 it('srd_05 excludes incompatible domains and methodologies; retains Unrated separately',()=>{
  const row=make('a',multi(0));
  const result=selectProtections([{...row,domain:'hyper_evm'},{...row,methodology:'0.2'},{...row,stage:'unrated'}],[],false,'name','0.3');
  expect(result.matches).toEqual([]);expect(result.unrated).toHaveLength(1);
 });
 it('srd_05 URL roundtrip and clear preserve chain and unrelated state',()=>{
  const v=parseView(new URLSearchParams('chain=solana&protections=timelock,multi_actor&includeUnresolved=1&protectionSort=delay_desc&p=orca,jito'));
  expect(parseView(new URLSearchParams(serializeView(v)))).toEqual(v);
  const clear=parseView(new URLSearchParams(serializeView({...v,protections:[],includeUnresolved:false,protectionSort:'name'})));
  expect(clear.chain).toBe('solana');expect(clear.compare).toEqual(['orca','jito']);expect(clear.protections).toBeUndefined();
 });
});
