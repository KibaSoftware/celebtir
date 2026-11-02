"use client";
import { useState } from 'react';
import { EvidenceList } from '@/components/Evidence';
import type { Evidence } from '@/lib/types';

/** Repeated criterion references are mounted only when the reader requests them. */
export function ReviewEvidence({evidence}:{evidence:Evidence[]}) {
  const [open,setOpen]=useState(false);
  return <details onToggle={e=>setOpen(e.currentTarget.open)}><summary>Source observations ({evidence.length})</summary>{open?<EvidenceList evidence={evidence}/>:null}</details>;
}
