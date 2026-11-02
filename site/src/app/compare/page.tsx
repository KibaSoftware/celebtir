import type { Metadata } from "next";
import { Suspense } from "react";
import { CompareView, CompareViewDefault } from "@/components/compare/CompareView";
import { PageHeader } from "@/components/ui/Ui";
import { compareItems } from "@/lib/compare";
import { loadHistory, loadSnapshot } from "@/lib/data";
import type { ProtocolHistoryFile } from "@/lib/types";

export const metadata: Metadata = {
  title: "Compare",
  description: "Two to four assessed DeFi protocols side by side: Stage, six factors, controllers, upgrades, value and tokens.",
};

// UI-16 to UI-18. The selection lives in the URL (`?p=a,b`), so a comparison
// can be shared as a link.
export default function ComparePage() {
  const s = loadSnapshot();
  const ids = Object.keys(s.scores.protocols);
  const items = compareItems(s, Object.fromEntries(ids.map((id) => [id, loadHistory<ProtocolHistoryFile>(id)])));
  return (
    <>
      <PageHeader art="compare"
        eyebrow="Research workspace"
        title="Compare"
        subtitle="Two to four protocols side by side: Stage, the six factors with their reasons, controllers, upgrades, value and tokens. Factors are compared one by one, never summed."
      />
      <Suspense fallback={<CompareViewDefault items={items} />}>
        <CompareView items={items} />
      </Suspense>
    </>
  );
}
