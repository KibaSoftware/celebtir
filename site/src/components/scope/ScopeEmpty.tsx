import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { CHAIN_LABELS } from "@/lib/scope";
import type { ChainFilter } from "@/lib/viewState";

export type ScopeKind = "protocols" | "tokens" | "changes" | "controllers" | "activity" | "value";

const KIND: Record<ScopeKind, string> = {
  protocols: "protocols",
  tokens: "tokens",
  changes: "changes",
  controllers: "shared controllers",
  activity: "protocols with activity data",
  value: "protocols with TVL",
};

/**
 * HUI-1: the one empty state for a chain with nothing to show. Filter-caused
 * empties keep their own "Show all"; this one is about the chain itself.
 */
export function ScopeEmpty({ chain, kind }: { chain: ChainFilter; kind: ScopeKind }) {
  const where = chain === "all" ? "" : `${CHAIN_LABELS[chain]} `;
  return (
    <p data-empty="" data-scope-empty={chain}>
      No {where}
      {KIND[kind]} in this snapshot yet.
      {chain === "hyperliquid" ? (
        <>
          {" "}
          Hyperliquid&apos;s builder-deployed perp markets are listed on <Link href="/hip3/">HIP-3 DEXs</Link>.
        </>
      ) : null}
    </p>
  );
}
