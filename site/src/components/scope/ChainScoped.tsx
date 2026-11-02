"use client";

import { Suspense, type ReactNode } from "react";
import { useViewState } from "@/lib/useViewState";
import type { ChainFilter } from "@/lib/viewState";

export type Variants = Record<ChainFilter, ReactNode>;

function Pick({ variants }: { variants: Variants }) {
  const [view] = useViewState();
  return <>{variants[view.chain]}</>;
}

/**
 * HUI-1: a page section rendered once per chain at build time; the browser
 * shows the variant the picker asks for. The static HTML (and the page
 * without JavaScript) shows "All chains".
 */
export function ChainScoped({ variants }: { variants: Variants }) {
  return (
    <Suspense fallback={variants.all}>
      <Pick variants={variants} />
    </Suspense>
  );
}
