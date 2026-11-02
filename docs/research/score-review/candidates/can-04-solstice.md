# can-04-solstice: Solstice USX / YieldVault

## 1. Identity and inventory

Research ID: can-04-solstice. Domain: solana. Category: stablecoin / yield (applicability pending). Classification: **NOT_TRACKED**. Immutable registered identifier: none; research ID only. Registry: no protocol entry or inactive draft found in active/draft inventory. Checked HEAD: b114b281ae6a57d3d6ff198963087e999323a5eb, dirty working tree (2026-10-28); HEAD alone does not bind untracked registry contents. Protected-input manifest binds the inspected files. 

## 2. Inclusion rationale

Separate issuance and yield-vault upgrade roots broaden the authority-path research set; category eligibility needs owner/methodology review. No TVL ranking or invented economic metric is used. Stage 0 is not an exclusion criterion. [Primary identity source](https://docs.solstice.finance/solstice-for-builders/smart-contracts).

## 3. Programs and roles

The following primary-source program identities were read independently. Each is executable and owned by the Solana upgradeable loader. Core-role completeness is UNVERIFIED; these are candidate core identities, not an admitted exhaustive inventory. Periphery: UNKNOWN, not asserted absent.

| Program | ProgramData | Observed upgrade authority | Observation slot | Last deploy slot |
|---|---|---|---:|---:|
| [USXyiSTsPEWz55pSK7sZoUL79ntoVGQbaTDT57tH6bx](https://solscan.io/account/USXyiSTsPEWz55pSK7sZoUL79ntoVGQbaTDT57tH6bx) | [3P5mNf86DSd2CNu1DToPBxbvqQL4V2kNwAQjErCwZxMq](https://solscan.io/account/3P5mNf86DSd2CNu1DToPBxbvqQL4V2kNwAQjErCwZxMq) | [BzMuMbtah9bqN4HERkPo2PsVoaQKD2xfvYq1xfDCFwEb](https://solscan.io/account/BzMuMbtah9bqN4HERkPo2PsVoaQKD2xfvYq1xfDCFwEb) | 462475378 | 441672563 |
| [eUSXyKoZ6aGejYVbnp3wtWQ1E8zuokLAJPecPxxtgG3](https://solscan.io/account/eUSXyKoZ6aGejYVbnp3wtWQ1E8zuokLAJPecPxxtgG3) | [BVDochYTcRGvCYmmdtsiX9qT63fy5dmGSB5hNgxMTbJY](https://solscan.io/account/BVDochYTcRGvCYmmdtsiX9qT63fy5dmGSB5hNgxMTbJY) | [CYr14TrXD5MAertRqcR58KirPz7XoHGZjfBsfn5M2voh](https://solscan.io/account/CYr14TrXD5MAertRqcR58KirPz7XoHGZjfBsfn5M2voh) | 462475378 | 416855566 |

## 4. Upgrade controller resolution

Observed path: executable program → ProgramData → non-null authority, for every row above. The header capture does **not** decode the terminal authority, voter permissions, threshold, configuration authority or alternate bypass. Those current fields are UNKNOWN. Documentation about multisigs is a research lead, not an observed threshold.

## 5. Timelocks

Effective delay per upgrade, parameter change, emergency action and withdrawal: UNKNOWN in this fresh capture. No timelock account or executable enforcement path was decoded here. Required follow-up: resolve each authority, its configuration/admin routes and operation-specific bypasses; record delay parameters and whether a separate actor can shorten them. A non-null authority is not evidence of either zero delay or a timelock.

## 6. Administrative permissions

Docs describe minting/redemption, administrative configuration and oracle roles. Actors, scope, delay, freeze and withdrawal powers remain UNKNOWN pending decoded accounts. The executable-header read establishes upgradeability only. Exact privileged actors, operations, immediacy/delay and alternate paths require the corresponding account/source review; undeclared paths are not proven absent.

## 7. Oracle, dependency, source and token evidence

Primary docs identify an oracle relationship but do not establish its deployed identity/controller in this capture. USX issuance and YieldVault require separate token/mint/freeze and redemption evidence. No deployed-bytecode build was independently reproduced in this capture.  Source capture hashes are in research/sources.json. Token provenance/mint/freeze powers are UNKNOWN for this packet unless explicitly present in the older published record; no token facts were modified.

## 8. Capture provenance and integrity

UTC: 2026-10-28T12:24:55.722Z. RPC host: api.mainnet-beta.solana.com; finalized commitment; 2 getMultipleAccounts calls, bounded header slices. Decoder: Solana UpgradeableLoaderState Program discriminator (2), ProgramData discriminator (3), little-endian deployment slot and authority option, implemented in scripts/score-review-research.mjs. Raw files: output/reviews/score-review-2026-10-26/research-solstice/program-accounts.json and programdata-accounts.json. Program/ProgramData fields and exact accounts are enumerated above. SHA-256:

- program-accounts: 7c65a16b07a798e13382ae192852ec9f2f0266f40dda054b7b5a7339a383229f
- programdata-accounts: 87a3fabff0b188c5ff3fb43585a2c7a3f6fc2f7c784a47d0b3d52fd535babac4

Explorer links are navigation aids, not time-pinned copies; the raw RPC captures bind these observations. Two batches can have different context slots; use each recorded field's slot.

## 9. Six-factor evidence sufficiency

| Factor | Fresh packet sufficiency | Published assessment |
|---|---|---|
| upgrade_control | INCOMPLETE: root observed; terminal controller and alternate paths unresolved | DRAFT / UNPUBLISHED; no rating computed |
| upgrade_delay | INCOMPLETE: no operation-specific enforcement decode | DRAFT / UNPUBLISHED; no rating computed |
| admin_powers | INCOMPLETE: complete privileged-action inventory not verified | DRAFT / UNPUBLISHED; no rating computed |
| oracle_dependency | INCOMPLETE: applicability and complete authority chain not verified | DRAFT / UNPUBLISHED; no rating computed |
| code_transparency | INCOMPLETE: no independent deployed binary verification | DRAFT / UNPUBLISHED; no rating computed |
| dependencies | INCOMPLETE: complete external authority/dependency inventory not verified | DRAFT / UNPUBLISHED; no rating computed |

Published Stage: none; no hypothetical Stage awarded. The selected research comparison is v0.2 live versus v0.3 local; admission requires applicability review, not transfer of an existing Stage.

## 10. Gaps, conflicts and executable follow-up

Source wording about non-upgradeability must distinguish SPL token accounts from the custom USX program. The captured custom program has a non-null upgrade authority; do not generalize marketing assurances across these distinct targets. Resolve the source wording with maintainers and decode both authority accounts.

For every listed program: capture terminal controller account and owner, decode voter permissions/quorum and all config/module/admin alternatives, identify privileged-operation accounts, verify timelock enforcement, establish oracle/dependency scope, reproduce deployed-source verification, and add isolated fixtures. Obtain independent Solana ground-truth reviewer identity, slot, UTC verification time and current cooling-policy review. Proposed reviewer role: owner-appointed Solana program/controller researcher; no reviewer has signed off. Existing entries require reassessment without changing their publication status. No missing authority path is waived.

## 11. Recommendation

review_recommendation: **DRAFT_RESEARCH_ONLY**

review_readiness_reasons:

- Met: primary identity reference and fresh executable/ProgramData/authority-root reads with hashes.
- Missing: complete current controller and bypass paths, operation-specific delay/admin inventory, source/dependency sufficiency, isolated collector fixtures and independent human review.
- Executable evidence-gathering actions exist; remain a research packet only.

publication_authorized=false
