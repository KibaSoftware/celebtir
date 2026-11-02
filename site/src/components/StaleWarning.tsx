"use client";

import { useSyncExternalStore } from "react";

/** UI-2: data older than this shows a warning. */
export const STALE_AFTER_HOURS = 36;

export function hoursSince(finishedAt: string, now: number): number {
  return (now - Date.parse(finishedAt)) / 3_600_000;
}

const HOUR = 3_600_000;
const noSubscription = () => () => {};
// Hour granularity keeps the snapshot stable between renders.
const browserHour = () => Math.floor(Date.now() / HOUR);
// Nothing at build time: the age is only known in the browser.
const buildHour = () => null;

/**
 * The site is static, so the age is computed in the browser at view time.
 * `now` is injectable for tests.
 */
export function StaleWarning({ finishedAt, now }: { finishedAt: string; now?: number }) {
  const hour = useSyncExternalStore(noSubscription, browserHour, buildHour);
  const clock = now ?? (hour === null ? undefined : hour * HOUR);
  if (clock === undefined) return null;
  const hours = hoursSince(finishedAt, clock);
  if (hours <= STALE_AFTER_HOURS) return null;
  const days = Math.floor(hours / 24);
  return (
    <span role="status" data-stale="true">
      Data is {days} day{days === 1 ? "" : "s"} old; the daily update may have failed.
    </span>
  );
}
