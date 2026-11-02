"use client";
import { useState } from 'react';
import { DetailPanel } from '@/components/ui/DetailPanel';
import { EvidenceList } from '@/components/Evidence';
import type { Evidence } from '@/lib/types';
import { PROTECTION_KEYS,PROTECTION_LABELS,selectProtections,type ProtectionRow,type ProtectionSort } from '@/lib/protections';
import { PROTECTION_NOTE } from '@/lib/scoreReview';
import type { ViewState } from '@/lib/viewState';
import { ScopedLink } from '@/components/scope/ScopedLink';
export function ProtectionsBrowser({rows,view,update}:{rows:ProtectionRow[];view:ViewState;update:(f:(v:ViewState)=>ViewState)=>void}){
 const [evidence,setEvidence]=useState<{title:string;items:Evidence[]}|null>(null);
 const filters=view.protections??[];
 const selected=selectProtections(view.chain==='hyperliquid'?[]:rows,filters,view.includeUnresolved??false,view.protectionSort??'name',rows[0]?.methodology??'');
 return <section aria-label="Compare verified protections">
  <h2>Compare verified protections</h2><p>{PROTECTION_NOTE}</p>
  <p>Solana observations only. Multiple approval keys do not establish that their owners are independent. Immutable programs have no measured execution delay.</p>
  <fieldset><legend>Verified observations (all selected criteria must match)</legend>{PROTECTION_KEYS.map(k=><label key={k} style={{display:'block'}}><input type="checkbox" checked={filters.includes(k)} onChange={()=>update(v=>({...v,protections:filters.includes(k)?filters.filter(x=>x!==k):[...filters,k]}))}/>{PROTECTION_LABELS[k]}</label>)}</fieldset>
  <label><input type="checkbox" checked={view.includeUnresolved??false} onChange={e=>update(v=>({...v,includeUnresolved:e.target.checked}))}/>Include unresolved</label>{' '}
  <label>Sort protections <select value={view.protectionSort??'name'} onChange={e=>update(v=>({...v,protectionSort:e.target.value as ProtectionSort}))}><option value="name">Protocol name (A–Z)</option><option value="delay_desc">Verified upgrade delay (longest first)</option><option value="delay_asc">Verified upgrade delay (shortest first)</option><option value="unresolved">Unresolved critical observations (fewest first)</option></select></label>{' '}
  <button type="button" onClick={()=>update(v=>({...v,protections:[],includeUnresolved:false,protectionSort:'name'}))}>Clear filters</button>
  {!selected.matches.length?<p>No assessed protocols match these verified criteria.</p>:null}
  {(['matches','unresolved','unrated'] as const).map(group=>selected[group].length?<div key={group}><h3>{group==='matches'?'Matching observations':group==='unresolved'?'Unresolved observations — not verified matches':'Unrated — insufficient evidence for Stage assignment'}</h3><ul>{selected[group].map(r=><li key={r.id}><ScopedLink href={`/protocols/${r.id}/`}>{r.name}</ScopedLink>{' · '}{r.delay===null?'Unknown effective delay':`${r.delay.toLocaleString('en')} seconds effective delay`}{' · '}{r.unresolved} unresolved critical observations<ul>{PROTECTION_KEYS.map(k=><li key={k}>{PROTECTION_LABELS[k]}: {r.observations[k].value==='TRUE'?'Established':r.observations[k].value==='FALSE'?'Not observed in the decoded path':'Unknown'}{r.observations[k].evidence.length?<>{' '}<button type="button" onClick={()=>setEvidence({title:`${r.name}: ${PROTECTION_LABELS[k]}`,items:r.observations[k].evidence})}>Inspect {PROTECTION_LABELS[k].toLowerCase()} evidence for {r.name}</button></>:null}</li>)}</ul></li>)}</ul></div>:null)}
 {evidence?<DetailPanel title={evidence.title} onClose={()=>setEvidence(null)}><h3>Evidence: accounts read</h3><EvidenceList evidence={evidence.items}/></DetailPanel>:null}
 </section>;
}
