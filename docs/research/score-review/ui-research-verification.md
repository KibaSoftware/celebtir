# UI and research verification

The changes add review annotations to published facts and scores; they do not replace the Rust evaluator. Local inspection uses snapshot 2026-10-26, methodology v0.3. Hosted captures remain the separate v0.2 baseline.

| Source contract | Implementation | Verification |
|---|---|---|
| Exact Stage definition, Stage 0 context, four data-state explanations | lib/scoreReview.ts, components/DataState.tsx, app/page.tsx, app/methodology/page.tsx | 305 UI tests; text snapshot updated with explicit owner authorization |
| Source-backed criterion labels, multiple causes, no guessed cause | lib/scoreReview.ts | Ten review/protection unit tests and deterministic artifact checks; three live attribution anomalies remain blocked |
| Accessible Stage inspection on Summary, Risk, Compare and protocol header | research/StageInspector.tsx, ProtocolBits.tsx, Tooltip.tsx and existing DetailPanel | Enter/Space/Escape, focus return, accessible counts, axe and four explicit viewport tests |
| Exact criteria and evidence on protocol profile | protocol/Sections.tsx, research/ReviewEvidence.tsx | Required copy retained; repeated evidence mounts only when expanded |
| Factor-first protection comparisons | lib/protections.ts, research/ProtectionsBrowser.tsx, page/RiskView.tsx | Three-valued AND; unknown-last delay sorting both directions; stable ID ties; distinct Unrated group; domain/version boundaries |
| Scope and shareable state | lib/viewState.ts | Whitelisted, canonicalized protection filters and existing URL-state lifecycle; browser reload/reset preserves chain |
| Observation-to-evidence binding | ProtectionsBrowser exact selected observation | Evidence dialog receives that observation's accounts, including administrative roots; it does not redirect them to an unrelated factor |

No new scoring composite or overall safety ordering exists. Name, observed delay and unresolved-observation sorts describe specific mechanisms. Multisig keys are not asserted to be independent people. Missing bypass evidence remains UNKNOWN; immutable programs are not assigned infinite delay. The current schema supports the implemented filter fields; individual missing facts remain UNKNOWN rather than silently passing. Native HyperEVM and HIP-3 do not receive Solana Stage annotations.

The Stage inspector displays canonical next-level criteria and exact scorer-provided missing requirements. Unrated records retain their published Stage 0 entry criteria for inspection; diagnostic `next_stage` is null, with original `published_next_stage` retained separately. No earned-progress claim or review-generated Stage is introduced.

## Captures

All review screenshots are in `output/reviews/score-review-2026-10-26/screenshots/`:

- `1440-stage-inspector.png` and `1440-protections.png`: desktop 1440×900.
- `768-stage-inspector.png` and `768-protections.png`: tablet 768×1024.
- `375-stage-inspector.png` and `375-protections.png`: mobile 375×812.
- `720-stage-inspector.png` and `720-protections.png`: equivalent 200% CSS reflow from a 1440×900 viewport; **not native browser chrome zoom**.
- `regression/`: existing route/gallery captures from the full regression suite.

The 375px Stage drawer was visually inspected: readable wrapping, visible close control and scrollable evidence. Full automated browser regression: 142 passed, 30 skipped. The four new explicit-viewport tests passed; phone-project duplicates are deliberately skipped. Native 200% browser zoom and owner visual acceptance remain open. Existing demo/platform-only pixel checks remain skipped according to their pre-existing conditions; no screenshot baseline was updated.

Hosted screenshots and `hosted-browser/results.json` cover 14 actual routes at 1440, 768 and 375px: Summary, Risk, Value, Activity, Tokens, Compare, Changelog, Controllers, HyperEVM, HIP-3, Methodology, Orca protocol, API and USDC token. All 42 fetched views returned 200, exposed the expected methodology/snapshot labels where applicable, and had no captured runtime errors, failed requests or horizontal document overflow. These checks verify the existing hosted build, not deployment of this local UI.

## Performance and limits

Existing JS budgets pass. Lighthouse desktop preset on Windows HeadlessChrome 156, local static server, three runs per URL: Summary performance 0.91 for all runs; Kamino 0.79/0.80/0.79. LHCI's selected median run reports 0.80 (this is not the arithmetic median of those three scores). Required minimum is 0.95. Accessibility is 1.00 for all six runs; LCP is 1856–1862ms and 2390–2426ms, within 2500ms. Exact simulated network/device settings and raw results are retained under `lighthouse/`.

Repeated criterion evidence rendering and redundant client-side causal payload were reduced before the final measurement. This did not resolve the performance failure. No assertion about pre-review performance is possible from an equivalent baseline measurement. Do not weaken the budget or claim production performance from this local server.
