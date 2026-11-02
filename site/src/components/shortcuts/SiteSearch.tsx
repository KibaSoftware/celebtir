"use client";

import { useRouter, useSearchParams } from "next/navigation";
import { useEffect, useId, useRef, useState } from "react";
import { KIND_ORDER, searchMatches, type SearchItem } from "@/lib/search";
import { parseView } from "@/lib/viewState";
import { withChain } from "@/lib/scope";
import styles from "./Shortcuts.module.css";

export type { SearchItem } from "@/lib/search";

/**
 * The global command search (UI-3): protocols, tokens and controllers by
 * name, id or address. `/` focuses it on pages without their own search box;
 * ↑ and ↓ move through the results, Enter opens one, Escape clears.
 * An ARIA combobox: focus stays in the field while the active option moves.
 */
export function SiteSearch({ items, navigate }: { items: SearchItem[]; navigate?: (href: string) => void }) {
  const router = useRouter();
  const params = useSearchParams();
  const chain = parseView(new URLSearchParams(params?.toString() ?? "")).chain;
  const go = navigate ?? ((href: string) => router.push(withChain(href, chain)));
  const [q, setQ] = useState("");
  const [active, setActive] = useState(0);
  const [open, setOpen] = useState(false);
  const id = useId();
  const root = useRef<HTMLDivElement>(null);
  const matches = searchMatches(items.filter(i => chain === "all" || i.chain === chain), q);
  const shown = open && q.trim().length > 0;
  const current = matches[Math.min(active, matches.length - 1)];

  useEffect(() => {
    const shortcut = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault(); root.current?.querySelector("input")?.focus(); setOpen(true);
      }
    };
    const away = (e: MouseEvent) => {
      if (root.current && !root.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", away);
    document.addEventListener("keydown", shortcut);
    return () => { document.removeEventListener("mousedown", away); document.removeEventListener("keydown", shortcut); };
  }, []);

  const choose = (item: SearchItem) => {
    go(item.href);
    setQ("");
    setActive(0);
    setOpen(false);
  };

  return (
    <div className={styles.search} role="search" ref={root}>
      <svg className={styles.searchIcon} width="15" height="15" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
        <circle cx="7" cy="7" r="4.8" fill="none" stroke="currentColor" strokeWidth="1.6" />
        <path d="M10.6 10.6L14 14" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
      </svg>
      <input
        type="search"
        role="combobox"
        data-shortcut="site-search"
        aria-label="Search protocols, tokens and controllers"
        aria-expanded={shown}
        aria-controls={`${id}-list`}
        aria-autocomplete="list"
        aria-activedescendant={shown && current ? `${id}-${matches.indexOf(current)}` : undefined}
        placeholder="Search protocols, tokens…"
        autoComplete="off"
        spellCheck={false}
        value={q}
        onFocus={() => setOpen(true)}
        onChange={(e) => {
          setQ(e.target.value);
          setActive(0);
          setOpen(true);
        }}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown" && matches.length) {
            e.preventDefault();
            setOpen(true);
            setActive((a) => (a + 1) % matches.length);
          } else if (e.key === "ArrowUp" && matches.length) {
            e.preventDefault();
            setActive((a) => (a - 1 + matches.length) % matches.length);
          } else if (e.key === "Enter" && current) {
            e.preventDefault();
            choose(current);
          } else if (e.key === "Escape") {
            setQ("");
            setOpen(false);
          }
        }}
      />
      <kbd className={styles.searchKey} aria-hidden="true">
        /
      </kbd>
      <div className={styles.results} hidden={!shown}>
        <ul id={`${id}-list`} role="listbox" aria-label="Search results">
          {KIND_ORDER.map((kind) => {
            const group = matches.filter((m) => m.kind === kind);
            if (!group.length) return null;
            return (
              <li key={kind} role="presentation">
                <div className={styles.group} role="presentation">
                  {kind === "Controller" ? "Controllers" : `${kind}s`}
                </div>
                <ul role="group" aria-label={`${kind}s`}>
                  {group.map((m) => {
                    const i = matches.indexOf(m);
                    return (
                      <li
                        key={m.href + m.label}
                        id={`${id}-${i}`}
                        role="option"
                        aria-selected={i === active}
                        data-href={m.href}
                        className={styles.option}
                        onMouseEnter={() => setActive(i)}
                        onMouseDown={(e) => e.preventDefault()}
                        onClick={() => choose(m)}
                      >
                        <span className={styles.optionLabel}>{m.label}</span>
                        {m.detail ? <span className={styles.optionDetail}>{m.detail}</span> : null}
                        <span className={styles.kind}>{m.kind}</span>
                      </li>
                    );
                  })}
                </ul>
              </li>
            );
          })}
          {matches.length === 0 ? (
            <li role="presentation" className={styles.none}>
              Nothing in this snapshot matches &ldquo;{q.trim()}&rdquo;.
            </li>
          ) : null}
        </ul>
      </div>
    </div>
  );
}
