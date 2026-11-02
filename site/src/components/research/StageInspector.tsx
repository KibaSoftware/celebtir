"use client";
import { useState } from 'react';
import { DetailPanel, PanelSection } from '@/components/ui/DetailPanel';
import { EvidenceList } from '@/components/Evidence';
import { ScopedLink } from '@/components/scope/ScopedLink';
import { AUDIT_LABELS, STAGE_DEFINITION, STAGE_ZERO_EXPLANATION, type AuditStatus, type StageInspection } from '@/lib/scoreReview';
import { STAGE_LABELS } from '@/lib/labels';
import { Tooltip } from '@/components/Tooltip';
import styles from '@/components/ProtocolBits.module.css';

export function StageInspector({inspection}:{inspection:StageInspection}){
 const [open,setOpen]=useState(false);
 const cs=inspection.criteria.filter(c=>c.requirement.stage===inspection.stage.next);
 const verified=cs.filter(c=>c.status==='UNMET_VERIFIED').length,unresolved=cs.filter(c=>c.status==='UNRESOLVED_EVIDENCE').length;
 const labels:Record<AuditStatus,string>={MET:'Satisfied requirements',UNMET_VERIFIED:'Verified requirements not met',UNRESOLVED_EVIDENCE:'Unresolved requirements',NOT_APPLICABLE:'Not applicable'};
 return <>
  <Tooltip label={`${inspection.name}: ${STAGE_LABELS[inspection.stage.stage]}, ${verified} verified unmet, ${unresolved} unresolved criteria`} expanded={open} onActivate={()=>setOpen(true)} trigger={<span className={styles.stage} data-stage={inspection.stage.stage}>{STAGE_LABELS[inspection.stage.stage]}</span>}><span>{STAGE_DEFINITION} {verified} verified unmet; {unresolved} unresolved. Activate to inspect requirements and evidence.</span></Tooltip>
  {open?<DetailPanel title={`${STAGE_LABELS[inspection.stage.stage]} requirements`} eyebrow={`${inspection.name} · methodology v${inspection.methodology}`} onClose={()=>setOpen(false)}>
    <p>{STAGE_DEFINITION}</p>
    {inspection.stage.stage==='stage_0'?<p>{STAGE_ZERO_EXPLANATION}</p>:null}
    <p>Next-level criteria: {cs.filter(c=>c.status==='MET').length} satisfied · {verified} verified unmet · {unresolved} unresolved · {cs.filter(c=>c.status==='NOT_APPLICABLE').length} not applicable.</p>
    {(['MET','UNMET_VERIFIED','UNRESOLVED_EVIDENCE','NOT_APPLICABLE'] as const).map(status=>{
      const rows=cs.filter(c=>c.status===status);return rows.length?<PanelSection key={status} title={labels[status]}>{rows.map(c=><div key={c.requirement.id}><h3>{c.requirement.description}</h3><p>{AUDIT_LABELS[c.status]}</p><p>{c.requirement.detail}</p>{c.status==='UNRESOLVED_EVIDENCE'?<p>Evidence gap: {c.causeCodes.join(', ')||'The published observations do not establish this conclusion independently.'} No additional capture has been made for this view.</p>:null}<EvidenceList evidence={c.evidence}/></div>)}</PanelSection>:null;
    })}
    <PanelSection title="What would be required for the next Stage">{inspection.stage.next?<><p>Published evaluator: {STAGE_LABELS[inspection.stage.next]}</p><ul>{inspection.stage.missing.map(r=><li key={r.id}>{r.description}</li>)}</ul></>:<p>No next Stage is defined.</p>}</PanelSection>
    <PanelSection title="Snapshot"><p>{inspection.date} · methodology v{inspection.methodology}</p><ScopedLink href={`/protocols/${inspection.protocolId}/#stage`}>Full protocol criteria and program addresses</ScopedLink><p><ScopedLink href="/methodology/#stages">Published methodology</ScopedLink></p></PanelSection>
  </DetailPanel>:null}
 </>;
}
