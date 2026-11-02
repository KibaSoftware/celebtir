# Ethereum admission decision

Recommendation: **BLOCKED**.

G0 has an unresolved intended-release discrepancy: hosted methodology v0.2 versus expected v0.3, with no exposed deployed source binding. G1 retains three evidence attribution blockers. G5 results and remaining performance/manual checks are recorded in final-review.md and gate-results.json. This is not a verified production baseline for extending scoring to another domain.

## Inspected reuse boundaries

`crates/celebtir-core/src/controller.rs` provides typed controller arrangements; the core facts/evidence model and Rust versioned scorer remain the common model. `crates/collect-hyperliquid/src/evm.rs` wraps shared rate-limited JSON-RPC, checks chain ID **999**, pins a block, and tags storage/code/call reads with that block. It cannot be pointed at Ethereum unchanged: chain identity and finality/block selection need explicit domain support.

`crates/collect-hyperliquid/src/resolve.rs` resolves declared proxy paths, Safe singleton/module and timelock/controller arrangements with bounded recursion. `verify.rs` handles source verification. Reuse these capabilities after fixture-backed generalization; do not create a competing Ethereum-only JavaScript scorer/runtime. Existing HyperEVM support is not proof of exhaustive Ethereum proxy/governor coverage. Unsupported types, non-enumerable proposers and unresolved alternate paths must remain unknown.

## Required admission brief

1. Owner-selected initial protocols, contract/asset identities and core/periphery boundaries, with sourced addresses and independent registry review.
2. Ethereum chain ID, pinned finalized block policy, chain-specific explorer/evidence and provider budget; all dependent reads anchored to the same block.
3. ProxyAdmin, transparent/UUPS/beacon and governor/timelock implementation coverage, including upgrade authorization, proposer/executor permissions, Safe modules/guards, delegatecall and alternate escape routes. No empty inventory interpreted as proof of absence.
4. Reproducible deployed-source verification and explicit incomplete proxy/source states; linked token provenance and oracle authority assumptions.
5. **NO_RATING_UNTIL_METHODOLOGY_REVIEW**. Owner approval of versioned domain applicability is mandatory; Solana Stage labels/thresholds must not transfer by analogy.
6. Explicit API/schema compatibility and history policy, isolated fixtures, hosted release/source binding, maintenance ownership and deployment gates.

No Ethereum source, data, scorer, registry or routes were added. This decision closes only the prerequisite assessment; it is not implementation authorization.
