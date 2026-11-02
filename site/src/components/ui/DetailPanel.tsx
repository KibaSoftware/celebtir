"use client";

import { useEffect, useRef, type CSSProperties, type ReactNode } from "react";
import { createPortal } from "react-dom";
import styles from "./DetailPanel.module.css";

/**
 * A research side panel: a drawer from the right on desktop, a bottom sheet
 * on phones. A modal dialog: focus moves in, Tab stays inside, Escape or the
 * backdrop closes it and focus returns to what opened it.
 */
export function DetailPanel({ title, eyebrow, onClose, children, art }: { title: ReactNode; eyebrow?: ReactNode; onClose: () => void; children: ReactNode; art?: "risk" | "tokens" | "compare" }) {
  const panel = useRef<HTMLDivElement>(null);
  const opener = useRef<HTMLElement | null>(null);
  const close = useRef(onClose);

  useEffect(() => { close.current = onClose; }, [onClose]);

  useEffect(() => {
    opener.current = document.activeElement as HTMLElement | null;
    panel.current?.focus();
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const siblings = [...document.body.children].filter((el): el is HTMLElement => el instanceof HTMLElement && el !== panel.current?.parentElement);
    const previousInert = siblings.map(el => el.inert);
    siblings.forEach(el => { el.inert = true; });
    const keys = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close.current();
      } else if (e.key === "Tab" && panel.current) {
        const items = [...panel.current.querySelectorAll<HTMLElement>("a[href], button:not([disabled]), input, select, summary, [tabindex='0']")].filter(el => el.getClientRects().length > 0);
        if (!items.length) return;
        const first = items[0]!;
        const last = items[items.length - 1]!;
        if (e.shiftKey && (document.activeElement === first || document.activeElement === panel.current)) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", keys);
    const back = opener.current;
    return () => {
      document.removeEventListener("keydown", keys);
      document.body.style.overflow = previousOverflow;
      siblings.forEach((el,i) => { el.inert = previousInert[i]!; });
      back?.focus?.();
    };
  }, []);

  const content = (
    <div className={styles.backdrop} onClick={onClose}>
      <div
        ref={panel}
        className={styles.panel}
        style={art ? {"--inspector-art": `url('/art/${art}.webp')`} as CSSProperties : undefined}
        role="dialog"
        aria-modal="true"
        aria-labelledby="detail-panel-title"
        tabIndex={-1}
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.head}>
          <div>
            {eyebrow ? <div className={styles.eyebrow}>{eyebrow}</div> : null}
            <h2 id="detail-panel-title" className={styles.title}>
              {title}
            </h2>
          </div>
          <button type="button" className={styles.close} onClick={onClose} aria-label="Close details">
            <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
              <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
            </svg>
          </button>
        </div>
        <div className={styles.body}>{children}</div>
      </div>
    </div>
  );
  return typeof document === "undefined" ? content : createPortal(content, document.body);
}

/** A titled block inside the panel; `tone` separates observed facts from methodology. */
export function PanelSection({ title, tone, children }: { title: string; tone?: "observed" | "method"; children: ReactNode }) {
  return (
    <section className={styles.section} data-tone={tone}>
      <h3 className={styles.sectionTitle}>{title}</h3>
      {children}
    </section>
  );
}
