"use client";
import { Children, cloneElement, isValidElement, useEffect, useId, useRef, useState, type ReactNode } from "react";
import { focusHashTarget } from "./protocol/HashFocus";
import styles from "./LimitedItems.module.css";

/** Keep long evidence lists usable; deep links reveal their target. */
export function LimitedItems({ children, table = false, ordered = false, colSpan = 3, className, label }: { children: ReactNode; table?: boolean; ordered?: boolean; colSpan?: number; className?: string; label: string }) {
  const items = Children.toArray(children);
  const [expanded, setExpanded] = useState(false);
  const root = useRef<HTMLElement | null>(null);
  const id = useId();
  useEffect(() => {
    const reveal = () => {
      let target: HTMLElement | null;
      try { target = document.getElementById(decodeURIComponent(location.hash.slice(1))); } catch { return; }
      if (!target || !root.current?.contains(target)) return;
      setExpanded(true);
      requestAnimationFrame(() => focusHashTarget());
    };
    const timer = setTimeout(reveal, 0);
    window.addEventListener("hashchange", reveal);
    window.addEventListener("popstate", reveal);
    return () => { clearTimeout(timer); window.removeEventListener("hashchange", reveal); window.removeEventListener("popstate", reveal); };
  }, [children]);
  const records = items.map((item, index) => isValidElement<{ hidden?: boolean; className?: string }>(item)
    ? cloneElement(item, { hidden: !expanded && index >= 10, className: [item.props.className, !expanded && index >= 10 ? styles.hidden : ""].filter(Boolean).join(" ") }) : item);
  const control = items.length > 10 ? <div className={styles.controls}>
    <span>Showing {expanded ? items.length : 10} of {items.length} {label}</span>
    <button type="button" aria-expanded={expanded} aria-controls={id} onClick={() => setExpanded(v => !v)}>{expanded ? "Show fewer" : `Show all ${items.length}`}<span className="sr-only"> {label}</span></button>
  </div> : null;
  if (table) return <tbody id={id} ref={node => { root.current = node; }} className={className}>{records}{control ? <tr><td colSpan={colSpan}>{control}</td></tr> : null}</tbody>;
  const List = ordered ? "ol" : "ul";
  return <List id={id} ref={node => { root.current = node; }} className={className} aria-label={label}>{records}{control ? <li className={styles.controlItem}>{control}</li> : null}</List>;
}
