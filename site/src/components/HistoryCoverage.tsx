import { loadHistory } from "@/lib/data";
import { activitySeries } from "@/lib/pages";
import { scopeDays } from "@/lib/scope";
import type { Snapshot, SummaryDay } from "@/lib/types";
import type { ChainFilter } from "@/lib/viewState";

/**
 * F9: say which series each count describes. DefiLlama's daily fee and
 * volume series are separate from Celebtir's own snapshots, which are the
 * points of the Stage and value charts.
 */
export function HistoryCoverage({ snapshot, chain }: { snapshot: Snapshot; chain: ChainFilter }) {
  const fees = activitySeries(snapshot, "fees");
  const volume = activitySeries(snapshot, "volume");
  const days = scopeDays(loadHistory<{ days: SummaryDay[] }>("summary")?.days ?? [], chain).filter(d => d.protocols > 0 || (d.unassessed ?? 0) > 0);
  const span = (s: { date: string }[]) => (s.length ? `${s[0]!.date} to ${s.at(-1)!.date}` : "none");
  return (
    <p>
      DefiLlama daily fees: {fees.length} day{fees.length === 1 ? "" : "s"} ({span(fees)}) · DefiLlama daily volume: {volume.length} day{volume.length === 1 ? "" : "s"} ({span(volume)}) · Celebtir snapshots: {days.length}
      {days.length ? ` (${days.map(d => d.date).join(", ")})` : ""}. Charts of ratings and value have one point per snapshot; missing days remain gaps.
    </p>
  );
}
