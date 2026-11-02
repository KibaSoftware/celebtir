import type { Metadata } from "next";
import { SharedControllers } from "@/components/graph/SharedControllers";
import { Termed } from "@/components/Term";
import { loadSnapshot } from "@/lib/data";
import { sharedControllers } from "@/lib/graph";
import { ChainScoped } from "@/components/scope/ChainScoped";
import { scope, SCOPES } from "@/lib/scope";
import type { ChainFilter } from "@/lib/viewState";

export const metadata: Metadata = {
  title: "Shared controllers",
  description: "Keys and multisigs that control more than one rated DeFi protocol.",
};

// UI-22: across the whole ecosystem, each controller listed once.
function ControllersPageBody({ chain }: { chain: ChainFilter }) {
  const shared = sharedControllers(scope(loadSnapshot(), chain));
  return (
    <>
      <h1>Shared controllers</h1>
      <p>
        <Termed
          text="Keys and multisigs that hold a power over more than one protocol, including the authorities of tokens those protocols use. One key can affect every protocol listed under it."
          seen={new Set()}
        />
      </p>
      <SharedControllers shared={shared} />
    </>
  );
}

// HUI-1: rendered once per chain; the picker chooses the variant.
export default function ControllersPage() {
  const variants = Object.fromEntries(SCOPES.map((c) => [c, <ControllersPageBody key={c} chain={c} />])) as Record<ChainFilter, React.ReactNode>;
  return <ChainScoped variants={variants} />;
}
