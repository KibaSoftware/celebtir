"use client";

import { ScopedLink } from "./scope/ScopedLink";
import { useCallback, useEffect, useId, useRef, useState, type CSSProperties, type ReactNode } from "react";
import styles from "./Tooltip.module.css";

/**
 * UI-10: a tooltip that works by keyboard. The trigger is a button: focus or
 * hover opens it, Escape or blur closes it, and the content is the trigger's
 * description, so screen readers announce it. Fixed positioning keeps it
 * visible inside scrolling tables. With `href` the trigger is a link instead
 * (UI-6 glossary terms): it navigates on click and shows the tip on focus.
 */
export function Tooltip({
  trigger,
  label,
  href,
  children,
  onActivate,
  expanded,
}: {
  trigger: ReactNode;
  label?: string;
  href?: string;
  children: ReactNode;
  onActivate?: () => void;
  expanded?: boolean;
}) {
  const id = useId();
  const ref = useRef<HTMLButtonElement & HTMLAnchorElement>(null);
  const tipRef = useRef<HTMLSpanElement>(null);
  const [style, setStyle] = useState<CSSProperties | null>(null);

  const show = useCallback(() => {
    const r = ref.current?.getBoundingClientRect();
    if (!r) return;
    const width = Math.min(320, window.innerWidth - 16);
    const left = Math.max(8, Math.min(r.right - width, window.innerWidth - width - 8));
    setStyle({ ...(r.bottom + 180 > window.innerHeight ? { bottom: window.innerHeight - r.top + 6 } : { top: r.bottom + 6 }), left, width, maxHeight: Math.max(80, window.innerHeight - 24), overflowY: "auto" });
  }, []);
  const hide = useCallback(() => setStyle(null), []);

  // Scrolling a table moves triggers beneath a stationary pointer; do not
  // reopen old hover cards from their scroll listeners. Keyboard focus can
  // scroll the trigger into view, so only a focused trigger follows scrolling.
  const open = style !== null;
  useEffect(() => {
    if (!open) return;
    const onScroll = (event: Event) => {
      if (event.target instanceof Node && tipRef.current?.contains(event.target)) return;
      if (document.activeElement === ref.current) show();
      else hide();
    };
    window.addEventListener("scroll", onScroll, { passive: true, capture: true });
    window.addEventListener("resize", show, { passive: true });
    return () => {
      window.removeEventListener("scroll", onScroll, { capture: true });
      window.removeEventListener("resize", show);
    };
  }, [open, show, hide]);

  const keys = (e: { key: string }) => {
    if (e.key === "Escape") hide();
  };
  if (href) {
    return (
      <span className={styles.termWrap} onMouseEnter={show} onMouseLeave={hide}>
        <ScopedLink ref={ref} href={href} className={styles.term} aria-label={label} aria-describedby={id} onFocus={show} onBlur={hide} onKeyDown={keys}>
          {trigger}
        </ScopedLink>
        <span ref={tipRef} id={id} role="tooltip" className={styles.tip} hidden={!open} style={style ?? undefined}>
          {children}
        </span>
      </span>
    );
  }
  return (
    <span className={styles.wrap} onMouseEnter={show} onMouseLeave={hide}>
      <button
        ref={ref}
        type="button"
        className={styles.trigger}
        aria-label={label}
        aria-describedby={id}
        aria-expanded={expanded ?? open}
        aria-haspopup={onActivate ? 'dialog' : undefined}
        onFocus={show}
        onBlur={hide}
        onClick={() => { show(); onActivate?.(); }}
        onKeyDown={keys}
      >
        {trigger}
      </button>
      <span ref={tipRef} id={id} role="tooltip" className={styles.tip} hidden={!open} style={style ?? undefined}>
        {children}
      </span>
    </span>
  );
}
