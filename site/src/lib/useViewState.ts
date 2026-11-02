"use client";

import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useCallback, useMemo, useState } from "react";
import { parseView, serializeView, type ViewState } from "./viewState";

/**
 * The page's view state, read from and written to the URL (UI-1). Updates
 * replace the history entry, so filtering does not flood the back button.
 * Pages using this must render it inside <Suspense> (static export).
 *
 * `router.replace` applies asynchronously, so an update is shown at once and
 * the URL catches up: the pending view is kept only while the URL it was made
 * from is still current, so the URL stays the source of truth.
 */
export function useViewState(): [ViewState, (next: (v: ViewState) => ViewState) => void] {
  const params = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();
  const current = params.toString();
  const fromUrl = useMemo(() => parseView(new URLSearchParams(current)), [current]);
  const [pending, setPending] = useState<{ from: string; view: ViewState } | null>(null);
  const view = pending && pending.from === current ? pending.view : fromUrl;
  const update = useCallback(
    (next: (v: ViewState) => ViewState) => {
      const v = next(view);
      setPending({ from: current, view: v });
      const query = serializeView(v);
      router.replace(query ? `${pathname}?${query}` : pathname, { scroll: false });
    },
    [view, current, router, pathname],
  );
  return [view, update];
}
