// UI-4: the compare tray. Up to four protocols, kept in localStorage so the
// selection follows the reader across pages until cleared.

import { useCallback, useMemo, useSyncExternalStore } from "react";
import { COMPARE_MAX } from "./viewState";

export const TRAY_MAX = COMPARE_MAX;
const KEY = "celebtir.compare";
const EVENT = "celebtir-compare";
const SAFE = /^[a-z0-9_-]{1,64}$/;

export function addTo(list: string[], id: string): string[] {
  return list.includes(id) || list.length >= TRAY_MAX ? list : [...list, id];
}

export function removeFrom(list: string[], id: string): string[] {
  return list.filter((x) => x !== id);
}

function parse(raw: string): string[] {
  try {
    const v: unknown = JSON.parse(raw);
    return Array.isArray(v) ? v.filter((x): x is string => typeof x === "string" && SAFE.test(x)).slice(0, TRAY_MAX) : [];
  } catch {
    return [];
  }
}

function read(): string {
  try {
    return window.localStorage.getItem(KEY) ?? "[]";
  } catch {
    return "[]";
  }
}

function write(list: string[]) {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(list));
  } catch {
    // Storage blocked: the tray still works for this page.
  }
  window.dispatchEvent(new Event(EVENT));
}

function subscribe(onChange: () => void) {
  window.addEventListener("storage", onChange);
  window.addEventListener(EVENT, onChange);
  return () => {
    window.removeEventListener("storage", onChange);
    window.removeEventListener(EVENT, onChange);
  };
}

/** The tray's protocols and the actions on it. Empty at build time. */
export function useCompareTray() {
  const raw = useSyncExternalStore(subscribe, read, () => "[]");
  const list = useMemo(() => parse(raw), [raw]);
  const toggle = useCallback((id: string) => write(list.includes(id) ? removeFrom(list, id) : addTo(list, id)), [list]);
  const remove = useCallback((id: string) => write(removeFrom(list, id)), [list]);
  const clear = useCallback(() => write([]), []);
  const set = useCallback((ids: string[]) => write(ids.slice(0, TRAY_MAX)), []);
  return { list, full: list.length >= TRAY_MAX, toggle, remove, clear, set };
}
