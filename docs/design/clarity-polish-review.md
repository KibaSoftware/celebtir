# Platform-wide clarity polish

Implemented locally on 2 November 2026 for owner visual review. The site now leads with control findings and puts complete technical records behind explicit disclosures. Branding, semantic colors, artwork, navigation, route identities, published ratings and methodology remain intact.

[Open the annotated before/after gallery](../../output/playwright/clarity-polish/index.html). It includes 19 route/entity examples at four sizes and four additional protocol-section comparisons. [Changed-file manifest](../../output/clarity-polish/changed-files.json), [protected-input/API parity](../../output/clarity-polish/parity.json), and all verification logs are under `output/clarity-polish/`.

## Scope and presentation decisions

The owner's confirmed platform-wide plan governs this iteration. The attached presentation brief supplied guidance; its narrower route restrictions did not limit this work. The intended reader understands DeFi but need not interpret account fields or developer terminology to understand the first screen.

Protocol profiles now follow Summary → Risk summary → Risk analysis → Stage → Permissions → Control graph → Programs → Dependencies and sources → Value → Activity → Upgrades → Updates. Existing section IDs and local jump links are preserved. Markdown twins follow the same order. Economic metrics remain available in the Value and Activity sections.

The first screen shows identity, snapshot/methodology, recorded upgrade controller, approval configuration and delay, Stage and next-Stage inspection counts, followed by six concise findings. Finding copy uses typed controller, permission, transparency, feed and dependency observations. It does not derive protections from favorable ratings. Complete original reasons, methodology criteria and evidence remain accessible through clearly labeled disclosures. Elevated ratings, insufficient evidence and sampling limitations remain visible.

Stage groups reuse the existing inspection statuses: met, verified unmet and unresolved. This is a presentation of existing results, not a new criterion evaluator or combined safety score.

Token profiles put minting, freezing and transfer findings and four factor ratings before mint/supply/source metadata. Existing issuer/category evidence supports the stablecoin explanation. Reading-layer wording distinguishes a recorded signing authority from a proven human signer or off-chain approval arrangement. Original stored controller classifications and recorded reasons are retained.

Risk inspectors distinguish observations, implications, supported protections, assessment boundaries and evidence. Compare uses concise observed configurations and keeps complete reasons in its existing inspector. The original comparison equality rules, row alignment, 2–4 selection limit, mobile paging, difference filtering and share state remain unchanged.

Long histories show ten records initially, preserving chronology, severity, attribution and coverage warnings. Shared controllers and graph relationships, permission holders and nested permission tables, oracle observations and evidence lists use the same disclosure pattern. All records remain mounted and reachable, including targets within collapsed ancestors. Initial fragments, hash changes and browser back/forward reveal and focus the actual target. Table controls span the correct number of columns; counts describe the displayed records accurately. The graph canvas stays bounded.

Summary previews retain complete reasons on demand. HIP-3 leads with compact facts and expandable technical records; it remains unrated. HyperEVM already had expandable contract records; shared evidence and authority-tree improvements apply without introducing ratings or Solana Stages. Methodology keeps its explanatory criteria and anchors visible and collapses the full TOML listing. Tokens, Value and Activity retain their analytical tables/charts. API, About and Archived retain their structure. Shared freshness presentation keeps date, version and stale-data warnings visible while disclosing numeric observation anchors. The Glossary's mobile alphabet overflow is fixed.

## Visual evidence and findings

The gallery pairs were captured in the same Chrome session at 1440×900, 768×1024, 375×812 and 720×450. The last size models 200% equivalent reflow. Captures wait for network idle and font readiness. Before views use the saved original source; after views use the final static export. Separate automated headless captures/metrics were also produced by the regression suite; their scrollbar widths differ slightly, so the gallery uses the matching same-session pairs.

| Default desktop page | Before height | After height | Main change |
| --- | ---: | ---: | --- |
| Kamino Lend | 31,197 px | 12,614 px | Concise factors and bounded permissions/history |
| Jupiter Lend | 33,373 px | 15,791 px | Unrated boundary retained, nested feed/evidence disclosure |
| Changes | 63,773 px | 2,384 px | Ten chronological records initially |
| HIP-3 | 6,179 px | 3,701 px | Facts first, technical details on demand |
| JLP | 2,525 px | 2,182 px | Powers first, responsive facts |

These measurements document the effect; reducing height was not an acceptance target. Some views, including Compare and Controllers, grow slightly because findings and disclosure controls now wrap instead of being clipped or crowded.

JLP exposed a concrete baseline overflow: content measured 524 px wide in a 375 px viewport, and 892 px in a 768 px viewport. The revised page fits at all four review sizes. The automated gallery verifies every reviewed route has no document-level horizontal overflow and no clipped profile-header metric. Technical tables retain their intended horizontal scrolling when expanded.

Section pairs show top-aligned factor cards without stretched empty bottoms, Stage disclosure, ten permission holders with nested evidence, and the unchanged bounded graph. The original narrow protocol screenshot was unavailable; none of these findings is attributed to an unrelated old Summary mockup.

## Changed files

The [complete manifest](../../output/clarity-polish/changed-files.json) contains 35 source/test files under `site/src`, two browser-test files and the two documentation files. Main areas:

| Area | Files and purpose |
| --- | --- |
| Shared disclosure and deep links | `LimitedItems.tsx/.module.css/.test.tsx`, `protocol/HashFocus.tsx`, `app/layout.tsx`, `Evidence.tsx` |
| Factual reading layer | `lib/presentation.ts/.test.ts`, `research/Finding.tsx`, `ControllerAuthorityTree.tsx` |
| Profiles | `ProtocolHeader.tsx/.module.css`, `Sections.tsx/.module.css`, `protocol-page.test.tsx`, token profile page, `lib/twin.ts` |
| Risk, comparison and exploration | `RiskView.tsx`, `CompareView.tsx`, `lib/compare.ts`, `TokenTable.tsx`, `SharedControllers.tsx`, `GraphList.tsx`, `Graph.module.css` |
| Other routes and layout | Summary page, Methodology page, Glossary page, `ChangelogView.tsx`, `Hip3List.tsx`, `Page.module.css`, `Reference.module.css`, `FreshnessBar.tsx/.module.css`, `globals.css` |
| Behavior checks and handoff | `qa-regressions.test.tsx`, `e2e/clarity.spec.ts`, `e2e/graph.spec.ts`, this report, `AGENTS.md` |

No packages, backend files, scoring rules, registry entries or API schemas were changed.

## Verification

All builds and tests select `CELEBTIR_DATA=D:\Work\celebtir\output\qa-v03-data`: preserved snapshot 2026-10-26, methodology v0.3. Nothing was collected or rescored.

| Check | Result and evidence |
| --- | --- |
| Protected inputs | **397 unchanged** SHA-256 hashes across original data, preserved v0.2/v0.3 candidates, registry, methodology/bundled methodology and schemas; `protected-before.json`, `protected-after.json`, `parity.json` |
| Exported public API | **51 files byte-identical** to candidate API v2, including schemas; `parity.json` |
| UI unit/component tests | **315 passed in 38 files**; `unit-final.log` |
| Lint / TypeScript | Passed; `lint.log`, `typecheck.log` |
| Static build | **117 pages generated**, 16 protocol Markdown twins; `build-final.log` |
| JavaScript budget | Passed; Summary 210.3 KB gzip, Kamino 203.9 KB, Value 204.7 KB; `budget-final.log` |
| Browser suite and corrective reruns | **151 unique cases passed, 31 existing/conditional skips**; `browser.log`, `browser-final.log`, `browser-verified.log`, `browser-passed.json` |
| Final accessibility/disclosure rerun | **45 passed, one duplicate mobile-gallery skip**; `browser-verified.log` |
| Explicit mobile touch events | Passed show-all/show-fewer tap behavior; `touch-verified.log` |
| Route/reflow gallery | **76 final route/viewport checks**, plus 76 matching original-source captures and 16 section captures; gallery and metric JSON files |

The broad browser run initially found seven failures: profile link contrast, hidden-target test selection, button-renaming test selection, and mobile glossary overflow. A corrective rerun resolved these and exposed token disclosure target spacing; that spacing was fixed and the final accessibility/disclosure rerun passed. The 151 count is the union of unique passing cases across the broad suite and focused reruns, not a claim that the initial run was green. Skips cover hosted-only checks without a configured host, demo-only pixel tests and desktop-only tests/gallery work excluded from the duplicate phone project. No golden screenshots or scoring goldens were regenerated.

New behavioral checks cover ten-item previews, ordered lists, table spans, show-all/show-fewer counts, keyboard focus retention, touch, malformed fragments, nested hidden targets, initial navigation and browser back/forward. Browser checks compare expanded Kamino permission records, published factor badges/evidence URLs and graph node/edge counts against the existing typed page model. Existing tests cover Stage results, URL state, filters, sorting, copying, comparison and inspector focus restoration. Unit tests verify presentation does not mutate source reasons or evaluated Stage statuses.

The preserved source is under `output/clarity-polish/before-source/site/src/`. The supplemental baseline is an isolated copy under `output/clarity-polish/baseline-repo/`, rebuilt with existing installed dependencies and the same candidate. Original and reproduced desktop baseline heights agree. Capture scripts and build logs are retained with the review evidence.

The final suite runs needed an unrestricted local process environment for Next export and loopback/browser checks. Sandbox `EPERM`/loopback failures were not treated as application regressions. The pre-existing dirty repository state was retained.

## Remaining review and proposed follow-ups

- Owner visual acceptance remains open. The gallery is the concrete review artifact; no publication is implied.
- Native browser chrome zoom, physical Windows display scaling, screen-reader use and physical-device touch remain manual checks. Browser emulation and equivalent CSS reflow passed.
- The candidate is deliberately old and its visible stale-data warning remains. This task provides no new hosted data, collection/timer, Linux publication or production performance evidence. Rust/full release gates and Lighthouse were not rerun for this frontend-only change.
- The Methodology reference still measures about 16,200 px at desktop because explanatory criteria remain visible. A future iteration could add a concise question-based outline above those criteria while retaining their anchors and full text.
- Complex token authority paths can still produce long, useful risk sentences. A future evidence-reviewed editorial layer could supply entity-specific short explanations, with the existing complete reason underneath. This should use structured facts and reviewed wording, never string truncation or inferred signer independence.

No commit, push, deployment, registry activation, collection or package installation was performed for the application. Historical candidate inputs and published API values are unchanged.
