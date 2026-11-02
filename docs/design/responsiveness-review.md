# Responsiveness correction — 2026-10-25

Status: implemented locally, ready for owner visual review. No commit, push or deployment. QA v0.2 (`output/qa-v02-data`, snapshot 2026-10-21) used for both builds; the freshness warning in screenshots is expected. No backend, methodology, data contract, evidence or security changes.

## Root cause, measured before changing layout

`Shell.module.css` combined `margin-left: 208px` with `max-width: 1680px` on desktop main content. The inherited right auto margin absorbed all surplus width. At 3840 CSS pixels, main occupied x=208–1888, leaving 1952px on the right and no matching space beside the rail. Header/footer independently centered a 1520px frame at x=1264–2784. This was a real alignment defect, not simply an intentional reading measure. No document overflow was needed to produce it.

The 208px fixed navigation rail is intentional. Prose limits (`72ch`, reference text `78ch`), local scrollable tables and viewport-sized modal drawers are also intentional; they were retained. Existing analytical grids mostly already used fluid `minmax(0, 1fr)` tracks. Summary's three-column grid was adjusted to use explicit zero minimums rather than intrinsic `1fr` minimum sizing. No document-level overflow hiding was added.

## Correction and files

- `site/src/app/layout.tsx`: one full-width workspace around the shell.
- `site/src/components/shell/Shell.module.css`: reserve rail space once on the workspace; center main and header within the remaining width using the shared container; retain navigation breakpoints.
- `site/src/app/globals.css`: shared analytical maximum 2560px, mobile gutter 16px. Desktop workspace uses `clamp(24px, 1.25vw, 48px)` gutters. Font/color/illustration tokens unchanged.
- `site/src/components/reference/Reference.module.css`: footer uses the same gutters and container width.
- `site/src/app/page.module.css`: summary grid intrinsic sizing correction.
- `site/e2e/responsiveness.spec.ts`: viewport measurements, before/after captures, balanced frame/edge assertions, no document overflow, rail link bounds/non-overlap, reflow/menu focus, and DPR scaling regression checks.
- `site/eslint.config.mjs`: exclude generated Playwright reports and test results, which contain third-party minified trace-viewer code. No application lint rule weakened.
- `AGENTS.md`: current status and review limits.

The analytical frame expands 52% at native 4K compared with the old 1680px cap, without stretching all content to 3840px. At 4K, its 2560px width leaves equal 536px margins inside the 3632px workspace beside the rail. Header, main, illustrations and footer share aligned content edges. Long-form explanatory text retains its separate reading measure and may intentionally occupy less than the analytical frame.

## CSS viewport measurements

Main frame, excluding its internal padding; desktop margins are measured within the area to the right of the rail.

| CSS viewport | Before main width | After main width | After left/right workspace margin | Gutter |
| --- | ---: | ---: | ---: | ---: |
| 3840 × 2160 | 1680 | 2560 | 536 / 536 | 48 |
| 3440 × 1440 | 1680 | 2560 | 336 / 336 | 43 |
| 2560 × 1440 | 1680 | 2352 | 0 / 0 | 32 |
| 1920 × 1080 | 1680 | 1712 | 0 / 0 | 24 |
| 1440 × 900 | 1232 | 1232 | 0 / 0 | 24 |
| 1280 × 800 | 1072 | 1072 | 0 / 0 | 24 |
| 768 × 1024 | 768 | 768 | 0 / 0 | 16 |
| 390 × 844 | 390 | 390 | 0 / 0 | 16 |

All eight sizes cover Summary, Risk, Value, Activity, Tokens, two-protocol Compare, Kamino profile and Methodology: 64 matched before/after screenshots. The ordinary browser suite additionally checks generated routes, scoped navigation, tables, inspector dialogs, keyboard interaction and axe accessibility. Representative screenshots were visually inspected at 4K Summary/Risk, ultrawide Compare, 1440 Summary, tablet Risk and mobile Summary. Wider tables use available space; tablet matrices scroll within their region rather than widening the document.

## Validation and limits

| Check | Result | Evidence under `output/responsiveness/` |
| --- | --- | --- |
| Baseline build + screenshots | PASS: 8 viewport tests, 1 after-only skip | `build-before.log`, `before-tests.log` |
| Corrected production build | PASS; 12 Markdown twins generated | `build-after.log` |
| Lint and TypeScript | PASS | `lint.log`, `typecheck.log` |
| Frontend unit tests | PASS: 286 across 33 files | `test.log` |
| JavaScript budget | PASS | `budget.log` |
| New responsive matrix + reflow + scaling | PASS: 10 desktop tests | `browser-all.log` |
| Full browser run | 134 passed, 24 skipped, 4 screenshot/navigation timeouts | `browser-all.log` |
| Affected gallery rerun, one worker | PASS: all 6 gallery cases; includes all 4 failed cases | `gallery-rerun.log` |

There are 138 unique passing browser cases after the rerun, not a claim that the initial full run was clean. The four timeouts did not reproduce with one worker; assertions and timeouts were not weakened. The 24 skips consist of 14 existing demo/device/baseline skips plus 10 deliberate phone-project duplicates of tests that set their own viewport. Initial lint on generated trace-viewer assets failed; excluding only generated output fixed it.

Reflow tests use actual CSS viewports 720×450 and 360×450, representing a 1440px window at 200% and 400%. They check overflow plus opening the mobile menu and Escape returning keyboard focus. Display-scaling models use 2560×1440 at DPR 1.5 and 1920×1080 at DPR 2; both correspond to 3840×2160 backing pixels. Effective CSS viewport and DPR are recorded, rather than inferring layout from monitor resolution. Browser controls/taskbars can reduce the real available viewport.

Native browser chrome zoom and changing the physical Windows display-scale setting were **not** automated. These are still owner/manual checks; equivalent CSS reflow and emulated DPR do not prove those native controls. No hosted/production inspection or deployment is claimed. Visual approval remains the owner's gate.

## Review artifacts and reproduction

Open `output/playwright/responsiveness/index.html` to select viewport/page and compare images. Raw metrics are in `before/*-metrics.json` and `after/*-metrics.json`; scaling metrics and reflow screenshots are under `after/`. Machine-readable results and changed-source hashes are in `output/responsiveness/results.json`.

From repository-root PowerShell:

```powershell
$env:CELEBTIR_DATA = Join-Path (Get-Location) 'output/qa-v02-data'
pnpm.cmd -C site build
$env:CELEBTIR_TEST_PORT = '13221'
$env:CELEBTIR_LAYOUT_PHASE = 'after'
pnpm.cmd -C site exec playwright test responsiveness.spec.ts --project=desktop
```

Use `--workers=1` for the existing full-page gallery if parallel capture stalls. No protected source dataset needs rewriting to reproduce these checks.
