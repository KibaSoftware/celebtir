# Methodology changelog

Every change to a published methodology file gets an entry here. A change never edits an old `v<version>.toml`: it adds a new version, so any past day can be re-scored exactly (SNAP-4).

## v0.1 (2026-10-17)

First published version.

- Six factors rated Low, Medium or High: upgrade control, upgrade delay, admin powers, code transparency, oracle dependency and dependencies. No composite score.
- Stages 0 to 2, plus Unrated, with Stage 1 thresholds mirroring L2Beat's (a 5-day exit window, or an emergency multisig of 8+ signers at 75%+).
- Weakest link: a protocol is rated by its weakest core program.
- A multisig whose config authority is one key counts as that key, with no delay ([ADR 005](/methodology/#decisions)).
- A controller Celebtir cannot recognize rates High and caps the Stage at 0 ([ADR 002](/methodology/#decisions)).
- Public but unverified source is Stage 0, with the verified build listed as missing ([ADR 001](/methodology/#decisions)).
- An admin power's delay comes from the time lock of the multisig that holds it ([ADR 003](/methodology/#decisions)).
- Tokens are rated on four factors (supply control, freeze and seize, transfer restrictions, provenance) and get no Stage.

## v0.2 - 2026-10-21

- Verified registry PDA controllers rate by the controlling program's upgrade authority.
- Lending and perpetuals with no registered feeds have Unknown oracle dependency.
- Delayed value control requires at least the Medium upgrade-delay threshold (24 hours).
- Explicit v0.1 replay retains its original scores and serialized reasons. See ADR 025.

- v0.2 risk sentences describe configuration powers without claiming withdrawal is impossible; every elevated token factor includes a reason.

## v0.3 - 2026-10-25

Launch-audit corrections (CELEBTIR_FIX_INSTRUCTIONS.md). The shipped default; v0.1 and v0.2 replay byte for byte (`fixtures/replay/`), and facts collected after v0.3 are lowered to the representation those versions knew before they score them.

- **Missing data is Unknown (F3, D8).** A protocol with no admin accounts read and no sourced registry review that found none rates admin powers Unknown, not Low; price feeds the same, Unknown instead of N/A, for every category. A review (`admin_reviewed_none`, `oracles_reviewed_none`) rates Low or N/A and cites its source; lending and perps can never be reviewed as using no prices. These two factors no longer borrow the core programs' upgrade reads as evidence.
- **Stake-pool authorities (D2).** SPL stake-pool manager, staker and deposit or withdraw authorities are read on chain and hold the parameters power: fees, validator set and deposit gating, not moving or freezing funds.
- **Program-derived addresses (F2, D1).** An off-curve authority is never a key. With a verified registry declaration (seeds re-derived, program executable) it rates as its controlling program's upgrade controller; without one it is a program-derived address whose controlling program is not resolved, rated Unknown (and, for a core program, Unrated). Token supply and freeze criteria are written into the methodology file.
- **Value classes (F7, D4 option B).** Multisig without delay now means an effective delay of zero; a delay above zero and below 24 hours is its own class, `multisig_short_delay`; 24 hours or more (the Medium upgrade-delay bar) is `multisig_with_delay`. Only that class and immutable count as protected.
- **Guarded oracle primary (F4, D3).** A primary outside the established networks (Edge by Chaos Labs, classified "other") rates Low only when the registry records fallback feeds from at least two distinct established networks that bound it. A documented guard that no registered on-chain feed shows is Unknown, never assumed.
- **Stage 2 (H9, D6).** A protocol pause is named "can pause (funds frozen in place)". Token-issuer freeze keys on accepted tokens are rated on token pages and in the collateral warning, not against Stage 2. Unrated has one definition: any Stage 0 requirement unmet.
- **Coverage (F5, D7).** Only Solana protocols are rated; HyperEVM protocols are listed unassessed, with facts and no ratings, Stage or value class. Published days scored under v0.2 keep their files; publication withholds those values.
- **Activity (F8, D5).** The 7-day change is the 7-day total against the previous 7 days over calendar days, and is unavailable when either week has an absent day or a day below 5% of the series median.
- **Wording (F12, H6).** Squads v3 has no time-lock field and is described as "no time lock (Squads v3)", never "0 s"; admin reasons name the account; a single callee is "the 1 callee"; reasons name full addresses, which the site shortens and links.
