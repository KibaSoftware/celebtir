// UI-6: glossary terms, linked with an inline definition at their first use
// on each page.

export interface Term {
  slug: string;
  term: string;
  /** Other spellings matched in text. */
  aliases?: string[];
  definition: string;
}

export const GLOSSARY: Term[] = [
  { slug: "admin-key", term: "admin key", aliases: ["admin keys"], definition: "A key stored in a protocol's own accounts that can change parameters, pause the protocol or move funds, without upgrading its code." },
  { slug: "config-authority", term: "config authority", definition: "A key that can change a Squads multisig's members, threshold and time lock without a vote. If it is set, it, not the multisig, is the real controller." },
  { slug: "cpi", term: "cross-program invocation", aliases: ["CPI"], definition: "How one Solana program calls another inside a transaction. Celebtir samples these calls to find a protocol's dependencies." },
  { slug: "effective-controller", term: "effective controller", definition: "Whoever can finally act once control is followed to its end: through vaults, multisigs and config authorities." },
  { slug: "evidence", term: "evidence", definition: "The on-chain account, field and slot a fact was read from, with a link to check it on Solscan." },
  { slug: "freeze-authority", term: "freeze authority", definition: "A key that can freeze any holder's balance of a token, so it cannot be moved." },
  { slug: "immutable", term: "immutable", definition: "No key can change it: the upgrade authority was removed, or the program was deployed with a loader that cannot upgrade." },
  { slug: "mint-authority", term: "mint authority", definition: "A key that can create new supply of a token." },
  { slug: "multisig", term: "multisig", aliases: ["multisigs"], definition: "An account that acts only when enough of its members approve. On Solana this is usually a Squads multisig." },
  { slug: "oracle", term: "oracle", aliases: ["oracles", "price feed"], definition: "An account that publishes prices a protocol reads, for example to value collateral or trigger liquidations." },
  { slug: "permanent-delegate", term: "permanent delegate", definition: "A Token-2022 extension that lets one key move or burn tokens from any wallet, without the holder's signature." },
  { slug: "programdata", term: "ProgramData", definition: "The account that holds an upgradeable Solana program's code and its upgrade authority." },
  { slug: "signer", term: "signer", aliases: ["signers", "voters"], definition: "A multisig member with the Vote permission. Celebtir counts signers, not all members, when it compares a threshold." },
  { slug: "slot", term: "slot", definition: "Solana's clock: each fact records the slot at which it was read." },
  { slug: "squads", term: "Squads", definition: "The multisig program most Solana protocols use to hold upgrade and admin keys. Celebtir reads Squads v4 (members, threshold, time lock, config authority) and Squads v3, which has no time-lock field: a v3 multisig is shown as having no time lock." },
  { slug: "stage", term: "Stage", aliases: ["Stages"], definition: "A protocol's headline level. Unrated: a Stage 0 requirement is unmet (core programs not identified, no public source, or upgrade control unreadable). Stage 0: the team can change everything quickly. Stage 1: upgrades wait at least five days, or need an emergency multisig of 8+ signers at 75%+, with every factor readable and builds verified. Stage 2: no quick upgrades and no admin key that can move funds or pause. Stages describe control, not code security." },
  { slug: "threshold", term: "threshold", definition: "How many signers of a multisig must approve before it can act." },
  { slug: "time-lock", term: "time lock", aliases: ["timelock", "time locks"], definition: "A wait between approving a change and the change taking effect, which gives users time to leave." },
  { slug: "token-2022", term: "Token-2022", definition: "Solana's newer token program. Its extensions can add powers such as a permanent delegate, a transfer hook or a pause switch." },
  { slug: "transfer-hook", term: "transfer hook", definition: "A Token-2022 extension that runs a program on every transfer; that program can refuse transfers." },
  { slug: "tvl", term: "TVL", definition: "Total value locked: the value of assets deposited in a protocol, from DefiLlama. Context only; it never changes a rating." },
  { slug: "upgrade-authority", term: "upgrade authority", definition: "The key or account that can replace a Solana program's code. It is stored in the program's ProgramData account." },
  { slug: "verified-build", term: "verified build", aliases: ["verified builds"], definition: "Proof that the deployed program was built from a public commit, checked by OtterSec's verification service." },
  { slug: "weakest-link", term: "weakest link", definition: "A protocol is rated by its weakest core program: one single-key program caps the whole protocol." },
];

export type Piece = string | { slug: string; text: string; definition: string };

const escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

const patterns = GLOSSARY.flatMap((t) => [t.term, ...(t.aliases ?? [])].map((text) => ({ text, t })))
  // Longest first, so "upgrade authority" wins over "authority"-like overlaps.
  .sort((a, b) => b.text.length - a.text.length);
const matcher = new RegExp(`\\b(${patterns.map((p) => escape(p.text)).join("|")})\\b`, "gi");
const byText = new Map(patterns.map((p) => [p.text.toLowerCase(), p.t]));

/**
 * Splits `text` into plain strings and glossary terms. Only the first use of
 * each term on the page links; `seen` carries that state across calls.
 */
export function withTerms(text: string, seen: Set<string>): Piece[] {
  const out: Piece[] = [];
  let last = 0;
  for (const m of text.matchAll(matcher)) {
    const t = byText.get(m[0].toLowerCase());
    if (!t || seen.has(t.slug)) continue;
    seen.add(t.slug);
    if (m.index > last) out.push(text.slice(last, m.index));
    out.push({ slug: t.slug, text: m[0], definition: t.definition });
    last = m.index + m[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}
