// F6: one reading of the change log for every view. On-chain transactions
// carry their block date (`occurred_on`); `date` is the day Celebtir recorded
// them. "Last on-chain change", the homepage count, insights and the change
// log all order and window by the block date.

import type { ChangeEvent } from "./types";

const DAY = 86_400_000;

/** When the change happened: its block date if known, else when it was recorded. */
export function changeDate(e: ChangeEvent): string {
  return e.occurred_on ?? e.date;
}

/** On-chain events, newest change first. */
export function onChainChanges(events: ChangeEvent[], protocol?: string): ChangeEvent[] {
  return events
    .filter(e => e.source === "on_chain" && (!protocol || e.protocol === protocol))
    .sort((a, b) => changeDate(b).localeCompare(changeDate(a)) || a.id.localeCompare(b.id));
}

export function latestOnChain(events: ChangeEvent[], protocol: string): ChangeEvent | undefined {
  return onChainChanges(events, protocol)[0];
}

/** On-chain changes that happened within `days` before (and on) `snapshotDate`. */
export function recentOnChain(events: ChangeEvent[], snapshotDate: string, days = 30): ChangeEvent[] {
  const end = Date.parse(snapshotDate);
  return onChainChanges(events).filter(e => {
    const t = Date.parse(changeDate(e));
    return e.protocol && t <= end && end - t <= days * DAY;
  });
}

/** The change-log anchor of an event, as the Atom feeds link it. */
export function eventAnchor(id: string): string {
  return id.replace(/[^A-Za-z0-9_-]/g, "-");
}
