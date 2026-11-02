"use client";

import { usePathname } from "next/navigation";
import { CHAIN_LABELS, pageChain, type Chain } from "@/lib/scope";
import { useViewState } from "@/lib/useViewState";
import { CHAINS, DEFAULT_VIEW, type ViewState } from "@/lib/viewState";
import styles from "./Shell.module.css";

const HINTS: Record<Chain, string> = {
  solana: "This page is about one Solana entity, so the chain is fixed here.",
  hyperliquid: "This page is about Hyperliquid, so the chain is fixed here.",
};

/** UI-9, HUI-1: the chain filter, kept in the URL (`chain=`). */
export function ChainSelectorView({
  view,
  update,
  fixed = null,
}: {
  view: ViewState;
  update: (next: (v: ViewState) => ViewState) => void;
  /** Set on pages about one chain: the picker shows it and does not change. */
  fixed?: Chain | null;
}) {
  const active = fixed ?? view.chain;
  return (
    <div
      className={styles.chains}
      role="group"
      aria-label="Chain"
      aria-describedby={fixed ? "chain-fixed-hint" : undefined}
      data-fixed={fixed ?? undefined}
      title={fixed ? HINTS[fixed] : undefined}
    >
      {CHAINS.map((c) => (
        <button
          key={c}
          type="button"
          className={styles.chain}
          aria-pressed={active === c}
          aria-disabled={fixed ? true : undefined}
          data-active={active === c ? "" : undefined}
          onClick={fixed ? undefined : () => update((v) => ({ ...v, chain: c }))}
        >
          {active === c ? <span className={styles.chainDot} aria-hidden="true" /> : null}
          {CHAIN_LABELS[c]}
        </button>
      ))}
      {fixed ? (
        <span id="chain-fixed-hint" className="sr-only">
          {HINTS[fixed]}
        </span>
      ) : null}
    </div>
  );
}

export function ChainSelector({ protocolChains = {} }: { protocolChains?: Record<string, Chain> }) {
  const [view, update] = useViewState();
  const fixed = pageChain(usePathname() ?? "/", protocolChains);
  return <ChainSelectorView view={view} update={update} fixed={fixed} />;
}

/** The pre-rendered selector before the URL is read (static export). */
export function ChainSelectorDefault() {
  return <ChainSelectorView view={DEFAULT_VIEW} update={() => {}} />;
}
