import type { Metadata } from "next";
import { Suspense } from "react";
import { RiskView, RiskViewDefault, type FactorCriteria, type Details } from "@/components/page/RiskView";
import { PageHeader } from "@/components/ui/Ui";
import { freshness, loadHistory, loadSnapshot } from "@/lib/data";
import { FACTOR_LABELS } from "@/lib/labels";
import { logoMap } from "@/lib/logos";
import { factorRules, loadMethodology } from "@/lib/methodology";
import { factorRows } from "@/lib/pages";
import { buildRows } from "@/lib/summary";
import { protectionRow } from '@/lib/protections';
import { FACTORS, type ChangeEvent, type Factor, type ProtocolHistoryFile } from "@/lib/types";

export const metadata: Metadata = {
  title: "Risk Analysis",
  description: "Every protocol's rating on the six control factors, with the reason, the methodology criterion and the on-chain evidence.",
};

export default function RiskPage() {
  const s = loadSnapshot();
  const ids = Object.keys(s.scores.protocols);
  const rows = buildRows(
    s,
    Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])),
    loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [],
  );
  const details = Object.fromEntries(FACTORS.map((f) => [f, factorRows(s, f).map(row => ({
    ...row,
    implications: s.scores.protocols[row.id].risks.filter(r => r.factor === f).map(r => r.text),
    authorities: f === "upgrade_control" || f === "upgrade_delay"
      ? Object.entries(s.facts.protocols[row.id].programs).filter(([,p]) => p.role === "core").map(([address,p]) => ({name: address, controller: p.upgrade_control.value}))
      : f === "admin_powers" ? Object.entries(s.facts.protocols[row.id].admin).map(([key,a]) => ({name: `${a.power} · ${a.field} · ${key}`, controller: a.controller.value})) : [],
  }))])) as Details;
  // The criteria as the methodology page states them, thresholds read from the TOML (SCO-2).
  const rules = factorRules(loadMethodology(s.scores.methodology_version));
  const criteria = Object.fromEntries(
    FACTORS.map((f) => {
      const r = rules.find((x) => x.factor === FACTOR_LABELS[f])!;
      return [f, { question: r.question, low: r.low, medium: r.medium, high: r.high }];
    }),
  ) as Record<Factor, FactorCriteria>;
  const meta = { date: freshness(s).date, methodology: s.scores.methodology_version };
  const protections = ids.filter(id=>s.facts.protocols[id].chain==='solana').map(id=>protectionRow(id,s.facts.protocols[id],s.scores.protocols[id],s.scores.methodology_version));
  const props = { rows, details, criteria, meta, logos: logoMap(ids), protections };
  return (
    <>
      <PageHeader art="risk"
        eyebrow="Six independent factors"
        title="Risk Analysis"
        subtitle="Explore the control assumptions, administrative privileges, and dependencies of assessed protocols."
      />
      <Suspense fallback={<RiskViewDefault {...props} />}>
        <RiskView {...props} />
      </Suspense>
    </>
  );
}
