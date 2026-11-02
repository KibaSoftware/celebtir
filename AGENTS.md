# Celebtir project context and agent guide

Last consolidated: 2026-10-26. Read this file first for any iteration. It replaces the root PRD, iteration specifications, backlog, progress log, QA plan and Claude working notes. This is the maintained project entry point; update it when behavior, status, commands or ownership changes.

## Orientation and authority

Celebtir is an open, evidence-linked scorecard of DeFi control and trust assumptions. It shows who can upgrade code, change parameters, pause operations, control token supply and influence prices. It is not a smart-contract security audit or investment recommendation. Users can inspect evidence, compare protocols, explore control graphs and consume static JSON/Markdown outputs.

The implementation covers Solana, HyperEVM and unrated HyperCore HIP-3 DEX facts. Support in code does not imply published coverage: the preserved 2026-10-21 live dataset has 12 Solana protocols and 29 tokens. The reviewed v0.2 candidate exports 57 routes. HyperEVM registry review and live cross-chain publication remain unfinished.

Use current owner instructions, this guide's current decisions, versioned methodology and implementation evidence together. The requirement register below preserves original acceptance contracts; explicit current decisions here supersede obsolete wording there. Do not treat a passing test or a historical checked box as proof of current correctness. Inspect the relevant code and data before changing it. Record substantive new decisions here.

the old automatic commit loop and two-session ownership split no longer apply. No commit, push, deployment or registry activation was performed during the QA implementation.

## Current state and immediate next work

- 2026-11-02 application v0.1.1 preparation: Rust workspace, six internal dependency requirements, all seven local Cargo lock entries and the private site package now use `0.1.1`. Root `CHANGELOG.md` records the initial source baseline; README distinguishes application, methodology and schema versions. Cargo metadata validates offline/locked and all four CLI release tests pass, including actual `--version`. The 235 third-party Cargo lock entries, pnpm lockfile and 397 protected historical inputs are unchanged. Suggested initial commit message: `output/version-0.1.1/INITIAL_COMMIT_MESSAGE.txt`; verification: `output/version-0.1.1/`. Owner will commit manually. Nothing staged, committed, tagged, pushed or deployed; no new full release-gate claim.

- 2026-11-02 first-commit hygiene review: `.gitignore` now excludes local Claude worktrees, browser sessions (including nested site sessions), redesign scratch work, `output/`, runtime collector cache, credentials and compiler/test artifacts. Owner-local `deploy.ps1` and `.env` remain ignored; no original files were deleted. Historical data, curated fixtures, registry/drafts, methodology, schemas, artwork and screenshot baselines remain eligible. `.gitattributes` preserves exact bytes for historical/versioned evidence rather than normalizing line endings. Owner approved a new root README. GitHub daily collection/data commits/Pages publication and its monitor require explicit `CELEBTIR_ENABLE_GITHUB_DAILY=true`, matching the host-first deployment decision.
- First-commit verification: a commit-eligible source copy with a frozen offline pnpm install passes normal static build (34 demo pages), typecheck, 315 UI tests, JS budget and candidate checks; targeted browser tests pass 13 cases with five duplicate phone-gallery skips. Fixed fresh-checkout type generation and test assumptions that depended on local QA entities/run dates, retaining assertions and using committed replay fixtures. Lint and workflow opt-in checks pass; 397 protected input hashes are unchanged. The candidate is approximately 801 files/28.4 MiB; no local artifacts or likely live credentials were found in the reviewed candidate. Evidence: `output/first-commit-review/REPORT.md`. This is repository hygiene validation, not a new full Rust/CI/Lighthouse/release gate. No staging, commit, push, deployment or registry activation.

- 2026-11-02 deployment password reuse: each real `deploy.ps1 -Mode Install/Deploy` invocation now prompts once, with hidden input before packaging, and reuses the account password for SSH/SCP/sudo/cleanup. DryRun and PackageOnly do not prompt. A temporary user-only DPAPI credential and locally compiled helper are removed in `finally`; prior askpass environment is restored. Passwords are not passed in argv, plaintext environment variables, archives or remote files. Remote echo is disabled before password transfer; a framed read and `/dev/tty` handoff preserve subsequent Install credential prompts even with cached/NOPASSWD sudo. SSH host keys must already be verified; encrypted identity keys must be unlocked separately. Same SSH/sudo account password is assumed. Offline password transport and entry tests pass in PowerShell 7 and Windows PowerShell 5.1; generated remote Bash syntax passes. Evidence: `output/deployment-password/`; details: `docs/deployment.md`. No host connection or deployment was performed; live SSH/sudo/PTY verification remains outstanding.

- 2026-11-02 platform-wide clarity polish: owner approved assessment-first presentation for DeFi-literate readers across existing routes. Protocol sections now place risks, Stage, permissions and control graphs before economic context, preserving fragment IDs. Concise findings retain complete recorded reasons, criteria and evidence in disclosures; Stage counts reuse the existing reviewed met/verified-unmet/unresolved results. Token powers precede identity metadata, with an explicit boundary between an authority address and a human signer. Shared lists show ten records initially, retain ordered histories and correct table spans, and reveal/focus nested deep-link targets on initial navigation and browser history. Shared layout fixes address intrinsic card heights, mobile facts and the glossary index. HyperEVM remains unassessed and HIP-3 unrated; no scoring or public API change.
- Clarity validation: 315 UI tests pass; lint, typecheck, 117-page static export and JS budget pass against preserved `output/qa-v03-data` (snapshot 2026-10-26, methodology v0.3). Full browser suite plus focused corrective reruns yield 151 unique passing cases and 31 documented skips; final accessibility/disclosure rerun is 45 passed/one duplicate-gallery skip. All 397 protected input hashes and 51 exported API v2 files remain identical. Before/after captures cover 19 route/entity examples at 1440×900, 768×1024, 375×812 and 720×450 equivalent reflow, plus protocol section comparisons. Review: `docs/design/clarity-polish-review.md`; annotated gallery: `output/playwright/clarity-polish/index.html`; logs and provenance: `output/clarity-polish/`. Native browser zoom, physical-device interaction and owner visual acceptance remain open. No new packages, collection, registry activation, commit, push or deployment.

- 2026-10-29 overview hover-card fix: shared tooltips dismiss on page/table scrolling instead of reopening previous hover cards. Keyboard-focused triggers still follow scrolling, and scrolling within evidence cards remains available. Four regression tests pass; 309 UI tests pass across the suite and an unrestricted rerun of three API checks blocked by sandbox loopback permissions. Typecheck and targeted lint are checked separately. Browser reproduction and hosted publication are not verified; no deployment.

- 2026-10-28 owner deployment log: fresh collection completed with zero failures and built methodology0.3, then F1 rejected HyperEVM Etherscan fallback evidence at `0x68e37dE8d93d3496ae143F2E900490f6280C57cD`. The collector already emits a Hyperevmscan implementation URL, `etherscan:etherscan` marker and matching `etherscan verified source` evidence; the gate recognized only Sourcify/Git formats. F1 now accepts this existing provider format only for verified HyperEVM facts with exact URL/marker/address-bound evidence. No collector, scorer, registry or stored facts changed. Validation: 24 invariant tests, four existing HyperEVM source-verifier tests and the existing v0.3 static candidate check pass. The reported failure occurred before publication; remote cleanup/state is not independently inspected. Retry uses `deploy.ps1 -Mode Deploy`; no remote retry was performed here. Evidence: `output/deployment-verifier-fix/`.

- 2026-10-28 deployment rework: owner requested exactly two choices, `deploy.ps1 -Mode Install` (provision and deploy) and `-Mode Deploy` (complete deployment on an installed host); running without a mode presents that menu. These supersede the older application-only `-Update` and `-UpdateRpc` contracts below. Both explicit modes build the current local source, collect fresh data and score with the latest versioned methodology verified against its bundled copy. No QA dataset is packaged or automatically seeded. Owner approved archiving the original same-day snapshot and publishing a fresh revision: `snapshot-revisions/<date>-<release>/snapshot` plus per-file hashes inside the isolated candidate; older days are not rescored. Failed/partial explicit collection or mismatched output date/methodology blocks publication. Daily scheduling retains its separate same-day reuse/partial policy and current-methodology check.
- Deployment rework validation: 22 portable Node tests, PowerShell packaging/entry checks, three mocked Nginx migration/recovery scenarios and three offline daily-pipeline Rust tests pass; Bash syntax checks pass. A real offline CLI integration creates v0.2, archives it, then freshly collects/scores v0.3 for the same date with original score bytes preserved. The updated Linux publication/rollback harness is wired into CI and both remote entry paths, but exits77 on this Windows machine because native Unix symlinks are unavailable. Host installation, TLS, timers and real publication remain unverified. Details: `docs/deployment.md` and `output/deployment-rework/REPORT.md`. No commit, push or remote deployment.

- 2026-10-28 score/coverage review (spec v1.1.1): public capture still shows snapshot 2026-10-26/methodology v0.2, while the owner expected v0.3. Browser labels agree with the public API; deployed source/run/timer binding remains unverified. The application-only update path preserves stored snapshot methodology, a possible mechanism rather than a confirmed remote diagnosis. No production change was made.
- Review reproduces all 12 published Solana score objects exactly: 10 Stage 0 and two Unrated. Review-only next-level criteria contain 14 verified unmet and 13 unresolved observations; three configuration-authority cause attributions remain blocked for lack of evidence. Stage inspectors and a scoped, evidence-linked protections browser preserve published ratings. Orca/Jito require reassessment; Marinade/Solstice/Phoenix Legacy remain research-only with unresolved authority paths and publication_authorized=false. Reports: `docs/research/score-review/final-review.md`; captures, hashes and gate results: `output/reviews/score-review-2026-10-26/` (actual capture dates 27–28 October).
- Current local gate: 535 Rust tests, 305 UI tests, 142 browser passes/30 skips, lint/typecheck, trace, WASM, static build (117 pages), candidate checks and JS budget pass using `output/qa-v03-data`. All 545 protected input hashes and candidate API values remain unchanged. Owner authorized new-version text reference updates; one UI text snapshot was updated, with no screenshot/scoring golden updates. Lighthouse performance fails the unchanged 0.95 budget (Summary0.91, selected Kamino run0.80); accessibility1.00 and LCP pass. Native browser zoom, owner visual acceptance and remote publication/timer verification remain open. Ethereum admission is BLOCKED; no Ethereum implementation, commit, push, deploy or registry activation.

- 2026-10-27 branding: the headline is **DeFi Risk Intelligence**, with supporting line **Independent. Evidence-based. Transparent.** Shared strings in `site/src/lib/site.ts` supply default browser metadata, the manifest and protocol share cards. Route-specific titles and existing header/homepage/footer layouts are preserved. Explanatory control prose, historical evidence and archived reports remain unchanged. This is a presentation-only update; no commit, push or deployment.
- Branding validation: site lint, typecheck, the v0.3 candidate static build (117 generated pages) and seven existing twin/share-card tests passed. Generated default/social metadata, manifest and assessed/unassessed route titles/card links were checked. Assessed and unassessed PNGs were visually inspected; an explicit column wrapper fixes the image renderer's Stage/factor-grid clipping. Windows sandbox builds hit EPERM on Next segment directories; the build passed outside the sandbox. Hosted output and owner visual acceptance remain unverified.

- 2026-10-26 launch-audit continuation: the implementation already contained methodology v0.3 and API v2 before this continuation; the older v0.2-default and inactive-draft descriptions below are historical. Owner confirmed retaining D1–D8 and requested a verified local candidate only. Current registry entries include expanded PDA/oracle/stake-pool and HyperEVM declarations; this does not constitute human review. The owner will manage the public GitHub corrections link (H1) manually. Evidence and remaining limitations: `output/release/audit-2026-10-26/REPORT.md`.
- API release integration targets `/data/v2/` across CI, gates, packaging and Nginx. The owner-reported 2026-10-26 update failure exposed a missing migration: `-Update` now adds v2 alongside v1 only in the managed Celebtir virtual host, validates/reloads Nginx and restores the file on failure. Certificates, timers, credentials and other virtual hosts are preserved. It does not collect or rescore history. Stored v0.1/v0.2 snapshots use publication/assessment compatibility checks; v0.3 snapshots retain the complete launch invariant gate. Local migration tests use mocked service commands; live Linux update remains unverified.
- F6 catch-up retains unreported transactions from the previous snapshot when the newest history scan is bounded or its cache is cold. Original snapshot files remain immutable. Report-verifier names are sourced from existing oracle declarations and shown in dependency lists.
- The 2026-10-26 local candidate contains 16 protocols (12 assessed Solana, four unassessed HyperEVM), 29 tokens and 10 HIP-3 DEXs. The final authority run has no collection failures; transaction sampling was disabled after public-RPC throttling. Historical transaction evidence is deduplicated across preserved snapshots for upgrade summaries and backfilled timelines, with incomplete coverage and detection latency retained. Fresh reads exposed 13 Kamino Scope account mismatches, unresolved token/Jito PDAs and the existing Jupiter/GMX/HyperEVM evidence gaps. These remain explicit. GitHub/public-link work is owner-managed.
- Final local validation for that candidate: 535 Rust tests, 295 UI unit tests and 138 browser tests passed (24 documented skips); fmt, Clippy, WASM, trace, lint, typecheck, static build, JS budget, API HTTP checks and zero invariant failures. All 15 protected historical inputs retain their hashes. Two demo Windows pixel-baseline differences remain owner-review items; Linux publication harness is skip 77, and hosted checks are unexecuted. See the release report for source/input hashes and logs. No commit, push or deployment.

- 2026-10-25 responsiveness correction, awaiting owner review: desktop main content was pinned at x=208 and capped at 1680px, while the header/footer independently centered 1520px containers. The shared workspace now reserves the 208px rail once and centers a common analytical frame up to 2560px with fluid 24–48px desktop gutters; prose limits and mobile/tablet main dimensions are preserved. Evidence: `docs/design/responsiveness-review.md`, `output/responsiveness/`, and before/after gallery `output/playwright/responsiveness/index.html`. Eight requested CSS viewports, 200%/400% equivalent reflow and 150%/200% display-scaling models pass. Unit tests: 286; browser tests: 134 initially passed, four gallery timeouts passed on serial rerun (138 unique passes, 24 documented skips). Lint/typecheck/build/budget pass. Native browser chrome zoom and physical Windows display scaling remain manual checks; no commit, push or deployment.

- 2026-10-25: `deploy.ps1 -Update` is the application-only deployment path for an already provisioned host. It rebuilds the Rust CLI and static site using locked application dependencies and the latest stored snapshot, without live collection, historical rescoring, QA reseeding, credential prompts, OS/toolchain installation or Nginx/certificate/timer changes. It retains the existing publication lock/journal and exact public-release checks before pruning. Dependency installation for the new source release still runs; missing system tools require full bootstrap. Local packaging/entry tests and Bash syntax are checked; actual Linux update execution remains unverified here. Full deployment remains the default for first setup; `-UpdateRpc` cannot be combined with `-Update`.

- 2026-10-24 owner-supplied deployment log: all Solana reads failed HTTP 401; HyperEVM returned four protocols, but zero tokens caused Next static export to fail before Celebtir Nginx installation. Host state has not been independently inspected. Added `deploy.ps1 -UpdateRpc` to replace the saved Solana URL through hidden input while preserving other providers, a redacted finalized batch-read preflight before compilation, and a nonempty entity-route publication guard. The Goblin Institute response is consistent with unmatched-host default routing; actual redirect/DNS/Nginx state still needs inspection. Do not change Goblin Institute's virtual host or assume that a failed bootstrap configured Celebtir HTTPS.

- 2026-10-23 predeployment review: **NOT READY** pending execution of the strengthened Linux publication/recovery harness with native Unix symlinks. Windows Git Bash returns skip 77; syntax success is not transaction/recovery proof. Local Rust checks (488 tests, fmt, Clippy, release and WASM), trace checks, site checks (286 unit, 128 browser passed/14 skips), storage parity and protected-input checks are recorded under `output/predeployment/`. No host connection, collection, certificate request, commit or deployment occurred. See `output/predeployment/REPORT.md` for the final source/package binding and measurements.
- Local compiler output is disposable. Use only `target/review` for reviews, incremental off and dev/test debug info off; retain the release CLI under `output/predeployment/celebtir.exe`, then remove temporary compiler output. Do not recreate parallel `target/qa` caches. The 33.54 GiB original target tree was compiler output, not the runtime binary or the evidence matrix.

- Storage validation (2026-10-23): owner requested report-focused live retention. `output/storage-audit/REPORT.md` measures local `target/` at 33.54 GiB (compiler output, not collected data), `data/` at 2.63 MiB and QA v0.2 at 1.87 MiB. `scripts/storage-audit.mjs` regenerated baseline/trimmed copies: 96 byte-identical report inputs/API/feed files per dataset; original data, QA seed, registry and methodology hashes unchanged. Runtime deployment now discards raw HIP-3 transport payloads and dependency cache entries after use, keeps decoded upgrade events/scan progress needed by the UI, prunes obsolete generations/releases, and removes compiler/Next intermediates. This owner instruction supersedes unbounded deployment-copy retention and runtime raw HIP-3 retention; explicit fixture captures remain separate. Historical snapshot inputs remain required by the current history rebuilder and are not removed: replacing them with an incremental matrix is a separate persistence change. Two portable storage tests pass; live Linux deletion/publication is not verified here. Local `target/` and other worktrees were not deleted.
- First hosted test tooling (2026-10-23): `deploy.ps1` targets `jarvis@57.128.251.85` and `https://celebtir.com` at the domain root, using the existing Nginx/Certbot installation. Owner chose full deployment, one-time `output/qa-v02-data` history seed, then fresh live collection and daily 02:17 UTC updates. Helpers under `scripts/deploy/` isolate source/data/toolchains under `/opt/celebtir`, preserve Goblinz services and inactive drafts, and configure automatic HTTPS renewal. Namecheap DNS, credentials, operations and rollback are in `docs/deployment.md`. No remote deployment occurred. PowerShell dry-run and Bash syntax checks passed; Linux publication tests were added to CI/bootstrap but could not execute here because Windows disallows native symlinks (Git Bash emulates them as copies). Hosted HTTPS, renewal, collection and browser checks remain outstanding. Reported host Ubuntu 25.04 is end-of-life; OS maintenance is separate from deployment.
- The final information-first implementation contract has been executed for local review; section-by-section evidence, component equivalents, source gaps and visual deviations are in `docs/design/final-implementation-review.md`. Original logo, exact navy/gold defaults, local Inter/Cormorant, responsive research navigation, evidence drawers, grouped token exploration and typed economic-table sorting are implemented. All three Figma pages were rechecked after the owner added nine route-specific illustrations (source frames 60:2–68:2); local PNG originals/WebP derivatives are retained in `site/public/art/`.
- Native HyperEVM has a dedicated `/hyperevm/` contract-authority view. Solana Stage/factor presentation is not transferred into native HyperEVM matrix/profile/comparison views; backend scoring and published outputs are preserved. HIP-3 remains facts-only with explicit missing-coverage/halt-history states. No unpublished registry draft was activated.
- Final implementation evidence is under `output/final-implementation/`, with an 80-image review gallery at `output/playwright/final-implementation/index.html`. UI tests: 286 passed; Rust workspace: 488 passed; browser tests: 128 passed/14 documented skips; lint, build/typecheck, JS budget and trace checks passed. Final unit/browser tests used the QA v0.2 directory matching the build. Protected hashes confirm 148 original data/registry/methodology files unchanged. This is not a new full release-gate/CI result. Native browser 200% zoom, Linux pixel baselines, hosted checks and owner visual acceptance remain outstanding; the reflow test uses the equivalent CSS viewport. No commit, push or deployment occurred.
- Iteration 1 and Hyperliquid implementation slices are complete except the release/review items below. QA findings A1?A8, B1?B5, C1?C9 and D1?D12 have implementation coverage; B6 is the development-only Next badge and needs no change.
- Methodology v0.3 is the scorer default. v0.1 and v0.2 remain explicitly replayable and immutable. Original `data/` snapshots remain v0.1; `output/qa-v02-data/` remains the preserved v0.2 candidate. Use `output/qa-v03-data/` for the isolated launch-audit candidate; consult its release report before treating it as publishable.
- `output/qa-v02-data/` is an isolated v0.2 candidate rebuilt from preserved facts. `output/qa-mixed-data/` is synthetic two-chain test data and must never be published as live evidence.
- `registry/drafts/qa-v02/` is a historical review packet. Active registry entries now include launch-audit changes; do not infer human approval from their location, old cooling dates, or successful automated reads. New activation still requires explicit owner review.
- Last full QA verification: `gates: ok`; 488 Rust tests, 279 UI unit tests, 109 live browser tests passed, seven existing device/demo skips. Separate mixed-chain run: 109 passed/seven skips; four demo visual-state checks passed unchanged. These are historical results, not a claim about future edits.
- Evidence: `output/qa-baseline/verification.json`, `output/qa-baseline/gates-final.log`, input/source hashes and screenshots under `output/playwright/`.

Outstanding work, in practical order:

1. Review original Solana registry/controller ground truth (legacy 5.5/5.6), then the v0.2 PDA/oracle/token drafts. Inspect `registry/drafts/qa-v02/README.md` for unresolved evidence rather than copying drafts into production.
2. Human-review the four active HyperEVM collector entries: kinetiq-khype, sthype, morpho-blue and hyperbeat-lst. Hyperbeat's role-registry upgrade path is read; its complete proposer set, stHYPE's complete proposer set and token-owner powers remain unresolved. Decide how to support HyperLend's unsupported Aave immutable-admin proxy. Four observed protocols do not complete the five-protocol target (9.13).
3. Verify the selected `https://celebtir.com/` Nginx host deployment, DNS, HTTPS renewal and daily collection; keep the GitHub daily/Pages workflow inactive during this host test. Repository/secrets/branch protection and least-privilege workflow setup (6.5) remain separate outstanding work. Confirm outstanding design decisions with the owner. Do not assume local success means hosted CORS, paths or schedules work.
4. Devnet test bed (8.1), seven consecutive successful daily runs (8.3 and both-chain 9.16), team previews seven days before launch (8.4), launch materials (8.5), schema 1.0 baseline freeze (8.7), and Linux pixel baseline capture (8.8). Publication to crates.io remains an owner action; packaging readiness is not publication.
5. Optional per-chain split on the summary TVL chart (HUI-6/9.17) is deferred until Hyperliquid history exists. HIP-3 ratings, HyperEVM tokens, HyperEVM upgrade-history/dependency tracing and HIP-3 operating history remain outside the implemented scope. QA v0.2 does NOT introduce HIP-3 ratings, despite the original wave-2 document reserving the v0.2 name for them.

## Repository map: where to change what

| Location | Responsibility / starting point |
| --- | --- |
| `Cargo.toml`, `rust-toolchain.toml` | Rust workspace, edition 2024, Rust 1.88; unsafe code forbidden. Seven crates. |
| `crates/celebtir-core/` | Chain-neutral facts, evidence, controllers, scores and schemas. Facts require evidence. |
| `crates/celebtir-registry/` | YAML types/loading/validation; sourced claims, shared addresses, authority PDA declarations, EVM registry fields. |
| `crates/collect-solana/` | Finalized RPC reads, ProgramData, Squads v3/v4, mints/Token-2022, IDL admin/oracle decoding, transaction dependencies and metrics. Start at `src/resolve.rs` for authority resolution. |
| `crates/collect-hyperliquid/` | HyperEVM pinned-block JSON-RPC, proxies/Safe/timelocks/source verification, HyperCore info API/HIP-3. This is implemented, not a stub. |
| `crates/celebtir-score/` | Pure deterministic scoring; no I/O or clock. Versioned rules, factor reasons, Stages and value-control classes. |
| `crates/celebtir-diff/` | Typed change events, stable identities, noise suppression and source attribution. |
| `crates/celebtir-cli/src/` | CLI orchestration: `collect.rs`, `score.rs`, `site_data.rs`, `summarize.rs`, `compact.rs`, `api.rs`. Binary is `celebtir`. |
| `registry/` | Active Solana/token/oracle/controller inputs and vendored `idl/`; Hyperliquid entries and separate inactive drafts. Sources are claims to verify, not proof on their own. |
| `methodology/` | Published versioned thresholds and changelog; scorer has bundled copies in `crates/celebtir-score/methodology/`. |
| `fixtures/` | Captured account/API responses with anchors and offline pipeline goldens. |
| `schemas/` | Generated facts, registry and API schemas; breaking published API changes require a new version path. |
| `data/` | Snapshot facts/scores/events/run reports, latest outputs, histories and cache. Treat published days as immutable. |
| `site/src/app/` | Next App Router pages: summary, risk, value, activity, tokens/detail, protocol/detail, compare, changes, controllers, HIP-3, archived and references. |
| `site/src/lib/data.ts`, `types.ts` | Snapshot loader and UI data types. Follow data provenance before altering display fallbacks. |
| `site/src/lib/scope.ts`, `viewState.ts`, `useViewState.ts` | Shared chain selection, scoped entities/history, URL state. |
| `site/src/lib/summary.ts`, `overview.ts`, `calendar.ts` | Aggregates, coverage and complete-day/week activity calculations. |
| `site/src/lib/controller.ts`, `graph.ts`, `permissions.ts`, `protocol.ts` | Controller presentation, graph identity, powers and protocol view data. |
| `site/src/components/scope/`, `shell/`, `page/` | Scoped links/sections/empty states, header/navigation and page layouts. |
| `site/src/components/ReasonText.tsx`, `LimitedItems.tsx`, `HistoryCoverage.tsx` | Evidence prose, expandable long lists/deep links and honest history labels. |
| `site/src/components/ui/`, `RatingBadge.tsx`, `site/src/lib/tokens.ts`, `site/src/app/globals.css` | Shared visual primitives, rating semantics and design tokens. |
| `site/scripts/` | Data copy, static postbuild artifacts/Windows route aliases, local export server, JS budget and hosted API checks. |
| `site/e2e/`, colocated `*.test.ts(x)` | Playwright route/interaction/visual/axe checks and Vitest unit/component coverage. |
| `scripts/`, `.github/workflows/` | Daily pipeline, gates, traceability, captures, CI and deployment automation. |
| `docs/design/` | Product design specifications. |
| `output/` | Local candidates, audit captures and logs; disposable build output is not authoritative source data. |

## Data flow and invariants

Registry + captured/read-only chain/API evidence -> collectors -> facts -> versioned scorer -> scores -> diff/events -> site-data/history/static API -> Next static export (`site/out`). `scripts/daily.sh` orchestrates validation, collection, scoring, diff, compaction and site data; workflow then builds/deploys. It refuses an existing snapshot date, works in a candidate directory and only publishes a collected/scored candidate. Replays belong in a separate data directory.

Evidence includes chain, address, field and slot/block/source. Solana uses finalized reads; HyperEVM pins a block, and HIP-3 uses its documented HyperEVM anchor without pretending it is a Solana slot. Failed reads/decodes become Unavailable/Unrecognized, never fabricated defaults. RPC URLs are reduced to hosts; do not print secrets or read `.env` merely for orientation. Tests replay offline; public network captures are separate explicit evidence operations.

JSON output is deterministic: stable sorting and no incidental timestamps outside run metadata. Preserve published facts, scores and methodology. Event corrections append references instead of deleting history. Distinguish on-chain, registry and methodology events; dependency changes need three consecutive observations and verification changes two. Backfill metrics are not Celebtir snapshots. Never reconstruct historical control classes with today's thresholds when a published score class exists.

## Methodology and controller decisions

Six protocol factors: upgrade control, upgrade delay, admin powers, oracle dependency, code transparency and external dependencies. Weakest core-program control determines the relevant protocol factor; periphery is displayed separately. Token scoring has four factors and no Stage. Stages describe control arrangements, not code safety. Unknown is distinct from High; Unknown upgrade control makes a protocol Unrated, other Unknown factors cap it at Stage 0. N/A and unavailable also have separate UI states.

Read the selected version's TOML and scorer rules for exact behavior. v0.2 thresholds include Low control at >=5 voters/60%, Medium at >=3/50%; delay Low at five days and Medium at 24 hours. Count voting permissions, not all multisig members. Config authorities/modules can bypass signatures or delays and must be resolved. Timelocks and Safe controllers are supported; bounded recursion and loop detection prevent claiming control that cannot be verified.

Owner-approved QA decisions supersede older proposals:

- `Controller::ProgramControlled { key, program, upgrade }` is supported through typed registry `authority_pdas` declarations. Require a source, exact seed/address derivation, an executable controlling program and bounded resolution of its upgrade authority. A no-data PDA is not proven merely by its apparent System Program ownership. Missing/mismatched/conflicting declarations remain conservative and flagged. v0.2 rates the program's upgrade controller; v0.1 replay behavior stays unchanged.
- Lending/perps without listed oracle feeds are Unknown, not N/A. Oracle roles must be evidenced, not inferred from account names.
- Delayed-control value attribution requires at least the methodology's Medium delay minimum (24 hours in v0.2). GMX's 600 seconds is not classified as protective delay. v0.1 keeps its old rule.
- Generated v0.2 risk copy describes configuration powers, avoiding unsupported theft/withdrawal assertions. Every elevated token factor has a reason. Old stored v0.1 strings remain untouched; presentation can normalize legacy copy.
- Published methodology files are immutable. A new rule requires a new version, bundled copy, changelog, tests and explicit historical replay verification.

The v0.2 registry packet contains 18 derived mint-authority declarations (SPL/Sanctum, Marinade, Wormhole); 39 Jupiter Lend oracle accounts; 78 GMX token configurations; five Jupiter Perps custody/Doves mappings; and a cbBTC wrapped/bridged-category proposal. Derivations and executable programs were checked against fresh captures, but Lend vault relationships, GMX provider semantics and Doves roles still need review. Sanctum's fwdSOL mint now named dumSOL is separate identity drift. These drafts do not change current ratings until reviewed and activated.

## UI behavior to preserve

Next.js 16.4.2 + React 19 + TypeScript static export. Read `site/AGENTS.md` and relevant installed `site/node_modules/next/dist/docs/` before Next API changes. Cache Components are off. The site uses React Flow for graphs and d3 utilities for charts; old spec library suggestions are not mandates to add unused dependencies.

- All/Solana/Hyperliquid scope applies to counts, KPIs, charts, histories, search, tokens, events and shared controllers through common selectors. Controller identity includes chain. Token provenance is evidence-based, never guessed from address shape. Entity pages fix the picker to their own chain.
- Internal links preserve selected chain, queries and fragments; default All omits `chain=all`. Empty chain coverage differs from an empty user filter, and reset preserves chain. Use shared scoped links rather than local ad hoc filtering.
- Activity KPIs and rows use the same completed date per metric, strictly before snapshot date. Weekly change compares two complete seven-day calendar totals; missing observations produce unavailable values. LST trading volume is N/A. History labels separate DefiLlama observations from actual snapshots.
- Compare shows selection chips immediately, suggests same-category peers and enables differences only with two selections. Long token/graph lists initially show ten; fragment navigation reveals and focuses the actual target.
- Reasons render code spans, concise evidence links and grouped repeated program reasons. Freeze authority can be Renounced. Shared-controller powers name the token/issuer. API docs link real generated schema files.
- Ratings use `RatingBadge` (text + icon); shared design tokens, not raw component colors. Preserve dark theme, keyboard operation, mobile layout at 375 px, readable search, nonoverlapping headers and viewport-aware tooltips.
- Static navigation uses no speculative prefetch and postbuild Windows segment aliases. Test actual scoped navigation/network responses, not only direct page loads. HIP-3 stays unrated and explains publication timing without inventing a launch date.

## Run locally and validate

PowerShell from the repository root, using the preserved QA candidate:

```powershell
$env:CELEBTIR_DATA = Join-Path (Get-Location) 'output/qa-v02-data'
pnpm.cmd -C site build
Set-Location site
node scripts/serve-out.mjs 3100
```

Open http://localhost:3100. Serving an existing export needs only the last two commands from the repository root; the server resolves `out/` from its working directory. For development, from the repository root run `pnpm.cmd -C site dev` (normally port 3000). Both build and dev run the data-copy hook: unset `CELEBTIR_DATA` uses original `data/`, not the candidate. Never infer the displayed methodology from the source-code default alone. `CELEBTIR_SITE_URL` controls absolute site metadata; hosting/base-path decisions are still pending.

Full local gate: `bash scripts/gates.sh`. On Windows use Git Bash, not WSL bash. Rust pin is 1.88; the verified installed `stable` was also 1.88, but check before substituting it. Useful resource settings from the QA run:

```powershell
$env:PATH = 'C:\Program Files\Git\bin;C:\Program Files\Git\usr\bin;' + $env:PATH
$env:CARGO_BUILD_JOBS = '1'
$env:CARGO_TARGET_DIR = 'target/review'
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
bash scripts/gates.sh
```

Gates run Rust fmt/clippy/tests, requirement trace + its regression test, wasm32 scorer build, site lint/typecheck/unit tests/build, generated artifact checks, JS budget and Playwright. CI additionally covers Lighthouse, coverage, dependency/license checks and release/MSRV checks. Choose relevant checks for an iteration, then run required gates before a commit; never weaken assertions or regenerate historical goldens just to get green.

Quick commands: `bash scripts/trace.sh`; `bash scripts/tests/trace_test.sh`; `cargo test --workspace`; `pnpm.cmd -C site test`; `pnpm.cmd -C site e2e`. CLI entry points: `cargo run -p celebtir -- --help`; the binary interface is `celebtir validate|collect|score|diff|build-site-data|explain` (see `crates/celebtir-cli/src/main.rs` for all flags/commands).

Browser workers are bounded at four. Axe runs in-page to avoid a reproduced multi-megabyte adapter JSON-transfer fault, and asserts a single frame so future embedded content cannot silently evade testing. Keep all WCAG/zero-violation checks. Tests such as `qa-regressions.test.ts(x)`, `qa-routes.spec.ts`, Rust `qa_v02.rs` and `qa_program_control.rs` cover the QA decisions. Historical input hashes and v0.1 byte-identical replay are especially important when touching scorer or publication code.

## Maintaining this context

For a new iteration: identify the affected layer in the map, inspect current evidence and decisions in this guide, implement with relevant regression coverage, record checks and limitations, and update current state/open work here. Keep detailed captures/reports under `docs/` or `output/`; do not create a competing root spec/backlog/progress document. Licensing files remain independent legal documents (code MIT/Apache-2.0; data CC BY 4.0).

The appendices retain original requirement IDs and completion ownership for `scripts/trace.sh`. They are historical acceptance contracts, not a fresh execution queue. A checked implementation slice does not imply owner review, production activation or deployment. An asterisk marks manual/CI verification; unstarred completed Must IDs require a matching test name (`SCO-3` -> `sco_3_...`). Update the register and ledger together when adding requirements.

## Requirement register

Original requirement text is retained to preserve acceptance detail. Current overrides above apply, especially v0.2 oracle/PDA/delay rules, factual risk copy, actual UI stack, HIP-3 remaining unrated and no automatic commits.

| ID | Requirement | Acceptance test | Priority |
| --- | --- | --- | --- |
| REG-1 | A JSON Schema defines the YAML format; `celebtir validate` checks every file against it. | A file with a missing `source` or an unknown `category` fails validation with the file and field named. | Must |
| REG-2 | Every address is valid base58 and 32 bytes; every program ID resolves to an executable account on mainnet. | A typo'd or closed program ID fails validation. | Must |
| REG-3 | Every program, multisig, admin account and oracle carries a `source` URL. | Validation rejects an entry without one. | Must |
| REG-4 | An address may appear in two protocols only if both list it under `shared_with`. | A duplicate without `shared_with` fails validation. | Should |
| REG-5 | Each protocol needs at least one `core` program; only core programs decide the Stage. | A protocol with only periphery programs fails validation. | Must |
| REG-6 | Registry changes arrive by pull request; CI runs `validate` and the author may merge only after a 24-hour cooling period, noted in the PR. | A required check blocks merging until 24 hours after the last commit. | Must |
| REG-7 | A file whose `last_reviewed` is older than 90 days shows a "registry review due" note on its protocol page. | Setting an old date surfaces the note after the next build. | Should |
| REG-8 | Vendored IDLs live in `registry/idl/` with the commit or slot they came from. | Each admin account's `idl` path exists and records its origin. | Must |
| COL-1 | One RPC client wraps every call with a token-bucket rate limit (default 8 requests per second, under Helius's 10) and a concurrency cap. | A load test of 1,000 calls never exceeds the configured rate. | Must |
| COL-2 | Retries use exponential backoff with jitter on 429, 5xx and timeouts, up to 5 attempts; other errors fail only the affected protocol. | A mocked 429 succeeds on retry; a mocked 400 marks one protocol failed and the run continues. | Must |
| COL-3 | All reads use `finalized` commitment and record the response slot; each protocol stores the min and max slot of its reads. | Every fact in a snapshot carries a slot. | Must |
| COL-4 | Accounts are read in batches with `getMultipleAccounts` (up to 100 per call). | One protocol with 10 accounts costs one call. | Must |
| COL-5 | Each run logs calls per method and stops before a daily budget (default 5,000 calls). | Setting the budget to 10 halts the run cleanly with a budget error. | Must |
| COL-6 | Programs owned by the upgradeable loader: read the program account, then its ProgramData with a data slice of the first 45 bytes, so program bytecode is never downloaded. | Decoding a fixture yields the last-deploy slot and the authority, or none. | Must |
| COL-7 | Programs owned by the older non-upgradeable loaders are recorded as immutable; other owners (loader v4, native) are recorded as "unsupported loader" rather than guessed. | Fixtures for each owner produce the right label. | Must |
| COL-8 | Controller resolution: match the authority against the Squads vault derived from each registry multisig; else check whether the key is on the ed25519 curve (a wallet) or off it (a program-derived address); else look up its owner in `controllers.yaml`. | Fixtures for a wallet, a Squads vault and an unknown PDA resolve to single key, multisig and Unrecognized. | Must |
| COL-9 | Squads v4 decoding checks the owner program and account discriminator, then reads threshold, time lock, config authority and members with their permissions; only members with the Vote permission count as signers. | A fixture with 5 members, 1 without Vote, reports 4 voters. | Must |
| COL-10 | A non-default config authority is resolved like any controller, up to 3 levels deep, with loops detected. | A multisig controlled by another multisig resolves both; a loop stops with an error. | Must |
| COL-11 | Admin accounts are fetched, their owner checked against the protocol's programs, and the named field decoded with the vendored IDL. | A wrong owner or a failed decode records "unavailable", never a default key. | Must |
| COL-12 | Oracle accounts: read the owner program and classify it with `oracles.yaml`; unknown owners are labelled "other". | A Pyth-owned fixture classifies as Pyth. | Should |
| COL-13 | Verified builds: query the OtterSec status endpoint per core program, store verified flag, both hashes, repo and commit, and flag a repo outside the registry's GitHub org. | A repo from another org shows a mismatch warning. | Must |
| COL-14 | Dependencies: sample the last 50 finalized transactions per core program, resolve lookup-table addresses, and use instruction stack heights to list programs the protocol calls directly; drop system, token, compute-budget and memo programs. | A fixture transaction with a nested call lists only the direct callee. | Should |
| COL-15 | One protocol's failure never stops the run; the run report lists every failure with its cause. | Breaking one registry entry still yields 11 scored protocols and a report. | Must |
| TOK-1 | `registry/tokens.yaml` lists each token's mint, issuer, type, provenance (native or bridge plus backing chain and address) and a source per claim. | Validation rejects a bridged token without a bridge and backing address. | Must |
| TOK-2 | Each protocol's registry file lists the tokens it uses and their role (collateral, borrowable, deposit, LP). | A token used by no rated protocol fails validation as out of scope. | Must |
| TOK-3 | The collector reads every mint in one batched call, records the owner program (classic SPL Token or Token-2022), supply, decimals, mint authority and freeze authority. | A fixture mint with both authorities set reports both. | Must |
| TOK-4 | For Token-2022 mints, every extension is parsed; unknown extensions are listed by type number instead of ignored. | A fixture with permanent delegate and transfer hook reports both, with their authorities. | Must |
| TOK-5 | Mint, freeze, delegate and hook authorities are resolved through the same controller logic as program authorities. | A freeze authority held by a Squads vault shows the multisig's threshold. | Must |
| TOK-6 | Tokens get Low / Medium / High ratings on four factors: supply control, freeze and seize powers, transfer restrictions, provenance. No Stage. | Every token page shows four ratings with evidence. | Must |
| TOK-7 | A protocol's page lists the tokens it depends on with their worst rating, and flags protocols that accept a High-rated token as collateral. | A protocol accepting a token with an active permanent delegate shows a warning. | Should |
| TOK-8 | Token changes (new authority, extension enabled, authority renounced) appear as change-log events. | Changing a fixture's freeze authority between two snapshots yields an event. | Should |
| TOK-9 | Supply and price history for each token from a free source, for the Value section's charts. | Each token page shows a 30-day supply line once 30 snapshots exist. | Could |
| VAL-1 | Daily TVL per protocol from DefiLlama, stored in each snapshot; history accumulates from the first run. | 7 runs produce a 7-point TVL series per protocol. | Must |
| VAL-2 | Value by control type: TVL summed per controller class, per day, for all protocols and per category. | The five classes sum to total TVL for every day. | Must |
| VAL-3 | TVL split by asset group (stablecoins, SOL and liquid staking tokens, BTC, other) where DefiLlama's per-token breakdown allows. | A lending protocol shows four asset groups summing to its TVL. | Should |
| VAL-4 | Upgrade history: list every transaction on each ProgramData account, classify each as deploy, upgrade, set authority, extend or close, and convert slots to dates. | A program upgraded 3 times shows 3 dated upgrades and its authority history. | Must |
| VAL-5 | Upgrade metrics per protocol: count, last upgrade, average interval, upgrades in the last 30 days. | Values match a hand count for one fixture protocol. | Must |
| VAL-6 | Usage activity: daily volume and fees per protocol from DefiLlama, with 7-day change; protocols without a volume metric show fees only. | Every protocol shows at least fees for the last 30 days. | Should |
| VAL-7 | A failed TVL or activity fetch hides that number with a "data unavailable" state; it never fails the run or changes a rating. | A mocked DefiLlama outage still publishes ratings. | Must |
| VAL-8 | Summary-level charts: total TVL rated, value by control type, and Stage distribution over time. | Charts render from the accumulated daily series. | Should |
| SNAP-1 | Every fact carries at least one Evidence item; a fact without evidence fails serialization. | Unit test: building a fact without evidence does not compile or returns an error. | Must |
| SNAP-2 | Output is deterministic: keys sorted, arrays sorted by stable IDs, no timestamps outside `run.json`. | Two runs over identical inputs produce byte-identical `facts.json`. | Must |
| SNAP-3 | Each file declares `schema_version`; a published JSON Schema validates every file in CI. | A field removed without a major version bump fails CI. | Must |
| SNAP-4 | `celebtir score --facts <file>` reproduces the published `scores.json` for any past day. | Re-scoring last week's facts matches last week's scores byte for byte. | Must |
| SNAP-5 | Secrets never appear in any output: RPC URLs are reduced to their host. | A grep for the API key across `data/` finds nothing. | Must |
| SNAP-6 | A day's snapshot stays under 2 MB; history files under 500 KB each. | CI fails if a file exceeds its size budget. | Must |
| SNAP-7 | Snapshots older than 180 days are compacted to weekly, keeping daily `events.json`. | Repository size growth stays under 100 MB a year. | Should |
| SCO-1 | `score(facts, methodology) -> Scores` has no I/O and no clock. | The crate builds with network and filesystem access denied; tests run offline. | Must |
| SCO-2 | All thresholds (signer counts, ratios, delays) live in `methodology/v0.1.toml`; none are hard-coded. | Changing the 5-day delay to 7 days in the file changes ratings without a code change. | Must |
| SCO-3 | Factor rules follow the PRD tables exactly; signer ratios use voters, not total members. | One test per table cell (6 factors × 3 levels) passes. | Must |
| SCO-4 | Weakest link across core programs decides each protocol factor; periphery programs are shown but not counted. | A protocol with one single-key periphery program keeps its Low rating. | Must |
| SCO-5 | Every rating returns a one-line reason, its evidence references, and what would raise it one level. | No rating in `scores.json` has an empty reason or next step. | Must |
| SCO-6 | The Stage decision returns the Stage plus the list of unmet requirements for the next one, as L2Beat shows "items missing for Stage N". | A Stage 0 protocol lists exactly which Stage 1 requirements fail. | Must |
| SCO-7 | Unavailable facts produce an Unknown rating, shown separately from High; Unknown upgrade control makes the protocol Unrated; any other Unknown caps the Stage at 0. | A fixture with an undecodable admin account shows Unknown, not High. | Must |
| SCO-8 | Property tests: adding a voter, raising a time lock, or removing an admin key never lowers a rating; scoring is deterministic. | `proptest` suites pass with 10,000 cases each. | Must |
| SCO-9 | Risk sentences in L2Beat's style are generated from facts with a severity: "Funds can be stolen if…", "Funds can be frozen if…", "Prices can be manipulated if…", each linked to its factor. | Every High rating produces at least one sentence; Low ratings produce none. | Must |
| SCO-10 | Tokens use the same engine with their four factors and no Stage. | Token scores appear in `scores.json` under their mint address. | Must |
| SCO-11 | The scorer compiles to WebAssembly, so iteration 2 can run it in the browser for the methodology playground. | `cargo build --target wasm32-unknown-unknown` succeeds in CI. | Should |
| CHG-1 | Events carry before and after values, evidence, and the transaction signature when the ProgramData history provides one. | An upgrade event links to its transaction. | Must |
| CHG-2 | Each event is tagged by source: on-chain, registry edit or methodology. Protocol "Updates" sections show on-chain events by default, with a toggle for the rest. | A registry fix never appears as an on-chain change. | Must |
| CHG-3 | Noise control: dependency events need 3 consecutive runs; verification changes need 2. | A dependency seen in one sample only produces no event. | Must |
| CHG-4 | Events have stable IDs and are never deleted; corrections are new events that reference the old one. | Rewriting history fails a CI check on `events.json` files. | Must |
| CHG-5 | After missed runs, the diff uses the last good snapshot and marks events "detected within N days". | Skipping two days yields events with a 3-day detection window. | Should |
| CHG-6 | An Atom feed of all events, and one per protocol, so people can subscribe without an alert system. | The feed validates and lists the latest 50 events. | Could |
| CLI-1 | The RPC URL comes from `CELEBTIR_RPC_URL`, falling back to the public endpoint with a warning. | Unset variable prints a warning and still runs. | Must |
| CLI-2 | Exit codes: 0 success, 1 validation error, 2 partial failure, 3 budget exceeded. | Each case returns its code in tests. | Must |
| CLI-3 | `--format json` on every command for scripting; text by default. | `explain --format json` validates against the scores schema. | Must |
| CLI-4 | `--offline` replays recorded responses from a fixtures directory. | The full pipeline runs in CI without network. | Should |
| CLI-5 | Published to crates.io with install instructions; `--version` prints the commit. | `cargo install celebtir` works on a clean machine. | Should |
| UI-1 | Every view's state (tab, filters, sort, columns, compare selection) lives in the URL, so any view can be shared as a link. | Pasting a copied URL into a fresh browser reproduces the exact view. | Must |
| UI-2 | A freshness bar on every page shows the snapshot date, slot and methodology version; data older than 36 hours shows a warning. | Serving a 2-day-old snapshot shows the warning. | Must |
| UI-3 | Keyboard shortcuts: `/` search, `F` filters, `C` compare tray, `?` shortcut list. | Each shortcut works on every page with a table. | Must |
| UI-4 | A compare tray: tick any row in any table to add it (up to 4); the tray persists across pages until cleared. | Adding protocols from Summary and Risk lands both in one compare view. | Must |
| UI-5 | Evidence links open Solscan for accounts and transactions; every address shows shortened with copy-to-clipboard and its full value on hover. | Every evidence link resolves to the right Solscan page. | Must |
| UI-6 | Glossary terms (upgrade authority, time lock, config authority, permanent delegate…) carry an inline definition on hover or tap. | Every term in the glossary is linked at its first use per page. | Should |
| UI-7 | Unknown, Unrated, Not applicable and Data unavailable each have one consistent visual and wording everywhere. | A visual regression test covers all four states. | Must |
| UI-8 | Above the table: total TVL rated, value by control type and Stage distribution, each over time with 7-day, 30-day and all-time ranges. | Charts render once 2+ snapshots exist and switch ranges without reloading. | Must |
| UI-9 | Category tabs (All, Lending, Spot DEX, Perps, Liquid staking) with counts and a one-line definition each; a chain filter shows Solana now and Hyperliquid in wave 2. | Counts match the registry. | Must |
| UI-10 | Table columns: rank, name and logo, category, risk rosette (six factors, tooltip with each rating), Stage (tooltip lists what is missing for the next Stage and notes that Stages don't measure code security), controller summary ("Squads 3 of 5 · no delay"), TVL with 7-day change, 24-hour volume or fees with 7-day change, last on-chain change. | Every column sorts; tooltips work by keyboard. | Must |
| UI-11 | A column picker and filters by Stage, factor rating and controller class. | Hiding a column and filtering to "single key" survives a reload via the URL. | Should |
| UI-12 | All 12 sections render for every protocol; a section with nothing to show says why ("no admin keys found") instead of disappearing. | Snapshot tests for each section's empty state. | Must |
| UI-13 | The Permissions section groups by actor, lists each power (upgrade, change parameters, pause, move funds, freeze tokens) and its delay, and links to the actor's node in the graph. | Clicking an actor focuses it in the control graph. | Must |
| UI-14 | Each protocol page has a Markdown twin at `/protocols/<id>.md`, and the site serves `llms.txt`. | Both are generated at build and linked from the page footer. | Must |
| UI-15 | A per-protocol Open Graph image shows name, Stage and the six factor ratings, for sharing on X. | Pasting a protocol URL into a link previewer shows its card. | Should |
| UI-16 | Choose 2 to 4 protocols by search or from the compare tray; rows cover Stage, six factors with reasons, controllers, upgrade metrics, TVL, activity and token exposure. | A comparison of 4 protocols fits a 1,440 px screen without horizontal scroll. | Must |
| UI-17 | A "show differences only" toggle hides rows where all protocols match. | Comparing two identical setups with the toggle on shows an empty-state message. | Must |
| UI-18 | Overlay charts: TVL and activity of the compared protocols on one chart. | Lines carry protocol names, not colour alone. | Should |
| UI-19 | Nodes: protocol, programs, admin accounts, tokens, multisigs, keys, config authorities. Edges are labelled with the power they carry ("can upgrade", "can pause", "can freeze") and their delay. | Kamino's graph shows every controller in its registry with labelled edges. | Must |
| UI-20 | Clicking a node opens a side panel with its facts, evidence and Solscan link; multisig members start collapsed as a count and expand on click. | A 9-member multisig shows "9 signers" until expanded. | Must |
| UI-21 | Node colour follows the node's rating, always paired with a text label or icon. | The graph passes a colour-blindness simulation check. | Must |
| UI-22 | A "shared controllers" view highlights keys or multisigs that control more than one protocol, across the whole ecosystem. | A key controlling two protocols appears once, linked to both. | Should |
| UI-23 | Every graph has a list view with the same content, for keyboard and screen-reader users. | The list view passes an axe accessibility scan. | Must |
| UI-24 | Dark only: near-black background, a monospace face for addresses and numbers, a sans face for prose; Low / Medium / High always carry text and an icon, never colour alone; WCAG AA contrast throughout. | Automated contrast checks pass on every page. | Must |
| UI-25 | Desktop first at 1,440 px; usable at 375 px, where tables collapse to cards and compare shows 2 protocols at a time. | Every page passes manual checks at both widths. | Must |
| UI-26 | React + TypeScript built as a static export (each route pre-rendered as HTML for search engines and link previews); TanStack Table for tables; React Flow for the graph; a charting library chosen in the Foundations phase. | The build outputs one HTML file per route and deploys to GitHub Pages. | Should |
| UI-27 | Performance: under 250 KB of compressed JavaScript on the Summary page; graph and chart code load only where used; largest contentful paint under 2.5 seconds on a mid-range laptop. | Lighthouse CI enforces the budgets on every pull request. | Could |
| API-1 | Static JSON at `/data/v1/summary.json`, `/data/v1/protocols/<id>.json`, `/data/v1/tokens/<mint>.json` and `/data/v1/events.json`, each linked to its JSON Schema. | Every file validates against its schema in CI. | Must |
| API-2 | Breaking changes go to `/v2/` with `/v1/` kept for 90 days and a deprecation note in the files. | A schema-breaking PR without a new version path fails CI. | Should |
| API-3 | Files are readable cross-origin from any website. | A fetch from another origin's test page succeeds. | Must |
| API-4 | An SVG badge per protocol (`/badges/<id>.svg`) shows its Stage, with a copyable embed snippet on the protocol page. | The badge updates on the day the Stage changes. | Should |
| API-5 | Every data file states its licence and attribution requirement. | The licence field is present in all files. | Must |
| OPS-1 | The workflow runs daily at 02:17 UTC (an off-peak minute) and can be started manually. | A manual run produces the same outputs as a scheduled one. | Must |
| OPS-2 | Steps: validate → collect → score → diff → build site data → commit data → build site → deploy. Each step uploads its outputs as artifacts. | A failure at any step leaves the deployed site unchanged. | Must |
| OPS-3 | Secrets (the RPC URL) live in repository secrets; the workflow token has only content-write and pages-write permissions. | A permissions audit shows no broader scopes. | Must |
| OPS-4 | On failure, the workflow opens or updates one issue labelled `pipeline-failure` with the run report. | Breaking the RPC URL creates the issue; the next good run closes it. | Must |
| OPS-5 | Data commits come from a bot identity, one per day, with a message summarising events and failures. | The git log shows one bot commit per day. | Should |
| OPS-6 | Each run reports projected monthly usage per external service and opens an issue above 70% of a free tier. | Raising call volume in a test triggers the warning. | Should |
| OPS-7 | GitHub disables scheduled workflows in public repos after 60 days without repository activity; the daily data commit keeps it active, and a weekly check confirms the schedule is still enabled. | The check fails loudly if the schedule is disabled. | Must |
| QA-1 | Decoder fixtures: real account bytes (ProgramData, Squads multisigs, classic and Token-2022 mints with extensions) captured with their slot and committed to `fixtures/`. | Every decoder has at least one fixture per variant it handles. | Must |
| QA-2 | A devnet test bed: one tiny program deployed in five configurations (immutable, single key, Squads without time lock, Squads with time lock, multisig with a config authority) plus a Token-2022 mint with a permanent delegate and a transfer hook. A weekly CI job runs the full pipeline against them. | Each configuration yields its expected rating; devnet costs nothing. | Must |
| QA-3 | Golden tests: the whole pipeline replays recorded responses offline and compares outputs to committed golden files. | Any unintended output change fails CI with a readable diff. | Must |
| QA-4 | Frontend: component tests, end-to-end tests for share-a-filtered-view, compare and graph node click, visual regression for the four data states, and automated accessibility scans. | All run on every pull request. | Must |
| QA-5 | CI gates: fmt, clippy with warnings as errors, tests, minimum Rust version, dependency licence and advisory checks, schema validation, Lighthouse budgets. | A PR failing any gate cannot merge. | Must |
| QA-6 | Ground-truth audit before launch: every protocol's controllers are re-checked against Solscan in a fresh session at least 24 hours after the registry entry was written, then signed off in a checklist; protocol teams' preview feedback is the second pair of eyes. | 12 of 12 protocols signed off. | Must |
| QA-7 | Line coverage at 90%+ for the scorer and 70%+ for collectors. | Coverage report enforced in CI. | Should |
| LCH-1 | 7 days before first publication, each protocol team gets an unlisted preview link and dispute instructions; contacts and responses are logged. | 12 of 12 teams contacted, with dates recorded. | Must |
| LCH-2 | Every page footer and the About page state: not investment advice, ratings describe on-chain configuration only, they may contain errors, and how to report one. | Present on every route. | Must |
| LCH-3 | Generated sentences are factual; a lint blocks loaded words ("scam", "unsafe", "rug") in templates. | Adding a banned word to a template fails CI. | Should |
| LCH-4 | Corrections policy: disputes acknowledged within 72 hours, resolved within 7 days, logged publicly. | The policy is published on the About page. | Must |
| LCH-5 | Licences: code dual-licensed MIT or Apache-2.0 (the Rust convention); data under CC BY 4.0. | Licence files exist and every data file references its licence. | Must |
| LCH-6 | No cookies and no personal data; any analytics must be cookieless. | A cookie scan of the site finds none. | Must |
| LCH-7 | Launch kit: a thread built on one headline finding with two charts, a 60-second recording of `explain` and the control graph, a README with an animated demo, and a write-up post. | All four are ready before launch day. | Must |
| HREG-1 | The registry schema accepts `chain: hyper_evm`. Contracts list `address`, `role` (core/periphery), `proxy` (`eip1967` / `uups` / `none`) and a `source` per claim. Addresses must be EIP-55 checksummed. | A lower-case or bad-checksum address fails `validate` with file and field named. | Must |
| HREG-2 | Timelock entries name their proposers. Safe entries need no member list, because members are read on-chain. | A timelock without proposers fails validation. | Must |
| HREG-3 | `controllers.yaml` and `oracles.yaml` gain EVM sections: Safe singletons (v1.3.0, v1.4.1, L2 variants), TimelockController, Pyth, RedStone, Chainlink, and the HyperCore oracle precompile. | Validation rejects a duplicate address across sections. | Must |
| HREG-4 | `hip3.yaml` keys are DEX names. An unknown name is a warning, not an error, because DEXs come and go. | A `hip3.yaml` entry for a DEX not in the API response warns and the run continues. | Should |
| HREG-5 | Five HyperEVM protocols are drafted with their selection evidence and date recorded. Each file is reviewed by a human before it is published. | The review lists 5 protocols with DefiLlama TVL and date. | Must |
| HCOL-1 | One JSON-RPC client with a token-bucket rate limit, retries with backoff on 429/5xx/timeouts (as COL-1 and COL-2), and a daily call budget (as COL-5). The URL comes from `CELEBTIR_HYPEREVM_RPC_URL`, falling back to the public RPC with a warning, and is reduced to its host in output. | A mocked 429 succeeds on retry; an unset variable warns and runs; no URL path appears in `run.json`. | Must |
| HCOL-2 | All reads at one pinned block (`eth_blockNumber` once per run, then `eth_call`/`eth_getStorageAt` at that block); every fact records it as `slot`. | Every HyperEVM fact in a snapshot carries the run's block. | Must |
| HCOL-3 | Proxy reads: the EIP-1967 implementation slot and admin slot. An admin that is a ProxyAdmin resolves through `owner()`. `proxy: uups` with an empty admin slot resolves through the implementation's `owner()`. `proxy: none` with no code-changing path is `Immutable`. | Fixtures for a transparent proxy with ProxyAdmin, a UUPS proxy and a plain contract give the right controller. | Must |
| HCOL-4 | Controller classification, in order: code size 0 → `SingleKey`; a known Safe singleton and `getThreshold`/`getOwners` → `Multisig { kind: safe }`; a listed timelock and `getMinDelay` → `Timelock`; otherwise `Unrecognized` with the code hash. Depth 3, loops detected. | Fixtures for an EOA, a Safe 3 of 5, a timelock and an unknown contract resolve to the four variants. | Must |
| HCOL-5 | Safe decoding: threshold, owners (sorted), and modules (`getModulesPaginated`). An enabled module can execute without signatures, so it is resolved as a config authority. | A Safe fixture with one module reports the module as config authority. | Must |
| HCOL-6 | Timelock decoding: `getMinDelay()`; each registry proposer confirmed with `hasRole(PROPOSER_ROLE, p)`; an outside `DEFAULT_ADMIN_ROLE` holder is treated as a config authority with no delay (edge case 4). | An unconfirmed proposer is `Unavailable`; an outside admin makes the effective delay 0. | Must |
| HCOL-7 | Admin powers: the `owner()` or role holders named in the registry per contract are read and resolved like any controller. A failed call is `Unavailable`, never a default. | A reverting `owner()` records unavailable. | Must |
| HCOL-8 | Verified source: Sourcify first (full or partial match), then the Etherscan v2 API (`chainid=999`) when `CELEBTIR_ETHERSCAN_KEY` is set; neither → Unknown. The key never appears in output. | A mocked Sourcify match marks verified; a grep for the key across `data/` finds nothing. | Must |
| HCOL-9 | Oracle contracts declared in the registry are classified by `oracles.yaml`; unknown addresses are labelled "other". | A Pyth address classifies as Pyth. | Should |
| HCOL-10 | Dependencies come only from the registry in wave 2. The fact says "declared, not traced". | The Dependencies reason names its source as the registry. | Should |
| HCOL-11 | One protocol's failure never stops the run; on either chain, a chain-wide failure (RPC down) fails only that chain's protocols. | With the HyperEVM RPC mocked down, all Solana protocols still score. | Must |
| HIP-1 | One `POST /info {"type":"perpDexs"}` per run. Each non-null entry becomes a DEX: name, full name, deployer, oracle updater (null = not delegated), fee recipient, per-asset OI caps and funding multipliers. Unknown fields are kept in the raw response, not dropped. | A captured response with N DEXs yields N DEX fact sets; the first (null) entry, the main Hyperliquid DEX, is skipped. | Must |
| HIP-2 | The raw response is stored per snapshot (`data/snapshots/<date>/raw/perpDexs.json`), with sorted keys, inside the SNAP-6 budget. | Two runs over the same response write byte-identical files. | Must |
| HIP-3 | Every HIP-3 fact carries evidence: chain `hyper_core`, the address, the HyperEVM block anchor, `field` naming the JSON path, and a Hyperliquid explorer URL. | No HIP-3 fact lacks evidence (SNAP-1 holds). | Must |
| HIP-4 | Deployer stake: the info API's staking data for the deployer is read and compared with the required stake from the HIP-3 docs, stored in the registry with its source. Unstaking in progress is flagged. The exact endpoint is confirmed and recorded in the fixture capture script. | A fixture with stake below the requirement sets a `stake_below_required` flag. | Must |
| HIP-5 | A failed or unparseable info API response marks all HIP-3 facts `Unavailable` for the day and never fails the run. | A mocked 500 still publishes HyperEVM and Solana results. | Must |
| HIP-6 | A DEX that appears or disappears, or whose deployer, oracle updater or caps change, raises an event (CHG rules). | A fixture pair with a changed oracle updater emits one event with before and after. | Must |
| HSCO-1 | `Timelock` and `Multisig { kind: safe }` serialize, validate against the published schema, and round-trip; schema versions get a minor bump (additive). | A facts file with both variants validates and re-serializes byte-identically. | Must |
| HSCO-2 | Existing scorer rules apply unchanged to HyperEVM. A timelock's delay is its `min_delay_secs`, and its control is its proposer. A Safe rates on threshold and voters like Squads. | A Safe 5 of 8 behind a 7-day timelock rates upgrade control Low, delay Low. | Must |
| HSCO-3 | A timelock whose admin is an outside key is rated on that key with zero delay. | The outside-admin fixture rates delay High. | Must |
| HSCO-4 | HIP-3 DEXs produce no factor ratings and no Stage in v0.1. `scores.json` lists them with `rated: false` and the reason "methodology v0.1 has no HIP-3 rules". | Scoring a facts file with DEXs adds no ratings and changes no other protocol's score. | Must |
| HSCO-5 | A Solana-only facts file scores byte-identically before and after wave 2. | The last published `facts.json` re-scores to the published `scores.json` (SNAP-4). | Must |
| HSCO-6 | Risk sentences cover Safe and timelock holders and pass the LCH-3 banned-words lint. | A sentence for a timelock reads "Upgrades wait 2 days and need …", and the lint passes. | Must |
| HUI-1 | The chain filter makes Hyperliquid selectable (UI-9). The choice lives in the URL (UI-1), and counts per chain match the data. | Choosing Hyperliquid shows only HyperEVM protocols and HIP-3 DEXs, and the URL reproduces it. | Must |
| HUI-2 | Evidence links go to `hyperevmscan.io` for HyperEVM and the Hyperliquid explorer for HyperCore. Addresses shorten and copy as today (UI-5). | Every Hyperliquid evidence link has the right host. | Must |
| HUI-3 | A HIP-3 DEX page shows deployer, oracle updater, fee recipient, stake and per-asset caps. Factor pills say "Not rated in v0.1" through `RatingBadge`, with no Stage badge. | The page renders with every value from a fixture; axe passes. | Must |
| HUI-4 | The control graph and permissions render Safe (owners as nodes) and timelock (delay on the edge, proposer as parent). | A graph test for a Safe behind a timelock shows both nodes and the delay label. | Must |
| HUI-5 | The methodology page explains the HyperEVM factor sources, the HIP-3 anchor block and why HIP-3 isn't rated yet. | The page contains the three sections. | Should |
| HUI-6 | Leaderboard and summary charts keep chain-neutral totals and can split by chain. | The TVL chart shows a per-chain split once Hyperliquid data exists. | Could |
| HOPS-1 | The daily workflow collects both chains in one run; `run.json` reports call counts and failures per chain. | `run.json` has a `hyperliquid` section. | Must |
| HOPS-2 | A fixture capture script (`scripts/capture-hyperliquid.*`) records `perpDexs`, staking and the `eth_call` results with block number into `fixtures/hyperliquid/`. Tests never touch the network. | Tests pass with the network disabled. | Must |
| HOPS-3 | `scripts/trace.sh` checks Hyperliquid requirement IDs against the completion ledger and tests, like iteration 1. | Removing a P9 ID from the completion ledger fails trace. | Must |

## Completion ledger

Historical slice IDs: `[x]` implemented, `[ ]` open, `[H]` owner/review work, `[G]` release gate. Refer to current state above for superseding QA work.

### P0 Bootstrap

- [x] **0.1** Git repo, Cargo workspace (6 crates), toolchain pin, licences — LCH-5*

- [x] **0.2** CI workflow: fmt, clippy -D warnings, test, trace — QA-5*

- [x] **0.3** Project guidance and `scripts/trace.sh`

- [x] **0.4** Initial architecture decisions reviewed by the owner (2026-10-17)

### P1 Core and registry (offline)

- [x] **1.1** Facts model: `Controller`, `Evidence`, `Fact<T>` that cannot exist without evidence — SNAP-1 · deps: 0.3

- [x] **1.2** Deterministic JSON (sorted keys, sorted arrays, no timestamps) and `schema_version` + JSON Schema export (`schemars`) — SNAP-2 SNAP-3 · deps: 1.1

- [x] **1.3** RPC URL reduced to host in all output; per-file size budgets — SNAP-5 SNAP-6 · deps: 1.2

- [x] **1.4** Registry types, YAML loading, JSON Schema, offline `validate` (schema, sources, core program, IDL path + origin) — REG-1 REG-3 REG-5 REG-8 · deps: 1.1

- [x] **1.5** `shared_with` rule for addresses in two protocols — REG-4 · deps: 1.4

- [x] **1.6** `registry/tokens.yaml` schema + protocol token usage — TOK-1 TOK-2 · deps: 1.4

- [x] **1.7** CLI shell (clap): subcommands, `CELEBTIR_RPC_URL` fallback, exit codes 0/1/2/3, `--format json` — CLI-1 CLI-2 CLI-3 · deps: 1.4

### P2 Scorer v0.1

- [x] **2.1** `methodology/v0.1.toml` loader; scorer crate has no I/O deps — SCO-1 SCO-2 · deps: 1.2

- [x] **2.2** Six factor rules, one test per table cell, voters not members — SCO-3 · deps: 2.1

- [x] **2.3** Weakest link across core programs; periphery shown, not counted — SCO-4 · deps: 2.2

- [x] **2.4** Reason, evidence refs and next step on every rating — SCO-5 · deps: 2.2

- [x] **2.5** Unknown ratings: neutral, caps Stage at 0; Unknown upgrade control → Unrated — SCO-7 · deps: 2.2

- [x] **2.6** Stage decision + list of unmet requirements for the next Stage — SCO-6 · deps: 2.3 2.5

- [x] **2.7** Risk sentences with severity + banned-words lint on templates — SCO-9 LCH-3 · deps: 2.4

- [x] **2.8** Token scoring: four factors, no Stage, keyed by mint — SCO-10 TOK-6 · deps: 2.2

- [x] **2.9** Protocol token exposure + High-rated collateral warning — TOK-7 · deps: 2.8

- [x] **2.10** Property tests (monotonicity, determinism), 10,000 cases — SCO-8 · deps: 2.6

- [x] **2.11a** Investigate release-build failure on Rust 1.88 (it was flaky on the local machine, not a dependency issue); add a release build to CI · deps: 2.10

- [x] **2.11** `wasm32-unknown-unknown` build in CI — SCO-11* · deps: 2.1

- [x] **2.12** `celebtir score` and `celebtir explain` on hand-written facts fixtures; scorer coverage ≥ 90 % enforced — QA-7 · deps: 2.6 1.7

- [x] **2.13** Methodology review approved (2026-10-17)

### P3 Solana collector

- [x] **3.1** RPC client: token bucket, concurrency cap, backoff with jitter, call budget — COL-1 COL-2 COL-5 · deps: 1.7

- [x] **3.2** `finalized` commitment, slot on every fact, min/max slot; `getMultipleAccounts` batching — COL-3 COL-4 · deps: 3.1

- [x] **3.3** Fixture capture script (real mainnet bytes + slot) and fixtures layout — QA-1 · deps: 3.2

- [x] **3.4** Program account + 45-byte ProgramData slice; loader classification — COL-6 COL-7 · deps: 3.3

- [x] **3.5** Squads v4 decoding: owner, discriminator, threshold, time lock, config authority, Vote-permission voters — COL-9 · deps: 3.3

- [x] **3.6** Controller resolution (vault derivation, on/off curve, `controllers.yaml`), config authority 3 levels, loop detection — COL-8 COL-10 · deps: 3.4 3.5

- [x] **3.7** Admin accounts: owner check + IDL field decode, "unavailable" on failure — COL-11 · deps: 3.6

- [x] **3.8** Oracle owner classification via `oracles.yaml` — COL-12 · deps: 3.2

- [x] **3.9** OtterSec verified-build status + repo-org mismatch flag — COL-13 · deps: 3.1

- [x] **3.10** Dependencies: 50 tx sample, lookup tables, stack heights, drop system programs, callee ProgramData one level, cache — COL-14 · deps: 3.6

- [x] **3.11** Mints: owner program, supply, decimals, mint/freeze authority — TOK-3 · deps: 3.2

- [x] **3.12** Token-2022 extensions (unknown listed by type number); authorities via controller logic — TOK-4 TOK-5 · deps: 3.11 3.6

- [x] **3.13** On-chain `validate`: base58/32 bytes, program exists and is executable — REG-2 · deps: 3.2

- [x] **3.14** `collect` command, per-protocol failure isolation, run report — COL-15 · deps: 3.7 3.8 3.9 3.12

- [x] **3.15b** Collector line coverage >= 70% enforced in CI (second half of QA-7) · deps: 3.14

- [x] **3.15** `--offline` replay + golden tests for the whole pipeline — CLI-4 QA-3 · deps: 3.14

- [x] **3.16** One fixture test per edge-case row in the spec (Unrecognized vault, Realms PDA, Squads v3, config authority + time lock, nested multisig, cannot act, closed program, …) · deps: 3.15

- [x] **3.17** List the underlying feeds of protocol-run oracle aggregators (Kamino Scope) where decodable · deps: 3.8 — registry `scope_chain` on the oracle entry

### P4 Metrics and change detection

- [x] **4.1** DefiLlama TVL into snapshots; outage hides numbers, never fails the run — VAL-1 VAL-7 · deps: 3.14

- [x] **4.2** TVL by asset group — VAL-3 · deps: 4.1

- [x] **4.3** Volume and fees, 7-day change — VAL-6 · deps: 4.1

- [x] **4.4** Value by control type (weakest-link attribution), classes sum to total — VAL-2 · deps: 4.1 2.6

- [x] **4.5** ProgramData history: classify deploy/upgrade/set-authority/extend/close, slot → date — VAL-4 · deps: 3.4

- [x] **4.6** Upgrade metrics: count, last, average interval, last 30 days — VAL-5 · deps: 4.5

- [x] **4.7** `diff`: typed events with before/after, evidence, tx signature, source tag — CHG-1 CHG-2 · deps: 3.14 2.6

- [x] **4.8** Noise control (3 runs / 2 runs), stable IDs, append-only check — CHG-3 CHG-4 · deps: 4.7

- [x] **4.9** Missed runs: last good snapshot, "detected within N days" — CHG-5 · deps: 4.7

- [x] **4.10** Token change events — TOK-8 · deps: 4.7 3.12

- [x] **4.11** `score --facts` reproduces a published day byte for byte — SNAP-4 · deps: 2.12 3.15

- [x] **4.12** `build-site-data`: `data/latest/`, `data/history/<protocol>.json` · deps: 4.4 4.6 4.8

- [x] **4.13** Summary chart series: total TVL, value by control type, Stage distribution — VAL-8 · deps: 4.12

- [x] **4.14** Compaction to weekly after 180 days — SNAP-7 · deps: 4.12

- [x] **4.15** Token supply history — TOK-9 · deps: 4.12

- [x] **4.16** Atom feeds (global + per protocol) — CHG-6 · deps: 4.8

### P5 Registry content (can start after 1.6)

- [x] **5.1** `registry/oracles.yaml` and `registry/controllers.yaml` with sourced program IDs · deps: 1.4

- [x] **5.2** Pick the 12 protocols (TVL, ≥2 per category); record the list and sources in the registry · deps: 1.4

- [x] **5.3** Draft 12 protocol YAMLs + vendored IDLs, each entry with a source URL; Realms check · deps: 5.2 — no SPL Governance among the 12

- [x] **5.4** Draft `registry/tokens.yaml` for every token the 12 use · deps: 5.3 1.6 — 12 tokens

- [H] **5.5** Human review of 5.1–5.4 after the 24-hour cooling period

- [H] **5.6** Ground-truth audit: re-check every controller on Solscan, sign-off checklist — QA-6*

### P6 Operations

- [x] **6.1** `daily.yml`: 02:17 UTC + manual trigger, step chain, artifacts per step — OPS-1* OPS-2* · deps: 4.12

- [x] **6.2** Failure issue labelled `pipeline-failure`, bot commit per day, usage projection, schedule check — OPS-4* OPS-5* OPS-6 OPS-7* · deps: 6.1

- [x] **6.3** Registry PR cooling-period required check — REG-6* · deps: 6.1

- [x] **6.4** "Registry review due" flag after 90 days — REG-7 · deps: 4.12

- [H] **6.5** Create GitHub repo, push, add secrets, branch protection, least-privilege workflow token, install `gh` — OPS-3*

### P7 Site (Next.js static export, `site/`)

- [x] **7.1** Scaffold: Next.js static export, TypeScript, dark theme tokens, fonts, Low/Medium/High icon+text, contrast checks, charting library — UI-24 UI-26 · deps: 4.12

- [x] **7.2** Data loader, freshness bar (36 h warning), the four data states — UI-2 UI-7 · deps: 7.1

- [x] **7.3** URL state for tabs, filters, sort, columns, compare — UI-1 · deps: 7.1

- [x] **7.4** Summary: charts with ranges, category tabs, protocol table, column picker, filters — UI-8 UI-9 UI-10 UI-11 · deps: 7.2 7.3

- [x] **7.5** Protocol page: 12 sections with empty states, permissions by actor — UI-12 UI-13 · deps: 7.2

- [x] **7.6** Control graph (React Flow), side panel, colour + label, shared controllers, list view — UI-19 UI-20 UI-21 UI-22 UI-23 · deps: 7.5

- [x] **7.7** Risk, Value, Activity, Tokens and token pages, Archived · deps: 7.4

- [x] **7.8** Compare page + tray, differences-only, overlay charts — UI-4 UI-16 UI-17 UI-18 · deps: 7.4

- [x] **7.9** Changelog, methodology, glossary (inline terms), about, API, footer notices, corrections policy, no cookies — UI-6 LCH-2 LCH-4 LCH-6 · deps: 7.2

- [x] **7.10** Keyboard shortcuts, address shortening + copy, Solscan evidence links — UI-3 UI-5 · deps: 7.4

- [x] **7.11** Markdown twin per protocol + `llms.txt`; Open Graph images — UI-14 UI-15 · deps: 7.5

- [x] **7.12** `/data/v1` static API with schemas, versioning rule, CORS, licence field; SVG badges — API-1 API-2 API-3 API-4 API-5 · deps: 4.12

- [x] **7.13** Mobile layout at 375 px (cards, 2-up compare) — UI-25 · deps: 7.8

- [x] **7.14** Frontend tests: component, e2e, visual regression, axe; Lighthouse budgets in CI — QA-4 UI-27 · deps: 7.13

- [x] **7.15** UI review completed 2026-10-19 on the owner's instruction; outstanding design and hosting decisions stay with the owner

### P8 Hardening and launch

- [H] **8.1** Devnet test bed (5 program configs + Token-2022 mint), weekly CI job — QA-2*

- [x] **8.2** Publish to crates.io readiness, `--version` prints the commit — CLI-5 · deps: 1.7 — the owner runs `cargo publish` (core, registry, score, diff, collect-solana, then celebtir)

- [H] **8.3** 7 consecutive green daily runs

- [H] **8.4** Team previews 7 days before launch, contacts logged — LCH-1*

- [ ] **8.5** Launch kit drafts: thread, write-up, README with demo (recording is human) — LCH-7* · deps: 8.3

- [ ] **8.7** Bump schema versions to 1.0.0 and freeze `schemas/baseline/<name>.v1.schema.json` (enables the SNAP-3 removal check) · deps: 8.3

- [H] **8.8** Seed the Linux pixel baselines: run the `ci` workflow by hand with `update_visual_baselines`, download the `visual-baselines-linux` artifact into `site/e2e/__screenshots__/linux/` and commit it (until then CI skips the pixel comparison and keeps the style check) · deps: 6.5

- [G] **8.6** Definition-of-done checklist fully ticked

### P9 Wave 2: Hyperliquid

- [x] **9.0** Owner reviewed Hyperliquid implementation scope (accepted all defaults, 2026-10-21)

- [x] **9.1** `scripts/trace.sh` checks Hyperliquid requirements too — HOPS-3 · deps: 9.0

- [x] **9.2** `Timelock` + `Safe` facts end to end: schema bump, round-trip, scorer tests, sentences — HSCO-1 HSCO-2 HSCO-3 HSCO-6 · deps: 9.1

- [x] **9.3** Registry: `chain: hyperevm`, EIP-55 addresses, proxy kinds, timelock proposers, EVM sections in controllers/oracles — HREG-1 HREG-2 HREG-3 · deps: 9.1

- [x] **9.4** HyperEVM JSON-RPC client: rate limit, retries, budget, pinned block, host-only URL — HCOL-1 HCOL-2 · deps: 9.1

- [x] **9.5** Fixture capture script + first fixtures (perpDexs, staking, eth_call/storage at block) — HOPS-2 · deps: 9.4

- [x] **9.6** Proxy reads and controller classification (EOA, Safe, timelock, unknown), depth 3, loops — HCOL-3 HCOL-4 · deps: 9.3 9.5

- [x] **9.7** Safe modules and timelock proposer/admin checks — HCOL-5 HCOL-6 · deps: 9.6

- [x] **9.8** Admin powers, oracles, declared dependencies — HCOL-7 HCOL-9 HCOL-10 · deps: 9.6

- [x] **9.9** Verified source: Sourcify, then Etherscan v2 with optional key — HCOL-8 · deps: 9.4

- [x] **9.10** HIP-3: perpDexs parse, raw response, evidence anchor, stake check, outage handling — HIP-1 HIP-2 HIP-3 HIP-4 HIP-5 · deps: 9.5

- [x] **9.11** Scores: HIP-3 `rated: false`; Solana-only byte-identical regression — HSCO-4 HSCO-5 · deps: 9.2 9.10

- [x] **9.12** CLI + pipeline: both chains per run, per-chain failure isolation, `run.json` per chain; HIP-3 change events — HCOL-11 HOPS-1 HIP-6 · deps: 9.7 9.8 9.9 9.11

- [x] **9.13a** `hip3.yaml` (required stake + source); unknown DEX names warn — HREG-4 · deps: 9.10

- [H] **9.13** Human review of the 4 drafted HyperEVM registry files (kinetiq-khype, sthype, morpho-blue, hyperbeat-lst); confirm the stHYPE timelock proposers and the Hyperbeat upgrade role; decide on HyperLend (Aave immutable-admin proxy, unsupported) — HREG-5* · deps: 9.3

- [x] **9.14** Site: Hyperliquid chain option, explorer links, HIP-3 page, Safe/timelock graph — HUI-1 HUI-2 HUI-3 HUI-4 · deps: 9.12

- [x] **9.15** Methodology page sections — HUI-5 · deps: 9.14

- [ ] **9.17** Per-chain split on the summary TVL chart (Could; deferred until Hyperliquid history exists) — HUI-6 · deps: 9.16

- [H] **9.16** 7 consecutive green daily runs covering both chains

## Folder layout review (2026-10-21)

Brand source artwork lives in site/assets/brand/celebtir-logo.png; regenerate served assets with pnpm.cmd -C site brand:assets. The redesign specification lives in docs/design/highwatch-product-redesign.md. Generated brand outputs remain in site/public/brand/ and Next metadata icons in site/src/app/.



## Highwatch visual gate (2026-10-22)

The owner requested adjustments before design propagation. Current Summary, Risk, grouped Tokens and Compare previews are under review; do not extend their visual direction to other pages or commit until approved. Tokens now default to asset-type grouping, with all groups expanded and counts/jump links; group collapse remains a proposed correction. Review captures, remaining weaknesses and validation are in site/output/playwright/highwatch-20261022/review/REVIEW.md. The local preview uses output/qa-v02-data at http://localhost:3100. This iteration does not activate registry drafts or change scoring/source data.

