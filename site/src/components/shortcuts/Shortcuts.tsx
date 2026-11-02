"use client";

import { useEffect, useRef, useState } from "react";
import styles from "./Shortcuts.module.css";

/** UI-3: the shortcuts, also shown by `?`. */
export const SHORTCUTS = [
  ["/", "Search: this page's search box, or the site search"],
  ["F", "Filters: open this page's filters"],
  ["C", "Compare tray: jump to it"],
  ["?", "This list"],
] as const;

const typing = (t: EventTarget | null) =>
  t instanceof HTMLElement && (t.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(t.tagName));

/** First keyboard-reachable element inside `root`. */
function firstFocusable(root: Element): HTMLElement | null {
  return root.querySelector<HTMLElement>("a[href], button:not([disabled]), input:not([disabled]), select, summary, [tabindex]");
}

/**
 * Global keyboard shortcuts (UI-3). Pages mark their targets with
 * `data-shortcut="search" | "filters"`; the compare tray is `"tray"`. Keys are
 * ignored while typing or with Ctrl, Alt or Meta held.
 */
export function Shortcuts() {
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState("");
  const dialog = useRef<HTMLDivElement>(null);
  const returnTo = useRef<HTMLElement | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && open) {
        setOpen(false);
        returnTo.current?.focus();
        return;
      }
      if (e.ctrlKey || e.metaKey || e.altKey || typing(e.target) || e.defaultPrevented) return;
      const say = (m: string) => setMessage(m);
      // Shift+/ is "?" on US keyboards; some layouts and tools report "/".
      const key = e.key === "/" && e.shiftKey ? "?" : e.key;
      switch (key) {
        case "/": {
          const target =
            document.querySelector<HTMLElement>('main [data-shortcut="search"]') ??
            document.querySelector<HTMLElement>('[data-shortcut="site-search"]');
          if (!target) return;
          e.preventDefault();
          target.focus();
          return;
        }
        case "f":
        case "F": {
          e.preventDefault();
          const target = document.querySelector<HTMLElement>('[data-shortcut="filters"]');
          if (!target) return say("This page has no filters.");
          if (target instanceof HTMLDetailsElement) {
            target.open = true;
            target.querySelector<HTMLElement>("summary")?.focus();
          } else {
            firstFocusable(target)?.focus();
          }
          return;
        }
        case "c":
        case "C": {
          e.preventDefault();
          const tray = document.querySelector('[data-shortcut="tray"]');
          if (!tray) return say("The compare tray is empty: tick a protocol in any table to add it.");
          (tray.querySelector<HTMLElement>("a[href]") ?? firstFocusable(tray))?.focus();
          say("Moved to the compare tray.");
          return;
        }
        case "?": {
          e.preventDefault();
          returnTo.current = document.activeElement as HTMLElement | null;
          setOpen(true);
          return;
        }
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [open]);

  useEffect(() => {
    if (open) dialog.current?.focus();
  }, [open]);

  return (
    <>
      <div role="status" className="sr-only">
        {message}
      </div>
      {open ? (
        <div className={styles.backdrop} onClick={() => setOpen(false)}>
          <div
            ref={dialog}
            role="dialog"
            aria-modal="true"
            aria-labelledby="shortcuts-title"
            tabIndex={-1}
            className={styles.dialog}
            onClick={(e) => e.stopPropagation()}
          >
            <h2 id="shortcuts-title" className={styles.title}>
              Keyboard shortcuts
            </h2>
            <dl className={styles.list}>
              {SHORTCUTS.map(([k, what]) => (
                <div key={k}>
                  <dt>
                    <kbd>{k}</kbd>
                  </dt>
                  <dd>{what}</dd>
                </div>
              ))}
            </dl>
            <p className={styles.hint}>Shortcuts are off while you type in a field. Escape closes this list.</p>
            <button type="button" className={styles.close} onClick={() => setOpen(false)}>
              Close
            </button>
          </div>
        </div>
      ) : null}
    </>
  );
}
