"use client";

import { Suspense } from "react";
import { usePathname } from "next/navigation";
import { FreshnessBar, type FreshnessInfo } from "@/components/FreshnessBar";
import { pageChain, type Chain } from "@/lib/scope";
import { useViewState } from "@/lib/useViewState";
import type { ChainFilter } from "@/lib/viewState";

interface Props { variants: Record<ChainFilter, FreshnessInfo>; protocolChains: Record<string, Chain> }
function Pick({ variants, protocolChains }: Props) {
  const [view] = useViewState();
  const fixed = pageChain(usePathname() ?? "/", protocolChains);
  return <FreshnessBar info={variants[fixed ?? view.chain]} />;
}
export function ScopedFreshness(props: Props) {
  return <Suspense fallback={<FreshnessBar info={props.variants.all} />}><Pick {...props} /></Suspense>;
}
