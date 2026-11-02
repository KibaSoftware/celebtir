"use client";

import Link from "next/link";
import { Suspense, type ComponentProps } from "react";
import { withChain } from "@/lib/scope";
import { useSearchParams } from "next/navigation";
import { parseView } from "@/lib/viewState";

type Props = ComponentProps<typeof Link> & { href: string };

function Scoped({ href, ...rest }: Props) {
  const params = useSearchParams();
  const view = parseView(new URLSearchParams(params?.toString() ?? ""));
  return <Link prefetch={false} href={/[?&]chain=/.test(href) ? href : withChain(href, view.chain)} {...rest} />;
}

/** HUI-1: an internal link that carries the chosen chain to the next page. */
export function ScopedLink(props: Props) {
  return (
    <Suspense fallback={<Link prefetch={false} {...props} />}>
      <Scoped {...props} />
    </Suspense>
  );
}
