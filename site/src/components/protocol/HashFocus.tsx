"use client";

import { useEffect } from "react";

export function focusHashTarget(prefix = "") {
  let id: string;
  try { id = decodeURIComponent(window.location.hash.slice(1)); } catch { return; }
  if (!id || !id.startsWith(prefix)) return;
  const target = document.getElementById(id);
  if (!target) return;
  for (let parent = target.parentElement; parent; parent = parent.parentElement) {
    if (parent instanceof HTMLDetailsElement) parent.open = true;
  }
  requestAnimationFrame(() => {
    if (!target.hasAttribute("tabindex")) target.setAttribute("tabindex", "-1");
    target.scrollIntoView?.({ block: "start" });
    target.focus({ preventScroll: true });
  });
}

/**
 * UI-13: following a link to `#node-…` moves keyboard focus to that node in
 * the control graph, not just the scroll position. Targets carry
 * `tabIndex={-1}` so they can take focus.
 */
export function HashFocus({ prefix }: { prefix: string }) {
  useEffect(() => {
    const focus = () => focusHashTarget(prefix);
    focus();
    window.addEventListener("hashchange", focus);
    window.addEventListener("popstate", focus);
    return () => { window.removeEventListener("hashchange", focus); window.removeEventListener("popstate", focus); };
  }, [prefix]);
  return null;
}
