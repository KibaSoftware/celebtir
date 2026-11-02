# Protocol selection review

Snapshot 2026-10-26, live methodology 0.2; capture refreshed 2026-10-28. 12 assessed Solana protocols, four unassessed HyperEVM protocols. HIP-3 remains a separate facts-only domain. No additions or registry activations.

## Current category coverage

- solana / liquid-staking: 5
- solana / perps: 2
- hyper_evm / liquid-staking: 3
- solana / lending: 2
- hyper_evm / lending: 1
- solana / dex: 3

Source observation dates/slots vary by field; snapshot date is not a guarantee every field was freshly read. stage-diagnostics.json preserves all relevant slot/source anchors. Incomplete source verification and unresolved authority paths remain explicit, including both Unrated Solana records. No freshness distribution is inferred from snapshot date alone.

## Peer-diversity matrix

| Candidate | Inventory | Configuration research value | Recommendation |
|---|---|---|---|
| [Orca](candidates/can-01-orca.md) | ACTIVE_EXISTING | Concentrated-liquidity comparator with a published Squads controller and finite upgrade delay. | REASSESS_EXISTING |
| [Marinade liquid staking](candidates/can-02-marinade.md) | NOT_TRACKED | Liquid staking implementation distinct from the shared SPL stake-pool program. Existing token coverage is not protocol coverage. | DRAFT_RESEARCH_ONLY |
| [Jito stake pool](candidates/can-03-jito.md) | ACTIVE_EXISTING | Shared SPL stake-pool controller versus pool-specific management provides an informative control comparison. | REASSESS_EXISTING |
| [Solstice USX / YieldVault](candidates/can-04-solstice.md) | NOT_TRACKED | Separate issuance and yield-vault upgrade roots broaden the authority-path research set; category eligibility needs owner/methodology review. | DRAFT_RESEARCH_ONLY |
| [Phoenix Legacy spot DEX](candidates/can-05-phoenix.md) | NOT_TRACKED | Order-book settlement provides a category comparator to AMMs; this identity is Phoenix Legacy spot, not current Phoenix perpetuals. | DRAFT_RESEARCH_ONLY |

Fresh header observations establish distinct upgrade roots, not stronger overall safety. Orca and Jito already provide AMM/shared-pool comparisons. Marinade adds an independent staking implementation; Phoenix Legacy adds order-book settlement; Solstice adds issuance/vault separation with category applicability still pending. Selection is based on research diversity, not manufacturing a Stage 1 histogram.

Effort is measured in evidence tasks: each prospective protocol needs controller/bypass resolution, privileged-operation inventory, oracle/dependency review, binary verification, isolated fixtures and independent human review (six work packages, potentially multiple accounts each). Solstice additionally requires category/methodology and source-wording reconciliation. Existing Orca/Jito need targeted reassessment of the same unresolved paths rather than duplicate registry rows. No elapsed-time estimate is asserted.

**Zero new admissions.** All dossiers set publication_authorized=false. No candidate meets every owner-review-readiness condition. Human registry review and release authorization remain separate from this packet.
