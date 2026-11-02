# Score and research review — final handoff

Spec v1.1.1 executed for local review on 27–28 October 2026. **Release sign-off is blocked.** Local regression passes; production version reconciliation, three source-attribution gaps, performance and manual verification remain open. No Ethereum implementation was started.

HEAD: `b114b281ae6a57d3d6ff198963087e999323a5eb`, main, initially dirty with README deletion and nearly all application files untracked. Existing branding work was preserved. This report is not a clean-commit diff or a deployed source binding. The generated `changed-files.json` lists review source/report/capture files and hashes, with reasons; disposable build output is listed separately.

## Findings

- Public snapshot 2026-10-26 uses methodology **0.2**; the owner expected **0.3**. It was still the public snapshot on 28 October. Current public API and browser labels agree internally. The remote commit/run/timer state is not exposed by the captured artifacts.
- All 12 published Solana protocol scores reproduce exactly under the current Rust v0.2 evaluator: **10 Stage 0, 2 Unrated, zero Stage 1/2**. Review next-level criteria contain **14 verified unmet and 13 unresolved** observations. These are criterion counts, not protocol counts or alternate scores.
- Drift Staked SOL, Jito and Jupiter Staked SOL lack enough source evidence to attribute the unresolved configuration-authority criterion. Their cause arrays remain empty, with `CAUSE_CLASSIFICATION_UNRESOLVED`; G1 is blocked rather than assigned an invented cause.
- No scorer mismatch was found. Counterfactual override analysis is `BLOCKED_UNSUPPORTED`; deterministic structural frequency analysis is supplied instead. The published rubric does not encode per-criterion exemptions; factor N/A is not converted into one.
- Orca and Jito: `REASSESS_EXISTING`. Marinade, Solstice and Phoenix Legacy: `DRAFT_RESEARCH_ONLY`. All five dossiers contain fresh executable/ProgramData authority-root captures and explicit unresolved terminal-controller/admin/bypass paths. **Zero admissions; publication_authorized=false throughout.**

The update deployment path rebuilds the newest stored snapshot without rescoring it, which can preserve v0.2 under a newer executable. This is a code-supported explanation to investigate, not a verified account of the remote deployment. See baseline-findings.md for the bounded read-only follow-up.

## Executed validation

Input for local gates: `CELEBTIR_DATA=output/qa-v03-data`, snapshot 2026-10-26/methodology0.3. Git Bash PATH precedes Windows/WSL bash; Rust uses `target/review`, one build job, incremental off, dev/test debug off. Browser workers: four. Checks below ran on 28 October; log timestamps and machine-readable execution records are retained in the review output.

| Check / command | Status | Actual evidence |
|---|---|---|
| `bash scripts/gates.sh` | PASS | logs/gates-final.log ends `gates: ok` |
| Rust fmt, Clippy, `cargo test --workspace`, WASM scorer | PASS | 535 Rust tests; includes legacy/version replay tests; no scorer changes |
| `bash scripts/trace.sh` and trace regression | PASS | 159 spec IDs, 116 completed Must IDs with tests |
| Site lint/typecheck/unit tests | PASS | 35 files, 305 tests |
| Static build and candidate/artifact checks | PASS | 117 generated pages, required API/twins/cards present |
| `pnpm.cmd -C site budget` | PASS | Summary209.4KB, Kamino203.5KB, About/Methodology184.4KB, Value203.9KB gzip |
| `pnpm.cmd -C site e2e` | PASS | 142 passed, 30 skipped; four new viewport/keyboard/axe cases included |
| Opt-in hosted browser check | PASS | logs/hosted-browser.log; 42 route/viewport observations, no captured JS/network/overflow failures |
| `pnpm.cmd -C site lighthouse` | FAIL | logs/lighthouse-final.log; performance0.91/0.80 versus0.95 minimum; accessibility1.00 and LCP pass |
| `node scripts/score-review-verify.mjs` | PASS | 545 protected hashes unchanged, byte-deterministic report, 12 exact score matches, cause dedup/mismatch fixture, candidate API unchanged, targeted credential scan |
| Native browser200% zoom / owner visual acceptance | NOT_RUN | CSS equivalent reflow passed; not claimed equivalent to browser chrome zoom |
| Remote Linux publication/timer/source binding | BLOCKED | Public artifacts lack required source/run identity; no host access performed |

Earlier failed attempts remain in logs: Windows sandbox loopback access and wrong Bash path caused environmental failures; corrected gate rerun passed. The initial text snapshot failure was the required new copy. Owner explicitly authorized updating necessary references for the new version; one UI text snapshot and its exact-copy assertion were updated. No data/scoring goldens or screenshot baselines were regenerated.

## Release blockers and required follow-up

| Severity | Finding / reproduction | Required action and regression evidence |
|---|---|---|
| P0 release intent | GET public summary remains v0.2 though owner expects v0.3 | Bind current host release, stored snapshot and daily publication journal; select an authorized reviewed v0.3 candidate. Re-run public API/browser methodology checks after any separately authorized deployment. |
| Evidence gate | Three configuration-authority classifications unresolved | Capture and independently review effective administrative/configuration paths, including affirmative absence evidence where claimed. Re-run deterministic diagnostics; do not fabricate a negative from an empty inventory. |
| P1 performance | Existing LHCI command fails both URLs | Investigate HTML/client payload, critical rendering and actual host compression with a comparable baseline; preserve budgets. Existing LHCI assertion is the regression check. Owner/performance maintainer owns release disposition. |
| Manual/environment | Native zoom, deployed source binding, timer freshness and Linux publication not verified | Complete on the relevant browser/host; local tests do not substitute for these checks. |

G0 BLOCKED; G1 BLOCKED; G2 PASS; G3 PASS; G4 PASS for the research deliverables (not admission); G5 FAIL; G6 BLOCKED. Machine statuses and evidence paths: `output/reviews/score-review-2026-10-26/gate-results.json`.

## File groups and decisions

`scoreReview.ts` and `protections.ts` centralize pure review semantics. Existing Summary/Risk/Compare/profile views carry typed inspections from the same selected snapshot. StageInspector uses the existing dialog primitive; ProtectionsBrowser binds selected facts to exact evidence; ReviewEvidence defers repeated lists. Tooltip adds optional activation without replacing existing hover behavior. Existing labels and data-state copy were updated; URL-state serialization remains shared. Scripts capture public evidence separately from offline reports/tests. Five dossiers, baseline/diagnostic/selection/UI/admission reports and AGENTS.md record findings. Every source path and generated evidence file is enumerated in changed-files.json.

Protected data, registry, fixtures, methodology and scorer source retain their initial hashes. Candidate public API values match the pre-review local export. Public captured artifacts retain their own timestamp/hash provenance. No external dependency was added. No deployment, registry or scheduling configuration was changed.

**NO COMMIT / NO PUSH / NO DEPLOY / NO REGISTRY ACTIVATION**
