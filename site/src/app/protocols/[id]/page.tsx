import type { Metadata } from "next";
import { OG_SIZE } from "@/components/OgCard";
import { ProtocolHeader } from "@/components/protocol/ProtocolHeader";
import { ProtocolSections } from "@/components/protocol/Sections";
import { freshness, loadHistory, loadSnapshot } from "@/lib/data";
import { logoFor } from "@/lib/logos";
import { STAGE_LABELS } from "@/lib/labels";
import { SITE_URL } from "@/lib/site";
import { buildProtocolPage, protocolIds } from "@/lib/protocol";
import type { ChangeEvent, ProtocolHistoryFile } from "@/lib/types";

// One static page per rated protocol (UI-12). Unknown ids are 404s.
// Typed by hand: generated route types only exist after `next build`.
type Props = { params: Promise<{ id: string }> };
export const dynamicParams = false;

export function generateStaticParams() {
  return protocolIds(loadSnapshot()).map((id) => ({ id }));
}

function page(id: string) {
  const events = loadHistory<{ events: ChangeEvent[] }>("events")?.events ?? [];
  return buildProtocolPage(loadSnapshot(), id, loadHistory<ProtocolHistoryFile>(id), events);
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const p = page((await params).id);
  // F5: an unassessed protocol's metadata names no Stage.
  const status = p.stage ? STAGE_LABELS[p.stage] : "unassessed (facts listed, not rated)";
  return {
    title: p.name,
    description: p.stage
      ? `${p.name}: ${status}. Who can change it, how fast, and the on-chain evidence for every rating.`
      : `${p.name} on ${p.chain === "hyper_evm" ? "HyperEVM" : p.chain}: ${status}. Who can change its contracts, with the on-chain evidence for every fact.`,
    // UI-14: the Markdown twin.
    alternates: { types: { "text/markdown": `/protocols/${p.id}.md` } },
    // UI-15: the share card.
    openGraph: {
      title: `${p.name} · Celebtir`,
      type: "website",
      images: [{ url: `/og/${p.id}.png`, width: OG_SIZE.width, height: OG_SIZE.height, alt: p.stage ? `${p.name}: ${status} and six factor ratings` : `${p.name}: ${status}` }],
    },
    twitter: { card: "summary_large_image", images: [`/og/${p.id}.png`] },
  };
}

export default async function ProtocolPage({ params }: Props) {
  const p = page((await params).id);
  const s = loadSnapshot();
  const snapshot = { date: freshness(s).date, slot: s.facts.protocols[p.id]?.max_slot ?? null, minSlot: s.facts.protocols[p.id]?.min_slot ?? null, methodology: s.scores.methodology_version };
  return (
    <>
      <ProtocolHeader page={p} logo={logoFor(p.id)} controller={s.scores.protocols[p.id]?.controller_summary ?? p.programs.filter(x => x.role === "core").map(x => x.controllerText).sort()[0] ?? "Unavailable"} snapshot={snapshot} />
      <ProtocolSections page={p} siteUrl={SITE_URL} />
    </>
  );
}
