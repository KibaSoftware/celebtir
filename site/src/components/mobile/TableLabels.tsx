"use client";

import { useEffect } from "react";

const SKIP = 'table[data-cards="off"]';

/** A header cell's name: its `data-label`, or its text without sort arrows. */
function headerName(th: Element): string {
  return th.getAttribute("data-label") ?? (th.textContent ?? "").replace(/[↕▲▼×]/g, "").trim();
}

function label(root: ParentNode) {
  for (const table of root.querySelectorAll("main table")) {
    if (table.matches(SKIP)) continue;
    const names = [...table.querySelectorAll(":scope > thead > tr:last-child > *")].map(headerName);
    if (!names.length) continue;
    for (const row of table.querySelectorAll(":scope > tbody > tr")) {
      [...row.children].forEach((cell, i) => {
        const name = names[i];
        if (name && cell.getAttribute("colspan") === null && cell.getAttribute("data-label") !== name) {
          cell.setAttribute("data-label", name);
        }
      });
    }
  }
}

/**
 * UI-25: below 640 px, tables become cards (globals.css). Each cell shows its
 * column's name from `data-label`, copied here from the header so no table
 * has to repeat it; client-rendered rows are labelled as they appear.
 * Labelling waits for an idle moment, so it never touches a table React is
 * still hydrating (an attribute added before hydration is a mismatch).
 */
export function TableLabels() {
  useEffect(() => {
    const idle = (fn: () => void) =>
      typeof window.requestIdleCallback === "function" ? window.requestIdleCallback(fn, { timeout: 500 }) : window.setTimeout(fn, 50);
    let queued = false;
    const run = () => {
      if (queued) return;
      queued = true;
      idle(() => {
        queued = false;
        label(document);
      });
    };
    run();
    const observer = new MutationObserver(run);
    observer.observe(document.body, { childList: true, subtree: true, characterData: true });
    return () => observer.disconnect();
  }, []);
  return null;
}
