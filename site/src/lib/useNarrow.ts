"use client";

import { useSyncExternalStore } from "react";

/** The phone breakpoint (UI-25); globals.css uses the same width. */
export const NARROW_QUERY = "(max-width: 640px)";

const media = () => (typeof window.matchMedia === "function" ? window.matchMedia(NARROW_QUERY) : null);

function subscribe(onChange: () => void) {
  const m = media();
  m?.addEventListener("change", onChange);
  return () => m?.removeEventListener("change", onChange);
}

/** True on a phone-width window; false at build time or without matchMedia. */
export function useNarrow(): boolean {
  return useSyncExternalStore(
    subscribe,
    () => media()?.matches ?? false,
    () => false,
  );
}
