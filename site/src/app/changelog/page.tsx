import type { Metadata } from "next";
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import { Suspense } from "react";
import { ChangelogDefault, ChangelogView } from "@/components/reference/ChangelogView";
import { PageHeader } from "@/components/ui/Ui";
import { dataDir, loadHistory, loadSnapshot } from "@/lib/data";
import type { ChangeEvent, SummaryDay } from "@/lib/types";
import { ChainScoped } from "@/components/scope/ChainScoped";
import { scope, scopeEvents, scopeDays, SCOPES } from "@/lib/scope";
import type { ChainFilter } from "@/lib/viewState";

export const metadata: Metadata = {
  title: "Changes",
  description: "Every on-chain control change, registry edit and methodology change, newest first, with an Atom feed.",
};

function ChangelogPageBody({ chain }: { chain: ChainFilter }) {
  const s = scope(loadSnapshot(), chain);
  const events = scopeEvents(loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [], loadSnapshot(), chain);
  const names = Object.fromEntries(Object.entries(s.facts.protocols).map(([id, p]) => [id, p.name ?? id]));
  for (const e of events) if (e.protocol && !names[e.protocol]) names[e.protocol] = e.protocol;
  const feedDir = path.join(dataDir().dir, "feeds");
  const feeds = existsSync(feedDir)
    ? readdirSync(feedDir)
        .filter((f) => f.endsWith(".atom") && f !== "all.atom")
        .map((f) => f.slice(0, -5))
    : [];
  const props = { events, names, feeds };
  const days = scopeDays(loadHistory<{days: SummaryDay[]}>("summary")?.days ?? [], chain).filter(d => d.protocols > 0);
  return (
    <>
      <PageHeader art="changes"
        eyebrow="Change history"
        title="Changes"
        subtitle="Every change Celebtir detected: on-chain control changes, registry edits and methodology changes, each marked as such so a fix is never mistaken for news."
      />
      <section aria-label="Observation coverage"><h2>{days.length < 2 ? "Insufficient history" : "Recorded observation coverage"}</h2><p>{days.length} Celebtir snapshots{days.length ? ` · ${days[0]!.date} to ${days[days.length-1]!.date}` : ""}. Collection gaps do not establish absence of changes. Before/after transitions require comparable observations or transaction evidence.</p></section>
      <Suspense fallback={<ChangelogDefault {...props} />}>
        <ChangelogView {...props} />
      </Suspense>
    </>
  );
}

// HUI-1: rendered once per chain; the picker chooses the variant.
export default function ChangelogPage() {
  const variants = Object.fromEntries(SCOPES.map((c) => [c, <ChangelogPageBody key={c} chain={c} />])) as Record<ChainFilter, React.ReactNode>;
  return <ChainScoped variants={variants} />;
}
