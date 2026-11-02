import type { Metadata } from "next";
import { Suspense } from "react";
import { TokenTable, TokenTableDefault, type TokenTableRow } from "@/components/page/TokenTable";
import { EmptyState, PageHeader } from "@/components/ui/Ui";
import { loadSnapshot } from "@/lib/data";
import { TOKEN_FACTOR_LABELS, TOKEN_KIND_LABELS, tokenRows } from "@/lib/pages";
import { TOKEN_FACTORS, type TokenFactor } from "@/lib/types";
import { ChainScoped } from "@/components/scope/ChainScoped";
import { ScopeEmpty } from "@/components/scope/ScopeEmpty";
import { scope, SCOPES } from "@/lib/scope";
import type { ChainFilter } from "@/lib/viewState";

export const metadata: Metadata = {
  title: "Tokens",
  description: "Who can mint, freeze, move or block the tokens the rated DeFi protocols accept.",
};

function TokensPageBody({ chain }: { chain: ChainFilter }) {
  const s = scope(loadSnapshot(), chain);
  const rows: TokenTableRow[] = tokenRows(s).map((t) => {
    const facts = s.facts.tokens[t.mint]!;
    const score = s.scores.tokens[t.mint];
    const prov = facts.provenance.value;
    return {
      mint: t.mint,
      symbol: t.symbol,
      kind: t.kind,
      issuer: t.issuer,
      program: facts.program,
      provenance: prov.kind === "native" ? "Native to Solana" : `Bridged via ${prov.bridge}`,
      ratings: t.ratings,
      scores: score?.factors,
      methodology: s.scores.methodology_version,
      observedAt: s.snapshotDate ?? s.run.started_at.slice(0, 10),
      authorities: {supply_control: facts.mint_authority.value, freeze_and_seize: facts.freeze_authority.value},
      additionalAuthorities: [
        ...(facts.extensions.value.permanent_delegate ? [{factor:"freeze_and_seize" as const,label:"Permanent delegate",controller:facts.extensions.value.permanent_delegate}] : []),
        ...(facts.extensions.value.pause_authority ? [{factor:"freeze_and_seize" as const,label:"Pause authority",controller:facts.extensions.value.pause_authority}] : []),
        ...(facts.extensions.value.transfer_hook ? [{factor:"transfer_restrictions" as const,label:"Transfer-hook authority",controller:facts.extensions.value.transfer_hook.authority}] : []),
        ...(facts.extensions.value.transfer_hook?.program_controller ? [{factor:"transfer_restrictions" as const,label:"Transfer-hook program controller",controller:facts.extensions.value.transfer_hook.program_controller}] : []),
        ...(facts.extensions.value.transfer_fee_authority ? [{factor:"transfer_restrictions" as const,label:"Transfer-fee authority",controller:facts.extensions.value.transfer_fee_authority}] : []),
      ],
      reasons: Object.fromEntries(TOKEN_FACTORS.map((f) => [f, score?.factors[f]?.reason ?? "not scored"])) as Record<TokenFactor, string>,
      exposure: t.exposure,
      risks: (score?.risks ?? []).map((r) => ({ severity: r.severity, text: r.text })),
    };
  });
  const props = { rows, factors: TOKEN_FACTORS, factorLabels: TOKEN_FACTOR_LABELS, kindLabels: TOKEN_KIND_LABELS };
  return (
    <>
      <PageHeader art="tokens"
        eyebrow="Asset-risk explorer"
        title="Tokens"
        subtitle="What you actually hold. Each token the rated protocols accept is rated on four factors: who can mint it, who can freeze or move it, what can block transfers, and where it comes from. These are the issuer's powers, separate from the risks of the protocols that accept the token."
      />
      {rows.length ? (
        <Suspense fallback={<TokenTableDefault {...props} />}>
          <TokenTable {...props} />
        </Suspense>
      ) : (
        <EmptyState kind="none">
          <ScopeEmpty chain={chain} kind="tokens" />
        </EmptyState>
      )}
    </>
  );
}

// HUI-1: rendered once per chain; the picker chooses the variant.
export default function TokensPage() {
  const variants = Object.fromEntries(SCOPES.map((c) => [c, <TokensPageBody key={c} chain={c} />])) as Record<ChainFilter, React.ReactNode>;
  return <ChainScoped variants={variants} />;
}
