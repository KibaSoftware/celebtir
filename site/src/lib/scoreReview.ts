// Review-only annotations over published facts. Never writes or replaces a score.
import type { Controller, Evidence, Factor, ProtocolFacts, ProtocolScore, Requirement } from './types';

export const STAGE_DEFINITION = 'Stage describes the maturity of verified governance and control arrangements. It is not a smart-contract audit, a measure of code safety, or investment advice.';
export const STAGE_ZERO_EXPLANATION = 'Stage 0: the protocol does not satisfy every requirement for Stage 1 under the selected Celebtir methodology. Some requirements may be unresolved because evidence is incomplete. Inspect the blockers below.';
export const PROTECTION_NOTE = 'These are observations about specific control mechanisms, not an overall safety ranking.';
export type AuditStatus = 'MET' | 'UNMET_VERIFIED' | 'UNRESOLVED_EVIDENCE' | 'NOT_APPLICABLE';
export const AUDIT_LABELS: Record<AuditStatus,string> = {MET:'Satisfied requirement',UNMET_VERIFIED:'Verified requirement not met',UNRESOLVED_EVIDENCE:'Requirement unresolved — evidence incomplete',NOT_APPLICABLE:'Not applicable'};
export type CauseCode = 'SINGLE_EFFECTIVE_KEY' | 'VOTER_THRESHOLD_BELOW_REQUIRED' | 'DELAY_BELOW_REQUIRED' | 'TIMELOCK_BYPASS' | 'CONFIG_AUTHORITY_BYPASS' | 'PRIVILEGED_ADMIN_POWER' | 'ORACLE_CONTROL_GAP' | 'CODE_TRANSPARENCY_GAP' | 'EXTERNAL_DEPENDENCY_GAP' | 'SOURCE_READ_FAILURE' | 'CONFLICTING_EVIDENCE' | 'UNSUPPORTED_DECODER' | 'UNMAPPED_CRITERION' | 'OTHER_SOURCE_BACKED';
export interface Cause { code: CauseCode; account: string; evidence: Evidence[]; observation: string }
export interface ReviewCriterion { requirement: Requirement; status: AuditStatus; uncertain: boolean; evidence: Evidence[]; causes: Cause[]; causeCodes: CauseCode[] }
export function controllerNodes(c: Controller): Controller[] {
  const children = c.kind==='program_controlled'?[c.upgrade]:c.kind==='multisig'?[...(c.config_authority?[c.config_authority]:[]),...Object.values(c.nested_signers??{})]:c.kind==='timelock'?[c.proposer,...(c.admin?[c.admin]:[])]:[];
  return [c,...children.flatMap(controllerNodes)];
}
/** Execution/configuration paths, excluding individual members of a quorum. */
function authorityNodes(c: Controller): Controller[] {
  const children=c.kind==='program_controlled'?[c.upgrade]:c.kind==='multisig'?(c.config_authority?[c.config_authority]:[]):c.kind==='timelock'?[c.proposer,...(c.admin?[c.admin]:[])]:[];
  return [c,...children.flatMap(authorityNodes)];
}
export function unresolvedController(c: Controller): boolean {
  return controllerNodes(c).some(n=>['unavailable','pda','unrecognized','cannot_act'].includes(n.kind) || n.kind==='multisig'&&(n.voters===0||n.threshold>n.voters));
}
/** Effective delay only for the fully decoded authority path; immutable is not infinite. */
export function observedDelay(c: Controller): number | null {
  if(unresolvedController(c))return null;
  switch(c.kind){
    case 'single_key': return 0;
    case 'program_controlled': return observedDelay(c.upgrade);
    case 'multisig': { const alt=c.config_authority?observedDelay(c.config_authority):c.time_lock_secs;return alt===null?null:Math.min(c.time_lock_secs,alt); }
    case 'timelock': { const alt=c.admin?observedDelay(c.admin):c.min_delay_secs;return alt===null?null:Math.min(c.min_delay_secs,alt); }
    default:return null;
  }
}
const factorFor: Record<string,Factor> = {'s1.upgrade_control':'upgrade_control','s1.admin_powers':'admin_powers','s1.verified_builds':'code_transparency','s0.public_source':'code_transparency','s0.upgrade_control_known':'upgrade_control','s1.upgrade_delay':'upgrade_delay'};
export function reviewCriteria(facts: ProtocolFacts, score: ProtocolScore): ReviewCriterion[] {
  const core=Object.entries(facts.programs).filter(([,p])=>p.role==='core');
  return score.stage.requirements.map(requirement=>{
    const id=requirement.id, factor=factorFor[id];
    const evidence: Evidence[] = factor?[...score.factors[factor].evidence]:[];
    const causes: Cause[]=[];
    let unresolved=false;
    const add=(code:CauseCode,account:string,ev:Evidence[],observation:string)=>{if(ev.length)causes.push({code,account,evidence:ev,observation});};
    const controls=id==='s1.admin_powers'||id==='s2.no_fund_control'?Object.entries(facts.admin).map(([a,p])=>[a,p.controller] as const):id==='s1.config_authority'?[...core.map(([a,p])=>[a,p.upgrade_control] as const),...Object.entries(facts.admin).map(([a,p])=>[a,p.controller] as const)]:core.map(([a,p])=>[a,p.upgrade_control] as const);
    if(['s0.upgrade_control_known','s1.upgrade_control','s1.upgrade_delay','s1.config_authority','s1.admin_powers','s2.immutable_or_governance','s2.no_fund_control'].includes(id)) for(const [account,f] of controls){
      evidence.push(...f.evidence);
      if(!f.evidence.length||unresolvedController(f.value))unresolved=true;
      for(const c of controllerNodes(f.value)){
        if(c.kind==='unavailable')add('SOURCE_READ_FAILURE',account,f.evidence,c.reason);
        if(c.kind==='pda'||c.kind==='unrecognized')add('UNSUPPORTED_DECODER',account,f.evidence,JSON.stringify(c));
        const accountFails = id==='s1.upgrade_control' ? score.programs[account]?.upgrade_control.rating==='high' : id==='s1.admin_powers' ? score.admin[account]?.rating==='high' : id==='s1.upgrade_delay' ? requirement.detail.includes(account) : id==='s1.config_authority';
        if(!requirement.met && accountFails){
          if((id==='s1.upgrade_control'||id==='s1.admin_powers')&&c.kind==='single_key'&&authorityNodes(f.value).includes(c))add('SINGLE_EFFECTIVE_KEY',account,f.evidence,JSON.stringify(c));
          // The published per-account assessment establishes threshold failure; no new threshold table.
          const rating=id==='s1.admin_powers'?score.admin[account]?.rating:score.programs[account]?.upgrade_control.rating;
          if((id==='s1.upgrade_control'||id==='s1.admin_powers')&&rating==='high'&&c.kind==='multisig'&&!c.config_authority)add('VOTER_THRESHOLD_BELOW_REQUIRED',account,f.evidence,JSON.stringify(c));
          if(c.kind==='multisig'&&c.config_authority&&observedDelay(c.config_authority)!==null){
            if(controllerNodes(c.config_authority).some(x=>x.kind==='single_key'))add('CONFIG_AUTHORITY_BYPASS',account,f.evidence,JSON.stringify(c));
            if((observedDelay(c)??Infinity)<c.time_lock_secs)add('TIMELOCK_BYPASS',account,f.evidence,JSON.stringify(c));
          }
          if(c.kind==='timelock'&&c.admin&&(observedDelay(c)??Infinity)<c.min_delay_secs)add('TIMELOCK_BYPASS',account,f.evidence,JSON.stringify(c));
        }
      }
      if(!requirement.met&&id==='s1.upgrade_delay'&&requirement.detail.includes(account)&&observedDelay(f.value)!==null)add('DELAY_BELOW_REQUIRED',account,f.evidence,`Effective delay ${observedDelay(f.value)} seconds; ${requirement.detail}`);
      if(!requirement.met&&id==='s1.admin_powers'&&score.admin[account]?.rating==='high')add('PRIVILEGED_ADMIN_POWER',account,f.evidence,`${facts.admin[account].power}: ${facts.admin[account].field}`);
    }
    if(id==='s0.core_programs')evidence.push(...core.flatMap(([,p])=>p.upgrade_control.evidence));
    if(id==='s0.public_source'||id==='s1.verified_builds')for(const [a,p] of core){
      evidence.push(...p.code_transparency.evidence);
      const c=p.code_transparency.value;
      if(c.kind==='unavailable'){unresolved=true;add('SOURCE_READ_FAILURE',a,p.code_transparency.evidence,c.reason);}
      if(!requirement.met&&c.kind!=='verified'){
        add('CODE_TRANSPARENCY_GAP',a,p.code_transparency.evidence,JSON.stringify(c));
        // Unverified source demonstrates missing verification, not a failed binary match.
        if(c.kind==='public_source_unverified')unresolved=true;
      }
    }
    if(id==='s1.no_unknown')evidence.push(...Object.values(score.factors).flatMap(s=>s.evidence));
    if(id==='s1.no_unknown')for(const [f,s] of Object.entries(score.factors))if(s.rating==='unknown'){
      unresolved=true;evidence.push(...s.evidence);
      const code:CauseCode|undefined=f==='oracle_dependency'?'ORACLE_CONTROL_GAP':f==='dependencies'?'EXTERNAL_DEPENDENCY_GAP':f==='code_transparency'?'CODE_TRANSPARENCY_GAP':undefined;
      for(const e of s.evidence){if(code)add(code,e.address,[e],s.reason);}
      // Controller failures are attributed to their concrete source reads as well.
      for(const [a,cf] of [...core.map(([a,p])=>[a,p.upgrade_control] as const),...Object.entries(facts.admin).map(([a,p])=>[a,p.controller] as const)])for(const c of controllerNodes(cf.value)){
        if(c.kind==='unavailable')add('SOURCE_READ_FAILURE',a,cf.evidence,c.reason);
        if(c.kind==='pda'||c.kind==='unrecognized')add('UNSUPPORTED_DECODER',a,cf.evidence,JSON.stringify(c));
      }
    }
    // Empty inventories do not independently prove that bypass/admin paths are absent.
    if((id==='s1.config_authority'||id==='s2.no_fund_control')&&!Object.keys(facts.admin).length&&!facts.admin_review)unresolved=true;
    const uniqueEvidence=[...new Map(evidence.map(e=>[JSON.stringify(e),e])).values()];
    const status:AuditStatus=unresolved||!uniqueEvidence.length?'UNRESOLVED_EVIDENCE':requirement.met?'MET':'UNMET_VERIFIED';
    const uniqueCauses=status==='MET'?[]:[...new Map(causes.map(c=>[`${c.code}:${c.account}`,c])).values()].sort((a,b)=>a.code<b.code?-1:a.code>b.code?1:a.account<b.account?-1:1);
    return {requirement,status,uncertain:status!=='MET'&&!uniqueCauses.length,evidence:uniqueEvidence,causes:uniqueCauses,causeCodes:[...new Set(uniqueCauses.map(c=>c.code))].sort()};
  });
}

export interface StageInspection { methodology:string; date:string; protocolId:string; name:string; stage:ProtocolScore['stage']; criteria:Omit<ReviewCriterion,'causes'>[] }
export function stageInspection(id:string,facts:ProtocolFacts,score:ProtocolScore,methodology:string,date:string):StageInspection {
  return {protocolId:id,name:facts.name??id,methodology,date,stage:score.stage,criteria:reviewCriteria(facts,score).map(({requirement,status,uncertain,evidence,causeCodes})=>({requirement,status,uncertain,evidence,causeCodes}))};
}

export type CandidateStatus = 'REASSESS_EXISTING'|'REJECT_OUT_OF_SCOPE'|'DEFER_INSUFFICIENT_EVIDENCE'|'DRAFT_RESEARCH_ONLY'|'READY_FOR_OWNER_REVIEW';
export function candidateDecision(c:{active:boolean;outOfScope:boolean;identified:boolean;completeEvidence:boolean;independentHumanReview:boolean;actionable:boolean}):{review_recommendation:CandidateStatus;publication_authorized:false}{
  return {review_recommendation:c.active?'REASSESS_EXISTING':c.outOfScope?'REJECT_OUT_OF_SCOPE':!c.identified?'DEFER_INSUFFICIENT_EVIDENCE':!c.completeEvidence||!c.independentHumanReview?c.actionable?'DRAFT_RESEARCH_ONLY':'DEFER_INSUFFICIENT_EVIDENCE':'READY_FOR_OWNER_REVIEW',publication_authorized:false};
}
