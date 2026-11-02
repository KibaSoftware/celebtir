"use client";

import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { navVisible, withChain } from "@/lib/scope";
import { useViewState } from "@/lib/useViewState";
import type { ChainFilter } from "@/lib/viewState";
import styles from "./Shell.module.css";

export const NAV = [
  ["/", "Summary"],
  ["/risk/", "Risk Analysis"],
  ["/value/", "Value"],
  ["/activity/", "Activity"],
  ["/tokens/", "Tokens"],
  ["/hyperevm/", "HyperEVM"],
  ["/hip3/", "HIP-3"],
  ["/compare/", "Compare"],
  ["/changelog/", "Changes"],
  ["/methodology/", "Methodology"],
] as const;

/** The nav item a path belongs to: exact for Summary, by prefix otherwise. */
export function activeHref(pathname: string): string | null {
  const path = pathname.endsWith("/") ? pathname : `${pathname}/`;
  if (path === "/") return "/";
  return NAV.find(([href]) => href !== "/" && path.startsWith(href))?.[0] ?? null;
}

/**
 * Main navigation: a row of sections with the current one marked (thin gold
 * underline and `aria-current`). Below 900 px it folds into a Menu button.
 */
export function MainNavigation({ chain = "all" }: { chain?: ChainFilter }) {
  const pathname = usePathname() ?? "/";
  const current = activeHref(pathname);
  // The folded menu is open for the path it was opened on: navigating closes it.
  const [openOn, setOpenOn] = useState<string | null>(null);
  const open = openOn === pathname;

  return (
    <nav aria-label="Main" className={styles.nav} data-open={open || undefined} onKeyDown={(e) => { if (e.key === "Escape") { setOpenOn(null); e.currentTarget.querySelector<HTMLButtonElement>("button")?.focus(); } }}>
      <button
        type="button"
        className={styles.menuButton}
        aria-expanded={open}
        aria-controls="main-nav-list"
        onClick={() => setOpenOn(open ? null : pathname)}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
          {open ? (
            <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
          ) : (
            <path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
          )}
        </svg>
        Menu
      </button>
      <ul id="main-nav-list" className={styles.navList}>
        {NAV.filter(([href]) => navVisible(href, chain)).map(([href, label]) => (
          <li key={href}>
            {/* HUI-1: the chosen chain follows the reader to the next page. */}
            <Link href={withChain(href, chain)} className={styles.navLink} aria-current={current === href ? "page" : undefined}>
              {label}
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  );
}

/** The navigation bound to the URL's chain (render inside <Suspense>). */
export function MainNavigationScoped() {
  const [view] = useViewState();
  return <MainNavigation chain={view.chain} />;
}
