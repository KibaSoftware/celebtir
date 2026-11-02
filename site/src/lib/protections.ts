import type { Controller, Evidence, Factor, ProtocolFacts, ProtocolScore } from './types';
import { controllerNodes, observedDelay, unresolvedController } from './scoreReview';
export const PROTECTION_KEYS=['timelock','multi_actor','transparency','bypass','unresolved_authority'] as const;
export type ProtectionKey=typeof PROTECTION_KEYS[number];
export type Truth='TRUE'|'FALSE'|'UNKNOWN';
export type ProtectionSort='name'|'delay_desc'|'delay_asc'|'unresolved';
export const PROTECTION_LABELS:Record<ProtectionKey,string>={timelock:'Has verified upgrade timelock',multi_actor:'Verified multi-actor approval',transparency:'Verified code/source transparency',bypass:'Administrative bypass identified',unresolved_authority:'Unresolved critical authority'};
export interface ProtectionObservation {value:Truth;evidence:Evidence[];factor:Factor}
export interface ProtectionRow {id:string;name:string;methodology:string;domain:string;stage:string;delay:number|null;unresolved:number;observations:Record<ProtectionKey,ProtectionObservation>}
function multi(c:Controller):Truth {
 if(unresolvedController(c))return 'UNKNOWN';
 if(c.kind==='program_controlled')return multi(c.upgrade);
 if(c.kind==='immutable')return 'UNKNOWN'; // Not applicable is not a positive multi-actor observation.
 if(c.kind==='single_key')return 'FALSE';
 if(c.kind==='timelock')return c.admin?and([multi(c.proposer),multi(c.admin)]):multi(c.proposer);
 if(c.kind==='multisig')return c.threshold>=2&&c.voters>=c.threshold?(c.config_authority?multi(c.config_authority):'TRUE'):'FALSE';
 return 'UNKNOWN';
}
export const and=(xs:Truth[]):Truth=>!xs.length?'UNKNOWN':xs.includes('FALSE')?'FALSE':xs.includes('UNKNOWN')?'UNKNOWN':'TRUE';
export function protectionRow(id:string,facts:ProtocolFacts,score:ProtocolScore,methodology:string):ProtectionRow {
 const core=Object.values(facts.programs).filter(p=>p.role==='core');
 const ev=core.flatMap(p=>p.upgrade_control.evidence);
 const delays=core.map(p=>p.upgrade_control.evidence.length?observedDelay(p.upgrade_control.value):null);
 const delay=delays.length&&delays.every(d=>d!==null)?Math.min(...delays as number[]):null;
 const critical=[...core.map(p=>p.upgrade_control),...Object.values(facts.admin).map(a=>a.controller)];
 const unknowns=critical.filter(f=>!f.evidence.length||unresolvedController(f.value));
 const bypass=critical.filter(f=>f.evidence.length&&controllerNodes(f.value).some(c=>c.kind==='multisig'&&c.config_authority&&observedDelay(c.config_authority)!==null||c.kind==='timelock'&&c.admin&&observedDelay(c.admin)!==null));
 return {id,name:facts.name??id,methodology,domain:facts.chain,stage:score.stage.stage,delay,unresolved:unknowns.length,observations:{
  timelock:{value:delay===null?'UNKNOWN':delay>0?'TRUE':'FALSE',evidence:ev,factor:'upgrade_delay'},
  multi_actor:{value:and(core.map(p=>p.upgrade_control.evidence.length?multi(p.upgrade_control.value):'UNKNOWN')),evidence:ev,factor:'upgrade_control'},
  transparency:{value:and(core.map(p=>!p.code_transparency.evidence.length||p.code_transparency.value.kind==='unavailable'?'UNKNOWN':p.code_transparency.value.kind==='verified'?'TRUE':p.code_transparency.value.kind==='no_public_source'?'FALSE':'UNKNOWN')),evidence:core.flatMap(p=>p.code_transparency.evidence),factor:'code_transparency'},
  // A missing bypass in a finite inventory is not proof of its absence.
  bypass:{value:bypass.length?'TRUE':'UNKNOWN',evidence:bypass.flatMap(f=>f.evidence),factor:'upgrade_control'},
  unresolved_authority:{value:unknowns.length?'TRUE':critical.length?'FALSE':'UNKNOWN',evidence:(unknowns.length?unknowns:critical).flatMap(f=>f.evidence),factor:'upgrade_control'},
 }};
}
export function selectProtections(rows:ProtectionRow[],filters:ProtectionKey[],includeUnresolved:boolean,sort:ProtectionSort,methodology:string){
 const matches:ProtectionRow[]=[],unresolved:ProtectionRow[]=[],unrated:ProtectionRow[]=[];
 for(const r of rows){
  if(r.domain!=='solana'||r.methodology!==methodology)continue;
  if(r.stage==='unrated'){unrated.push(r);continue;}
  const match=filters.length?and(filters.map(k=>r.observations[k].value)):'TRUE';
  if(match==='TRUE')matches.push(r);else if(match==='UNKNOWN'&&includeUnresolved)unresolved.push(r);
 }
 const collator=new Intl.Collator('en',{sensitivity:'base',numeric:true});
 const compare=(a:ProtectionRow,b:ProtectionRow)=>{
  let d=0;
  if(sort==='name')d=collator.compare(a.name,b.name);
  if(sort==='unresolved')d=a.unresolved-b.unresolved;
  if(sort==='delay_asc'||sort==='delay_desc')d=a.delay===null?(b.delay===null?0:1):b.delay===null?-1:(a.delay-b.delay)*(sort==='delay_asc'?1:-1);
  return d||(a.id<b.id?-1:a.id>b.id?1:0);
 };
 return {matches:matches.sort(compare),unresolved:unresolved.sort(compare),unrated:unrated.sort(compare)};
}
