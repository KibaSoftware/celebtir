# CELEBTIR — HIGHWATCH PRODUCT & EXPERIENCE REDESIGN

**Implementation specification for Codex / product design / frontend engineering**  
**Version:** 1.0 — 21 October 2026  
**Status:** Proposed. Review required; **do not commit, deploy, or change production data**.  
**Scope:** Whole application — Solana, Hyperliquid, HIP-3, shared navigation, analytical pages, research pages, content and supporting routes.  
**Identity:** **Celebtir**, intended to evoke *Silver Watch*; institutional DeFi risk intelligence with a subtle Elvish Highwatch character.

> **Precedence:** This file supersedes the **visual layout, styling, information architecture and content-presentation** directions in earlier Celedir/Celebtir redesign prompts. It does **not** supersede established source data, methodology, scoring, API contracts, safe migration procedures, or the requirement for owner approval before committing.

> **Primary objective:** Transform an improved-but-uniform administrative dashboard into a clear, distinctive, evidence-led public research product. Borrow L2BEAT's information architecture discipline, prioritization and investigation pathways—not its exact visual appearance. Users should progress naturally from **finding → context → comparison → explanation → on-chain evidence**.

## How to use this file

1. **Audit before editing.** Read the repository, route tree, screenshot baseline, data contracts, approved logo assets and currently active methodology. This file is a design/implementation target, not permission to fabricate features or disregard the codebase.
2. **Preserve invariant facts.** Never alter scoring, Stage calculations, source values, on-chain evidence, controller logic, snapshot semantics or data contracts to make the interface look better.
3. **Design page compositions first.** Build the Summary prototype and shared components; validate the visual direction at real desktop and mobile widths **before** propagating it across routes.
4. **Implement in gated phases.** Keep the worktree uncommitted and provide a design review package with screenshots, changed files, testing results and known gaps.
5. **Use real coverage.** If a capability, time series, source or metric is absent, show a trustworthy scoped view or honest empty state; do not manufacture it.

---

# 00 — Success criteria and non-negotiables

## Product promise

A visitor should understand within seconds: **Who can change a protocol, what powers do they hold, how quickly can they act, which assets and value depend on them, what evidence supports the finding, and what has changed since the last observation?**

A successful interface:

- Feels **calm, legible, distinctive and authoritative** rather than relentlessly dense.
- Gives each page **one dominant question and one dominant analytical view**.
- Makes complex evidence easy to reach without forcing it into every summary row.
- Allows expert scanning across projects without misleading oversimplification.
- Makes metadata, coverage and source limitations visible without turning them into primary navigation chrome.
- Uses the brand sparingly and consistently; does **not** turn the product into a fantasy-themed game.
- Delivers accessible, responsive, high-performing layouts at desktop, tablet and mobile widths.

## Hard constraints — do not violate

- **Official brand: CELEBTIR** (not Celedir). Change user-facing copy, metadata, manifest and docs as appropriate; **do not rename APIs, storage paths, environment variables, or internal identifiers without safe migration**.
- Use the **already-approved silver/gold watchtower-star insignia**, sourced from the actual approved asset; do not invent a different mark. Primary mark is symbol-only. Optional separate serif wordmark may accompany it in navigation/communications. Prepare verified transparent and light/dark/favicon renditions; do not mistake a black-background raster for a transparent PNG.
- **No composite or aggregate risk/security score.** The six protocol factors remain independent. Stages are their own governance/maturity assessment and are **not** a security rating.
- **Risk ratings:** Low / Medium / High / Unknown / Not Applicable remain semantically distinct. Missing source data is not zero. Unknown is not Low; Not Applicable is not Unknown.
- Preserve methodology version and criteria from the **live repository**. Screenshot versions differ (v0.1, v0.2); treat the repository's active version as authoritative and label snapshots by their effective methodology version.
- Preserve existing six-factor protocol methodology, token four-factor methodology, evidence traceability, controller-resolution behavior and chain-specific semantics.
- TVL, token value and activity are **context**, not inputs to a risk rating. Don't infer that large = safe or small = unsafe.
- Never claim realtime data where the page is snapshot-based; respect chain clock, observation date, refresh lag, and provider coverage.
- No synthetic historical snapshots, invented deltas, fabricated protocol rankings, invented incident feeds, invented HIP-3 assessment criteria or fake evidence links.
- Maintain all existing routes, shareable links, search, filters, columns, compare, source downloads, keyboard navigation and application contracts. Add functionality only when the underlying data supports it.
- Do not copy L2BEAT's logo, illustrations, proprietary assets, wording, or pixel-level designs.
- **No commit, deployment, database mutation, destructive cleanup or mass deletion until owner review.**

---

# 01 — Baseline audit: what the supplied screenshots reveal

Screenshots reviewed: Summary, Risk Analysis, Value, Activity, Tokens, Compare, Methodology; supporting earlier screenshots also show the previous versions of Summary, Risk, Value, Activity, Tokens, Compare and Methodology. The global navigation shows a **HIP-3** route, but its internals were not shown; audit the actual implementation instead of assuming its contents.

| Area | Observed issue | Consequence | Required change |
|---|---|---|---|
| Global | Tiny visual scale inside a narrow centered region even on large desktop screenshots | Large monitors feel empty outside but crowded inside | Adaptive wide data canvas and a separate narrower reading canvas; scale type and whitespace within components |
| Branding | Approved insignia has little prominence; navy and blue dominate almost all surfaces; gold appears as tiny bullets | Highwatch feels like a paint job, not an identity | Intentional use of silver geometry, gold emphasis and brand proportion; fewer generic cards |
| Summary | Five near-equal KPIs, many near-equal navy blocks, two summaries of control exposure, crowded risk rows and long lists | No immediate insight or visual pacing | Primary editorial governance finding, supporting metrics, one strong composition chart, scannable leaderboard, compact findings |
| Risk | Many almost-identical Low/Medium/High pills; dense filters; no visible contextual reasoning until interaction | Rating can be spotted but not understood | Factor-specific explanatory labels, a readable matrix, contextual evidence inspector and accessible filters |
| Value | Control breakdown repeated as KPI, stacked bars, categories and table; historical chart absent due to insufficient Celebtir history | Repetition without progression | Snapshot-first exposure narrative, true time series only when valid, clearly separated rankings and drill-down |
| Activity | Four equal KPI cards; volume coverage visibly partial (3/12); two similar charts; tiny sparklines | Implies completeness and feels generic | Coverage-first context, better charts, comparison controls and differentiated analytical purpose |
| Tokens | One extremely long uninterrupted directory; tiny text; same red supply-control badges repeated; long accepted-by link lists | Fatigue, reduced discovery, risk-label confusion | Token overview + advanced directory, grouping/filtering, terse row summaries, issuer/permissions drill-down |
| Compare | Almost-empty landing screen; raw-looking suggested chips; vague instruction and large dead area; screenshot supplied twice | Weak first-use experience | Deliberate empty-state selection flow, curated compatible example pairs, intelligent selection, clear compare preview |
| Methodology | Very long single-column document with stacked explanatory panels and dense technical text; reduced readability in full-page capture | Difficult to navigate and audit | Research-document layout, sticky TOC, anchor navigation, per-factor chapters, criteria tables, examples, machine-readable exports |
| HIP-3 | Route exists; detailed screen not supplied | Risk of applying wrong generic design | Audit actual coverage; build dedicated deployer/market control lens within existing assessment scope |
| Supporting routes | Not all shown | Inconsistent experiences risk | Audit protocol, token, controller, Changes, Glossary, About, API, archives, error states and footer links |

## Root cause

The first pass optimized **component consistency** but not **information hierarchy**. Everything became a card, pill or table. Internal type is small, descriptions are numerous, and each section is given similar emphasis. The next iteration must redesign **relationships, rhythm, prominence and discovery**, not merely increase padding.

## Baseline artifacts to capture before coding

- Screenshots at **1440×900**, **1920×1080**, **1280×800**, **768×1024**, **390×844** for every major route; record current browser scale/DPR.
- Inventory of route names and deep links, API props, table column semantics, data statuses, analytics providers, chart libraries, logo assets, breakpoints and design tokens.
- A fixture of **actual, source-traceable sample data** (not fabricated) with cases for sparse history, partial provider coverage, Unknown, Not Applicable, no-delay controller, multi-chain scope and long names.
- Current active methodology and its exact factor names/thresholds; do not silently carry over obsolete screenshots.

---

# 02 — Experience strategy: editorial hierarchy instead of widget accumulation

## A repeatable page formula

Each major page should use these **roles**, not necessarily the same visual layout:

1. **Orientation:** clear title, one-line purpose, current chain and data scope.
2. **Primary answer:** one significant analytic statement or visualization answering the page's main question.
3. **Explanation:** supporting measures, comparison or distribution; summarize coverage.
4. **Exploration:** filterable matrix, ranked table, document navigator or protocol directory.
5. **Evidence:** methodology explanation, account identifiers, source links, timestamps, change logs and limitations.

Do not place five competing primary actions or five identically weighted KPIs above the fold.

## Visual emphasis budget

- Per viewport, generally **one primary visual**, at most **two clearly subordinate focal points**.
- A large area of bare navy background does **not** count as effective whitespace if the content is compressed.
- Not every section receives a card or border; use open sections for headline information and understated dividers for related content.
- Put short interpretation **adjacent** to data, not hidden in unrelated help text.
- The default view is the **most useful subset**; progressive disclosure exposes raw detail.
- The user should never have to read all six risk cells merely to understand that a protocol has **no upgrade delay**.

## Suggested page questions

| Route | Primary user question | Dominant treatment |
|---|---|---|
| Summary | Who can change the rules controlling assessed DeFi value? | Governance/exposure editorial overview |
| Risk Analysis | Which protocols have which control assumptions, and why? | Evidence-backed risk comparison matrix |
| Value | How much economic value sits behind each control arrangement? | Exposure composition chart plus TVL ranking |
| Activity | What reported economic activity exists, and how complete is coverage? | Time series with explicit coverage |
| Tokens | What authority exists over the tokens protocols accept? | Token explorer with permissions and exposure graph |
| HIP-3 | What powers do market deployers/operators have over listed perps? | Deployer/market research profile and control summary, only when data exists |
| Compare | How do two to four selected projects differ? | Structured, differences-first parallel research |
| Changes | What materially changed, when, and how was it observed? | Timestamped annotated change timeline |
| Methodology | How are findings derived and reproduced? | Navigable technical research document |
| Protocol detail | What is the effective control structure of this protocol? | Profile with control diagram, factor findings, timeline and evidence |
| Token detail | Who can mint/freeze/restrict a token and where is it used? | Authority and issuer profile with linked exposures |

---

# 03 — Visual identity: institutional Elvish Highwatch

The user-selected identity is **watchtower, wisdom and subtle Elvish elegance**: soft flowing lines, clean geometric proportions, institutional trust, bold enough to reproduce via print and embroidery. The website is an analytical instrument, not a Tolkien-themed entertainment product.

## Color tokens (initial values; test contrast in context)

| Token | Hex | Use |
|---|---|---|
| `canvas/primary` | `#080E1A` | Application body / large negative space |
| `canvas/secondary` | `#101A2B` | Subsections, global nav |
| `surface/raised` | `#17263A` | Major analytical panels only |
| `surface/hover` | `#20324A` | Hover, pressed and selected backgrounds |
| `border/subtle` | `#2A3A50` | Dividers and low-emphasis card edges |
| `border/focus` | `#6599E8` | Focus ring, selected data interactions |
| `text/primary` | `#F5F7FC` | Important headings, facts and numbers |
| `text/secondary` | `#C5CFDD` | Supporting copy / headings |
| `text/muted` | `#93A6BC` | Meta labels (contrast test required) |
| `brand/silver` | `#C5CFDD` | Geometric structure, brand marks |
| `brand/gold` | `#D4B46A` | Rare editorial/identity emphasis, selected highlight |
| `brand/blue` | `#6599E8` | Links, primary chart series and interactive selection |
| `brand/white` | `#F5F7FC` | Main information |
| `status/low` | `#35B780` | Protocol factor Low (semantic only) |
| `status/medium` | `#E6AE49` | Protocol factor Medium |
| `status/high` | `#EA6B72` | Protocol factor High |
| `status/unknown` | `#98A5B5` | Unknown / not enough evidence |
| `status/na` | `#6E7C90` | Not Applicable, with separate pattern/icon |
| `status/stage` | `#AA9BD9` | Stage badge distinct from risk colors |

**Do not equate green/red status colors with investment quality.** For activity deltas, avoid automatic "green is good" semantics unless the metric has a well-defined preference direction. Chart series colors are for data groups, not always risk.

### Color allocation and backgrounds

- Target **neutral/navy majority**, modest silver/white information layer, blue used for clickable/highlighted state, and gold used sparingly as a distinctive **anchor**.
- Silver lines and geometric edges can echo the approved logo without literal stars on every component.
- Avoid purple-dominant UI, excessive metallic gradients, gold-on-gold, glow, fake texture, background constellations under tables and ornamental scrollwork.
- Prefer **flat readable data colors** and crisp borders; allow one exceptionally subtle navy-to-sapphire ambient treatment in hero/branding area if it passes legibility checks.
- **Dark mode is the product default.** Ensure the official mark works on both dark and light backgrounds. If a light UI theme already exists (or is separately approved), derive it from semantic tokens with re-tested text/risk contrast; **do not add an unrequested theme-toggle project merely to support the favicon**.

### Typography

- Primary UI: legible contemporary sans (Inter, Geist, or existing suitable family); **no fantasy fonts** in data.
- Optional elegant serif on **Celebtir wordmark and selected editorial major headings only**; make a fallback-safe choice and confirm licensing. Heading typography should remain legible, not calligraphic.
- Data and addresses: tabular numerals; monospace only for hashes, addresses, slots, code and relevant precise technical identifiers.
- Suggested **desktop scale**: page title 32–40 px / 1.15–1.25; large KPI 38–52 px; section title 21–25 px; body 15–16 px / 1.5–1.65; table labels 13–14 px; micro/meta 12–13 px. Avoid 10–11 px explanatory copy.
- Long-form documentation: reading column ~65–78 characters; body 16–17 px / 1.65–1.8; headings well-spaced.
- Use figure/tabular alignment, consistent sign and unit formatting, and adequate contrast at normal 100% browser zoom.

### Space, shape and layout

- Use an **adaptive data canvas**: approx. 1320–1480 px max on wide screens, with at least 24–40 px gutters; dense research pages can use almost all available width. Keep narrative text narrower; don't make entire website a 900 px column on 2048 px screens.
- Base spacing scale: 4, 8, 12, 16, 24, 32, 48, 64 px. Between **major narrative sections** often 48–72 px; within grouped rows 12–24 px. Vary section rhythm intentionally.
- Card padding: 20–28 px for major modules; 12–16 px for compact table/tile widgets. Data-row target height ~44–56 px depending on density mode.
- Radius: 8–12 px for cards, 6–8 px for controls; avoid endlessly rounded pills everywhere.
- Silver thin dividers, measured asymmetric layouts and sweeping curves used sparingly in hero treatments are preferable to decorative imagery.
- Create a **compact/comfortable table density switch** only if useful for expert workflows; default must be comfortably readable.

### Brand and favicon

- Logo: official provided transparent source; maintain intrinsic aspect ratio and safe padding, never stretch/distort/recreate via generated substitutes.
- Navigation height supports a legible ~26–32 px symbol; use separate restrained serif “Celebtir” text where space permits. Avoid turning it into a huge fantasy header.
- Favicon 16/32/48 px: derive from the original insignia, simplify if necessary based on optical tests; verify on both light and dark tab backgrounds. Test actual raster alpha channel (outside must be transparent) and edge halos.
- Provide monochrome black/white mark only with designer-approved derivation suitable for print/embroidery; minimum stroke width must survive 16 px.

### Motion and feedback

- Short interactions ~120–220 ms; hover/focus/selection/folding should feel precise, not cinematic. Respect `prefers-reduced-motion`.
- No particle effects, parallax starfields or ambient animation. No animation to mask latency or data changes.
- Focus rings and mouse/keyboard/touch feedback should be consistent across charts, cells, tabs and command search.

---

# 04 — Navigation, routes and user journeys

## Global navigation

Preserve route names/deep links; optionally regroup visible nav while leaving direct destinations accessible:

- **Explore**: Summary, Risk Analysis, Value, Activity, Tokens, HIP-3 (when relevant to selection).
- **Research**: Compare, Changes, Methodology.
- Secondary: About, Glossary, Shared Controllers, Archived Protocols, API, Atom Feed, `llms.txt`, Report an Error.

On wide screens, show the primary areas in one restrained header. On narrower screens, group secondary sections into clearly labeled menus—not truncated microscopic text. Always expose active route and selected chain.

### Chain/system scope

- Clearly differentiate **All chains**, **Solana**, **Hyperliquid**. All-chains aggregates are valid only for comparable, de-duplicated denominators; disclose coverage and inclusion rules.
- Solana slot numbers are Solana-specific. Show appropriate Hyperliquid block/time/identifier when available; do not attach a Solana slot to a Hyperliquid-only claim.
- **HIP-3 is a Hyperliquid market/deployer feature, not a third blockchain**. It may have its own analysis destination but should not be misrepresented as a network selector choice.
- Filters that cross methodologies must not produce fake apples-to-apples risk comparisons. Display labels, applicability and method versions.

### Snapshot and provenance bar

Replace three prominent floating metadata pills with a concise factual line, visually secondary to the page title:

`Snapshot · 21 Oct 2026  |  Solana slot: …  |  Methodology v…  |  Coverage: …`

Use actual values dynamically and explicit timezone where time is involved. Full details and copyable identifiers live in an accessible popover. A stale-data banner appears only when the underlying age exceeds the documented freshness policy.

### Search and cross-linking

- Global ` / ` command search: protocols, tokens and supported controllers/markets; results grouped by object type and scope, with chain chips. Provide keyboard navigation, empty-state guidance, ESC close, clear search and URL links.
- Never promise search on address/issuer if not indexed. Distinguish matched symbol from matched name/address.
- All protocol names link to protocol profiles; token names link to token details; controller names/addresses link to entity/evidence. Preserve external explorer links.
- Breadcrumbs on deep research pages; sticky in-page section navigation on long protocol/methodology pages.
- URL-addressable filter/tab/selection state wherever it materially supports collaboration. Back/forward navigation restores state.

### Primary journeys (test explicitly)

1. Summary exposure finding → category breakdown → affected protocol → control permissions → verified account evidence.
2. Risk matrix High/Unknown cell → explanation → methodology threshold → underlying snapshot/account.
3. Token directory → asset permissions → issuer/mint/freeze authority → protocols accepting token → role in collateral/deposit.
4. Protocol profile → Add to Compare → choose second → differences view → shared URL.
5. Changes feed → prior/new observation → assessment diff → source record.
6. Hyperliquid / HIP-3 market → responsible deployer → controllable parameters and oracle responsibilities → evidence (only for supported datasets).

---

# 05 — Summary: flagship landing experience

**User question:** “Who actually has power over the protocols and value I am looking at?” This page is the emotional and analytical first impression; fix it before propagating design elsewhere.

## Specific screenshot problems to solve

- Five equal-sized KPI cards make “12 protocols assessed” nearly as visually important as the central economic control finding.
- The 71.4% figure shown for value without delayed/immutable control is buried in a modest KPI. It is potentially the page's most meaningful observed insight; **calculate it dynamically** and label its denominator and classification.
- “Value by upgrade control” and “Protocols by Stage” are boxed alike, although one is economically central and the other is supporting context.
- Risk distribution, 12-row leaderboard and four insight panels form an uninterrupted wall of containers. A panel listing many upgrade delays sits beside nearly empty ones.
- Summary risk icons are too compressed and dependent on a legend.

## Required composition (desktop)

```text
[Context: selected chain, dated snapshot, sample size]

[Editorial title: DeFi Risk Intelligence]  [Independent. Evidence-based. Transparent.]
[Single hero insight: e.g. exposure without delay]
[Dominant horizontal visual: exposure by control type]    [2-3 supporting figures]

[Protocol landscape: optional Stage distribution, 1-2 concise findings]

[Browse assessed protocols]                  [Sort / search / category / columns]
[Readable leaderboard: protocol, control finding, six factor summary, Stage, TVL]

[What deserves attention]                   [Recent verified changes OR data limitations]
[Evidence + methodology link]
```

### A. Editorial hero

- Brand headline: **“DeFi Risk Intelligence”**; supporting line: **“Independent. Evidence-based. Transparent.”** Keep chain scope contextualized separately. The 2026-10-27 branding update applies to existing metadata and share-card surfaces; it does not introduce a homepage hero or change the header/footer layout.
- One sentence: **“Independent analysis of who can upgrade protocols, change parameters and affect the value they manage.”** Adapt only to the actual methods and evaluated powers.
- Primary finding uses a large **number or control-type statement**, explanatory short title, numerator/denominator and transparent classification.
- Example **only if the current live data supports it**: “71.4% of analyzed TVL is governed by multisigs without a protective upgrade delay.” Do **not** write “71.4% is unsafe”, conflate administrative and upgrade delays, or imply all assets can be stolen.
- Major finding should drive to the Value exposure view and a filtered list of affected protocols.
- Show at most 2–3 supporting key figures, e.g. analyzed TVL, protocols analyzed, number of protocols with High upgrade-delay rating; annotate current snapshot.

### B. Primary visualization

- A large **horizontal composition bar** with percentages, amounts and legend (single key / unrecognized / no-delay multisig / delayed multisig / immutable), **if all classes are supported**.
- Include raw counts or amount alongside percentages; account for rounding and data coverage. Do not mix TVL-weighted percent and protocol-count percent without headings.
- Click a segment to filter the leaderboard and update the displayed context; keyboard equivalent mandatory.
- Provide an equivalent accessible text table.
- Stage breakdown should be clearly secondary, with explicit “Stages do not measure code security” explanation.

### C. Protocol leaderboard

Default columns: protocol identity + category; **control finding** (e.g. “5-of-10 multisig, 24h delay”); compact six-factor risks; separate Stage; TVL and relevant activity. Keep “latest change” optional if all rows are “none recorded”.

- Protocol names/logos visually anchor each row; distinguish loading/unknown logos from generic initials.
- Replace unreadable six-icon microscopic codes with **six recognizable factor slots**, stable ordering, labels on header and keyboard-accessible explanations. Short codes can be used in a compact mode but require header/tooltips and screen-reader descriptions.
- Avoid showing six fully spelled-out pills per row in Summary; detailed ratings belong in Risk. Optional mini-risk strips are acceptable if their meaning is discoverable.
- Upgrade-controller summary can be a meaningful short sentence rather than a hash-heavy cell.
- Default sort by analyzed TVL **only if documented**; sortable columns and persistent URL state. Provide a no-sort or alphabetical option.
- Surface metric freshness, provider absence, Unknown, Not Applicable and no-recorded-changes as **distinct states**.
- Users should be able to tab to and open full protocol profiles.

### D. Findings (replace four equal insight cards)

- Show **up to three relevant, non-empty insight modules**, ranked by editorial relevance determined with **auditable deterministic rules**, not subjective AI narrative.
- Possible modules: “Protocols with upgrade delays below policy threshold,” “Unrecognized/unknown controller exposure,” “High dependencies,” “Verified control changes.”
- Long lists must be collapsed to **top 3–5 with ‘See all’**; otherwise they dominate the page.
- If there are no events, use a restrained one-line “No recorded control changes in the displayed window” and a link; **do not reserve an empty full-height column**.
- Show why an insight is included and its data window. A recent change is not automatically a worsening risk.

### E. Snapshot/historical behavior

- When fewer than two compatible governance snapshots exist, **do not occupy a large hero block with a blank time-series panel**. The cross-sectional composition remains meaningful.
- Keep a concise coverage notice with exact requirement and current snapshot count.
- Historical trends should appear only when actual comparable observations exist, with methodology changes marked rather than plotted as performance moves.

**Acceptance:** Within five seconds at 1440px, readers can point to the main control-related exposure observation, understand its denominator, identify the main researched universe, and find a protocol to investigate.

---

# 06 — Risk Analysis: from badge grid to evidence-led comparison

**Question:** “Which control assumptions differ among protocols, what do they mean, and what is the observed evidence?”

## Default view

- Large clean page title and one explanatory sentence; short “six independent factors” eyebrow is optional, not a repeated decorative section label.
- A single **compact filter toolbar** with: category, selected chain, factor, severity, search, reset. Put advanced settings in one progressive-disclosure menu.
- Main **risk matrix** should have a sticky protocol column, sticky header, scrollable if necessary, readable ~44–52 px rows, stable factor column widths and inline column definitions on hover/focus.
- Ratings must be shown with a **semantic icon + label**, not six indistinguishable tinted tiny pills. Offer low-noise density; never color-only.
- Add a useful *finding descriptor* via cell detail or compact second line where it truly clarifies: “No delay”, “Verified deployment”, “Oracle operator controlled”, “Dependency unknown”. Do not derive these descriptors from rating alone: use the actual observed fact.
- Keep Stage visually separated (different color), never blend into the six-factor matrix.

## Interaction: clicking a cell

Open a right-side inspector on wide screens or an accessible dialog/expandable sheet on small screens. Must contain:

1. **Finding** (protocol, factor, rating, snapshot/methodology version).
2. **Observed state** — actual signer threshold, delay, authority, verification status, oracle source or called-program dependency as relevant.
3. **Why the criterion applies** — exact methodology condition in current version; readable concise excerpt with deep link.
4. **What would change the rating** — specific necessary change, conditional on applicable threshold, not generic advice.
5. **Evidence** — account/program address, slot/block, source transaction/build/commit as applicable, explorer and copy affordances.
6. **Limits** — unresolved reads, sampling gaps or assumptions.

Avoid a tooltip-only experience for serious evidence. The inspector must support deep links and be keyboard-accessible; closing it restores focus.

## Matrix intelligence

- Filters for one factor and rating can optionally display a one-sentence cohort interpretation, e.g. “9 of 12 protocols have High upgrade-delay ratings,” computed from current dataset.
- Offer a **summary row or compact chart** above the matrix for rating distributions; not a second giant chart duplicating the exact matrix.
- Unknown and Not Applicable each have explicit distinct legends and help text. Clarify when denominator excludes non-applicable cases versus counts them in the total.
- Do not color full rows red based on a single High factor.

**Acceptance:** A user can find every High upgrade-delay rating, open any cell and reach an on-chain address plus the precise active methodology rule within two additional interactions.

---

# 07 — Value: make economic exposure the story

**Question:** “How much value is exposed to each controller arrangement, and where does that exposure sit?”

## Page order

1. Title + coverage: which protocols, data provider(s), observation date and any exclusions/overlap caveats.
2. **Main exposure breakdown** — largest element. Use segmented bar / treemap-like allocation only if accurate and accessible. Explicitly label both $ amounts and share of *analyzed* TVL.
3. Supporting metrics: analyzed TVL, largest protocol share, delayed vs no-delay sums. Avoid repeating every numeric result in three equivalent cards.
4. **Historical TVL** if sufficient compatible observations exist; if not, honest narrow notice and stronger single-snapshot view.
5. Protocol ranking: identity, category, TVL, share, controller class, control summary, available delta/sparkline.
6. Asset-group composition: stablecoins, SOL/staked SOL, BTC, other, etc., **only if classifications and coverage exist**. Show exclusions explicitly; may be in a secondary tab/section.

## Presentation rules

- Controller categories must have **distinct restrained chart colors or patterns**; the current predominantly all-blue bars make categories hard to read. Keep gold for emphasized analytic choice, not an automatic hazard rating.
- Provide filters by protocol category/chain and a “show as table” view for each visualization.
- Show denominator, unit (USD), source, aggregation methodology, timestamp. If comparing Solana and Hyperliquid data, disclose differing observation clocks/providers and avoid double counting.
- Do not describe TVL as “value secured” unless the product's definition justifies that exact term.
- Do not treat a TVL aggregate containing overlapping assets/protocol deposits as unique economic exposure if double-counting is possible. Use “summed reported protocol TVL” and clarify overlap if necessary.
- Positive/negative change colors should not imply improving/declining security.
- Prioritize legible labels; hover-only exact values are insufficient. Make narrow bar segments keyboard-accessible or provide a connected table.

### Historical series

- Timescale choices must adapt to actual dates: e.g. 7D/30D/90D/1Y/ALL only when present, or disabled with explanation.
- The 30-day DefiLlama history must **not** be relabeled as 30 days of Celebtir risk history. TVL provider history and governance-snapshot history are different.
- Preserve real gaps in data; no invented interpolation. Where a value uses an as-of previous observation, call that out precisely.

**Acceptance:** The page has one unmistakable exposure visualization and does not restate the same numbers in three equal-weight modules.

---

# 08 — Activity: readable charts and transparent coverage

**Question:** “What measurable protocol activity is reported, when did it happen, and how complete is it?”

## Top composition

- One clean dashboard header with the selected chain, data source and time window.
- Lead with **daily volume and fees**, but display coverage next to each metric as first-class text, e.g. **“Volume: 3/12 protocols reporting”**, not tiny tooltip-only metadata.
- Clarify whether aggregate counts cover all assessed protocols or only reporting ones. When provider plan lacks perpetual volume, do not imply 12/12 coverage of volume.
- Use two charts in side-by-side panels at wide resolutions, full-width stacked on smaller screens. Give each ~300–400 px meaningful plotting height when possible.
- Time-range control is one shared accessible segmented group; the charts must label if they show different available date spans.

## Chart improvements

- Clear chart titles, current value, unit labels, readable axes, crosshair/tooltip with exact timestamp and scope, missing-point handling, color-blind-safe marker/style and tabular fallback.
- Keep gridlines sparse; do not use strong panels and eight legends if two data series suffice.
- Use blue for primary series; muted silver/gold or alternative line pattern for comparison. Stronger hover line, cursor and selected date.
- Do not smooth or interpolate sparse source records to create visual momentum; preserve discontinuities.
- Display dates with relevant provider timezone; do not assume snapshot slot determines DefiLlama observation day.

## Protocol table

- Columns: protocol, supported 24h volume, supported 24h fees, 7-day comparison when valid, sparkline where enough observations exist, coverage/source. Keep sort active and explain metrics in concise header tooltips.
- Sparklines need accessible numeric equivalents and honest blank/Unavailable states. Never draw a zero line for unavailable data.
- Move less important source explanations to details, not every row.

**Acceptance:** At a glance, users know the size and **incompleteness** of reported activity and can inspect a source-accurate time series.

---

# 09 — Tokens: turn the endless listing into an asset authority explorer

**Question:** “What can token authorities do, and which protocols depend on these assets?”

## Problems in current screenshot

The page shows many consecutive tokens (liquid staking assets, bridged assets, stablecoins and others), a repeating High supply-control badge, four authority columns, and **very long lists** of accepted-by protocols. It is useful raw coverage but lacks overview, meaningful grouping, visual rhythm and fast paths to investigate.

## New structure

1. **Header:** “Token authority & provenance” or existing “Tokens” title; explain these ratings describe token-level powers, **not price risk, solvency, peg stability or investment quality**.
2. **Compact overview** based on actual data: tokens tracked, token types, count with freeze/seizure powers, count with transfer restrictions, issuer/authority coverage. Avoid simplistic “N risky tokens” totals.
3. **Top navigation/filters:** search symbol/issuer/mint; chips for type (liquid staking / bridged / stablecoin / governance / other) and authority factor; severity and accepted-by-protocol filters; reset; count of visible results.
4. **Directory** with sticky header and sensible row height. Desktop columns: token identity/issuer, type, four factor ratings, used by; optionally provenance or chain as supported. Provide a column chooser for experts.
5. **Token detail** via accessible row expansion or dedicated route; don't force all explanation into directory rows.

## Directory row specifics

- Token identity shows **symbol and issuer** without excessive tiny address strings. Mint/address is available on expansion with copy and explorer links; treat address collisions and duplicate symbols explicitly.
- Avoid the word “High” repeated as the whole story: short evidenced descriptors such as “Mint authority retained” or “Freeze enabled” are available on expansion or cell inspector. High rating alone does not prove malice.
- “Accepted by” shows **up to two protocol links plus `+N more`** in compact mode; clicking shows complete list with roles (collateral, borrowable, deposit, LP) and links. The SOL/USDC rows must not expand to nine uncontained lines by default.
- Support view grouping by **asset type**, **issuer** or **authority classification**, with visible counts; group headers must not imply risk equivalence.
- Use pagination or virtualization if enough rows warrant it; preserve keyboard navigation, screen-reader row counts, browser-find behavior where feasible and stable scroll position.
- Where token metadata is unknown, display Unknown distinctly. Do not silently classify issuer-controlled tokens as malicious.

## Token detail information design

**Identity:** name/symbol, issuer, token program/standard, chain, verified mint/asset ID, provenance and source.  
**Four independent factor findings:** Supply control; Freeze and seize; Transfer restrictions; Provenance. Each includes observed authority/configuration, rating, rationale and exact evidence.  
**Exposure relationships:** protocols using token, role per protocol, and any supported amounts. If amounts are unavailable, avoid invented exposure diagrams.  
**Changes:** observed authority changes, with previous/new snapshots if supported.  
**Method:** token-specific rating criteria, scope limits and discrepancy-reporting link.

**Acceptance:** A user can find a stablecoin, identify whether an issuer retains freeze authority and navigate to every assessed protocol accepting it, without scrolling a 25-row directory or decoding badges.

---

# 10 — HIP-3: a first-class Hyperliquid research lens

**Important:** The header includes “HIP-3”, but no HIP-3 page screenshot or backing schema was supplied. **Audit the actual route, datasets and methodology before implementation**. The following is an IA target, not authorization to invent a scored product.

HIP-3 denotes **builder-deployed perpetuals on Hyperliquid**. Official Hyperliquid documentation describes deployer responsibilities for market definition, oracle/configuration and operation. Unlike a generic Solana lending protocol, the relevant research objects can be deployer, DEX, market, oracle and controls. See reference links in Appendix.

## What this page should answer

- What builder/deployer is responsible for each covered market/DEX?
- Which parameters can they control, e.g. oracle updates, leverage/margin configuration, market halt/settlement, relevant caps, depending on currently supported chain capabilities?
- Where does a market depend on deployer-operated data or permissions?
- What actual changes are recorded, and what source verifies them?

## Proposed layout, gated by available evidence

1. Intro: define HIP-3 in one or two sentences, clearly label Hyperliquid scope and coverage.
2. Current covered deployers/markets: **counts only if verifiable**; do not call uncovered markets safe or unrated as automatically High.
3. A **deployer/market directory**: operator, market/DEX, supported collateral/underlying, selected controllable powers, source/snapshot, coverage. Filter by deployer and market.
4. A detail panel/profile: **Control authority → Capability → Configuration → Observed change → Evidence**. Use a compact visual hierarchy, not an ornamental control diagram with fake connections.
5. Historical changes table/timeline if records exist.
6. Methodology or research-scope explanation: clearly distinguish **observed powers**, **risk assessments actually supported**, **unscored metadata**, and **currently unobservable fields**.

## HIP-3 boundaries

- Don't use Solana's six-factor scoring, Squad thresholds or slot semantics unchanged for HIP-3 unless the established project methodology explicitly justifies it.
- Don't treat HIP-3 as a chain, an ordinary token, or a generic protocol category.
- Don't claim live control or oracle state from cached responses; timestamp every reported observation.
- Avoid hard-coding protocol parameter values or requirements in UI copy: official Hyperliquid capabilities can evolve. Read actual on-chain/config data and active documentation.
- If no robust HIP-3 assessments exist, show a useful transparent **market-control directory** with no fake Stage/rating chips and an explicit coverage statement.

**Acceptance:** A Hyperliquid researcher can identify supported deployer authorities and verify their source without mistaking unscored data for a completed security assessment.

---

# 11 — Compare: fix the empty starting experience

**Question:** “How do two to four chosen projects differ on specific controls, evidence and activity?”

## Current failure

The supplied Compare screenshot shows a title, search bar, “show differences” switch, awkward suggested text-like buttons, a short instruction, and then a large empty navy region. This is the most conspicuous *interaction design* problem of the newly supplied pages. The screen feels unfinished even though the comparison engine may work after selections.

## First-load / zero-selection design

- A proper **guided selection panel** occupies the primary reading region—not a tiny toolbar followed by an empty page.
- Large succinct copy: **“Compare how protocols are controlled.”** Then “Select 2–4 protocols to inspect each independent factor, controller and evidence side by side.”
- Two clearly labeled protocol selector slots or an autocomplete **“Select first protocol” / “Select second protocol”**, prefilled only if arriving from a selection context.
- Under selectors, show **real, eligible comparison suggestions** from loaded data, grouped by sensible category (e.g. two available lending projects), with logos and compact descriptors. Do not hard-code dummy comparisons or present unstyled gray debug chips.
- Offer an optional “Start with an example comparison” link only if a real pair and complete data exist.
- Disabled primary Compare action must explain what is missing; ideally comparison updates automatically as soon as two selections exist.
- Show **no fictional preview chart** in the empty state. Use concise illustrated geometry (CSS, not a fabricated logo) or helpful instructions if design needs balance.

## Selected comparison design

```text
[Compare protocols]                     [Show differences only] [Share link]
[Protocol A header]         [Protocol B header]       [+ Add up to 4]
[Category/chain/Stage/TVL metadata aligned at top]

[Section: Effective upgrade control]
[Controller type / signers / delay / effective authority]
[Section: Six independent risk factors]
[Each factor: rating + short factual observation; expand to rationale and evidence]
[Section: Dependencies / token exposure]
[Section: Economic and activity context — coverage qualified]
[Section: Upgrade history / methodology limits]
```

- Sticky protocol headers on scroll, aligned columns for 2–4 protocols; no uneven widths caused by long source links.
- **Difference highlighting** should identify differences in *rating and underlying observed fact* independently. “No differences” must not imply identical evidence.
- Show factor rating + **meaningful short explanation**, then expandable evidence. Avoid forcing users through dozens of full raw paragraphs side by side.
- Control diagrams/addresses can live in foldouts or a research inspector.
- Support same-chain and cross-chain comparisons only where metrics and methods are compatible; visibly flag incomparable rows as Not Comparable (do not misuse Not Applicable if methodology defines it differently).
- Preserve selection in shareable URL; add/remove reorder controls, keyboard focus, clear state, robust browser back/forward.
- If requested pair has one unrated protocol or differing methodology versions, present coverage/compatibility notes and compare available facts separately.
- Put potentially 100+ lines of raw technical evidence behind clear detail expanders; avoid repeating entire protocol pages in a grid.

**Acceptance:** Zero-selected, one-selected, two-selected, four-selected, invalid/deleted selections and shared URLs all feel intentionally designed. A visitor can create a useful two-protocol comparison without hunting for an obscure tray.

---

# 12 — Changes: a usable audit and event discovery interface

**Question:** “What changed, and which recorded observations prove it?”

- Primary layout: chronological feed/timeline + compact filter sidebar or horizontal toolbar depending viewport.
- Filters: chain, event type (upgrade controller, delay, admin authority, dependencies, token authority, HIP-3 control, Stage/rating), protocol/token, date interval; only expose event types supported by data.
- Each event: **affected object, field changed, previous → current value, when first observed, methodology version, confidence/source, evidence link**.
- Distinguish **on-chain event time**, **snapshot detection time**, and **report publication time**. Do not call a detected difference an exact transaction-time event if only daily snapshots exist.
- Ratings changing because the **methodology version changed** must be clearly marked differently from an underlying on-chain configuration change.
- If the provider's history is filling, state the oldest recorded date. “No events recorded” is not “No upgrades ever.”
- Collapsible grouped events prevent a long list of repeated transaction IDs from dominating the timeline.
- Chronological pagination, deep links to an event, export/source references where supported.
- Empty state explains scope and observation window rather than promising monitoring that does not exist.

**Acceptance:** Every displayed “change” is sourced to a verifiable prior/new observation and clearly states whether it is detected, confirmed on-chain, or a methodology revision.

---

# 13 — Methodology: turn a long scroll into a credible research manual

**Question:** “How exactly did Celebtir reach this assessment, and could another analyst reproduce it?”

The supplied full-page screenshot suggests a long stacked document with version header, principles, cards, factor sections and technical content. A long document is appropriate; a **single undifferentiated vertical scroll is not**. Avoid shortening or hiding essential rigor simply to appear minimal.

## Desktop layout

```text
[Methodology vX.Y | Effective date | Version history | Download/read raw config]

[Left sticky TOC, ~220px] [Reading column, ~720–850px] [Optional small "On this page" annotations]
  Overview                   Purpose, scope and exclusions
  Core principles            Evidence / weakest-link / effective controller / unknown / recomputation
  Rating model               Six protocol factors; Stage evaluated separately
  Factor 1                   Question → rule tiers → worked examples → evidence
  Factor 2                   ...
  ...                        ...
  Token methodology          Four authority/provenance factors
  Hyperliquid / HIP-3        Dedicated criteria IF formally adopted
  Source & reproducibility   Snapshot / provider / stale source / algorithm / limitations
  Version history            Diff to prior versions
  Glossary / FAQ             Cross-linked
```

## Presentation requirements

- Sticky, collapsible table of contents with active section tracking, anchor IDs, copy-link actions and keyboard support. On mobile, a **“Contents”** dropdown or drawer; preserve document order.
- Optional search within the methodology document (client-side if content local).
- Keep **definitions, thresholds, criteria and exceptions visible and scannable**, using tables where comparisons genuinely help and textual explanations where nuance matters.
- Each factor chapter must contain: **question**, **what is observed**, **Low/Medium/High logic**, **Unknown**, **Not Applicable**, **effective controller interpretation**, **exception cases**, **what would improve the finding**, **example**, **evidence/source format**, and links back to affected protocol views. Add only distinctions that actually exist in the scored methodology.
- Stages get a visually separate chapter. Explain **what Stages measure**, unmet requirements and why Stage does not stand for code security. Avoid L2BEAT Stage criteria by default; use Celebtir's actual rules.
- Preserve precise “weakest core program” and effective-controller semantics if they remain part of active methodology; distinguish assumption and actual read state.
- Version information: effective date, superseded versions, immutable versioned configuration/source, plain-language changes, and impact on historical comparability.
- Source data/reproducibility section: how account state is read at chain slot/block, when/how facts are snapshotted, data sources, calculation and known limitations. **Never claim historical replay is possible where the chain RPC cannot provide past state**.
- Add method-to-UI crosslinks: from any risk inspector directly to the relevant methodology subsection and back.

## Document styling

- Main reading column ~65–78 characters; 16–17 px body, generous leading and paragraph spacing. Avoid microtext for full technical paragraphs.
- Use readable callouts for **Definitions**, **Exceptions**, **Limitations**, **Evidence**; only a small percentage of content should be inside dark cards.
- Long code, hashes and TOML snippets inside horizontally scrollable code blocks with copy buttons; never overflow layout.
- Tables may scroll horizontally on mobile but must preserve row headers and access to all rules.
- Glossary hover/links should define technical terms, but the document must remain understandable without hover.
- Typographic hierarchy: H1 per page, H2 main chapters, H3 factors/subtopics, H4 examples. Never use arbitrary font changes to imply level.
- Print-friendly CSS for research review; print page has dark ink on white, visible source URLs/footnote equivalents, no clipped code/tables.

## Content tone

- Define terms once, refer consistently. Prefer **“The on-chain record shows ...”**, not **“This protocol is safe/unsafe.”**
- Short introductions followed by full exact rules—not sensational headlines and not a wall of methodology boilerplate before the first useful concept.
- Add one worked example per type of complex rule, explicitly identified as **illustrative** if not pulled from a verifiable current protocol.
- A visible notice explains methodology version changes and how to report evidence errors.

**Acceptance:** Users can jump from a High upgrade-delay assessment to the exact criterion, understand it without reading the full document, and verify how the score was derived.

---

# 14 — Protocol profiles: the actual research destination

**Question:** “What controls this particular protocol, what could happen, and why does Celebtir say so?”

Do not stop at improving summary tables. Protocol profiles should be the product's most credible research artifact.

## Above the fold

- Protocol logo, name, chain, category, external official/docs/explorer links, assessment scope and **as-of** time/slot.
- Primary **effective control summary** in readable language, e.g. “A 5-of-10 multisig can authorize upgrades after a 24-hour timelock,” only if the actual control graph supports that statement. Otherwise explicitly “Controller unresolved”.
- Stage separate from factors and TVL; clear “What this Stage means” link.
- A narrow six-factor overview strip with text labels, not a single risk score.
- One main task: **Inspect effective controller** or **View evidence**, plus secondary “Compare”.

## Deep sections

1. **Control architecture:** effective controlling account(s), upgradeable program(s), multisig signers/threshold, timelocks, admin powers, fallback/recovery paths and authority chain. Diagram may be built only from verified relationships; display unknown nodes visibly.
2. **Factor analyses:** six independent assessments with fact → criterion → source → limitation; contextual changes from prior snapshots.
3. **Stage requirements:** met/unmet/unknown requirements individually; don't conflate unknown with failure.
4. **Economic context:** current reported TVL/activity with source coverage and “context-only” explanatory note.
5. **Token dependencies:** accepted assets, roles and issuer-level control exposures with links.
6. **Upgrade and authority history:** verified changes as available, not “0 upgrades ever” when history is incomplete.
7. **Technical evidence:** account/program IDs, transaction links, build/source verification, config excerpts, snapshot/methodology version, copy and download where supported.
8. **Scope and limitations:** unsupported chains/proxy programs, ambiguous authorities, provider gaps and stale evidence.

### Control diagram rules

- Graph nodes: accounts/programs/controllers/timelocks/oracle owners/other called programs, **only if the dataset actually records those objects**.
- Edges explicitly labeled “can upgrade”, “owns authority”, “requires M-of-N”, “controls parameter”, “calls”, etc.; don't imply risk from graph distance alone.
- Provide accessible textual equivalent of every graph; small screen displays a readable relationship list if graph would become microscopic.
- Never draw decorative links where real controller reachability is unknown.

### Long pages

Use a sticky in-page TOC, collapsible raw source blocks and meaningful segment headers so large project reports feel navigable—not a three-page unbroken sequence of identical navy cards.

**Acceptance:** A researcher can reach an evidence account or transaction from a specific finding, and copy the precise observation identifier with no ambiguity.

---

# 15 — Supporting routes and system experiences

## Shared controllers / permissions graph

- List controllers with identity, supported chain, type, affected protocols and proven capabilities. Allow filtering by scope and known controller.
- Distinguish **shared signer** from **shared effective control**; avoid inferring common operational control merely from an address name or similar branding.
- Graph is optional and must be generated solely from sourced relationships; always provide table alternative.
- Show chain, on-chain proof, slot/block and observation limitations.

## Archived protocols

- Preserve research and previous snapshot facts with **archived** status, last observed date, reason if known and current availability limits.
- Don't silently remove archived protocols from deep links; show archive badge rather than confusing 404.
- Exclude from active totals according to documented aggregation rules and show counts.

## About, glossary, API, feeds and `llms.txt`

- **About:** mission, methodology independence, coverage, limitations, contribution/error-reporting routes, team/organization facts only if verified.
- **Glossary:** alphabetical/searchable accessible definitions of timelock, effective controller, Stage, oracle, multisig, mint authority, Not Applicable, chain-specific terms; deep-link terms from risk inspections and methodology.
- **API:** public endpoints/schema/version/auth/rate limits/examples **only if existing**. Present readable code blocks and tested copy buttons. Do not assert guaranteed availability or SLA without facts.
- **Atom feed / Changes:** document exact event scope and detection window; feed link must point to an actual valid resource.
- **`llms.txt` / Markdown research exports:** optional advanced discoverability only if already supported or deliberately implemented; do not turn raw research into unvalidated automated claims.
- **Feedback/error reporting:** short reachable form/link, source context, protocol, snapshot and affected field; no dead mailto/inert buttons.

## Universal system pages

Design loading, empty/no results, stale data, partial coverage, API/network error, unknown chain, unsupported route, archived content and 404 with consistent helpful language and recoverable actions. Footer should be quiet, correct and readable—not tiny compliance boilerplate impossible to distinguish.

---

# 16 — Copywriting and information design rules

**Voice:** restrained, analytical, concise, explanatory. Never dramatic, patronizing, investment-promotional or excessively technical in primary headings.

## Language patterns

| Avoid | Prefer | Why |
|---|---|---|
| “This protocol is secure” | “Code transparency: Low risk — deployed build verified against source” | Rating measures one condition, not all security |
| “Unprotected $6B” | “$6B reported TVL behind controllers with no defined upgrade delay” | Separates exposure from exploitability |
| “No changes” | “No control changes recorded during the observation window” | History may be incomplete |
| “0 volume” for missing | “Unavailable — provider does not report volume for this protocol” | Missing ≠ zero |
| “Unsafe token” | “Mint/freeze authority retained by issuer” | Names observed power |
| “Stage 0 High risk” | “Stage 0 · Factor ratings shown separately” | Different frameworks |
| “Live snapshot” | “Snapshot captured at [date/time/slot]” | Accurate freshness |
| “All protocols” when subset only | “12 assessed protocols in the selected coverage universe” | Correct denominator |

## Rules per information element

- Every statistic has: **unit, source, timestamp/as-of, population, numerator/denominator or coverage when appropriate**.
- Every chart has: one-sentence headline, Y unit, X scope, accessible legend, tooltip, readable data table, explicit missing-data convention.
- Every severity has: semantic icon/color, label and short observed fact; deeper explanation accessible.
- Every table: title explaining why it exists, useful default sort, stable headers, sensible date format, search/filters when warranted, blank-state treatment, data source and export if available.
- Every link: descriptive accessible text; external links indicated and vetted; truncated addresses copy full original value.
- Every abbreviation: introduced or glossary-linked at first meaningful use; avoid six codes in a single unlabeled heading.
- Every absence: choose among **zero, no observation, not applicable, unknown, unavailable from provider, still indexing, outside coverage**, not one generic dash.
- Avoid prose paragraphs in the main data table; move source explanations to inspectors or details.

## Precision and representativeness

- Prefer decimal/precision appropriate to scale ($8.43B, $26.3M, $140.85K), with exact values in tooltip/source; don't convey precision beyond source reliability.
- Positive/negative indicators must state comparison period and whether comparable sample coverage changed.
- If time-series coverage grows or shrinks, warn before comparing totals unadjusted.
- Never compare denominator-incompatible ratios under one color scale.
- If color semantics conflict with brand (e.g. red High versus gold emphasis), meaning always wins over brand aesthetics.

---

# 17 — Data contracts, provenance and semantic integrity

Before changing React/Vue/Svelte components (identify the actual stack first), write or reuse a single **presentation-facing, typed semantic layer** that preserves existing API values while carrying their meaning.

## Required value distinctions

A metric/rating should carry, when supported:

```text
raw_value | formatted_value
measurement_unit | definition | source | provider
scope: chain, category, object IDs, inclusion/exclusion
observed_at | detected_at | data_window | slot_or_block
methodology_version (for scored assessments)
state: reported | true_zero | missing | unavailable | unknown | not_applicable | stale
coverage: observed_count / eligible_count
proof: address, transaction, program, code hash, source URL as relevant
```

**Illustrative shape only** — adapt to existing repo types; do not replace working contracts merely to match this example.

### Data policies

- Never coerce `null`, absent, NaN, unsupported or `undefined` into numeric zero during formatting/charting.
- Differentiate *provider unavailable* from *on-chain unknown* and from *methodologically Not Applicable*.
- Risk ratings must not be rederived from frontend presentation logic; UI consumes authoritative scored outcome and supporting observed fact.
- A Stage of **Unrated** is neither Stage 0 nor High risk. Unknown factor data may cap a Stage if documented; use active methodology.
- All “High” counts specify factor and denominator; avoid generic “high risk protocols” if factor-specific High is the only supported claim.
- When chain selections change, clear incompatible protocol/filter state gracefully and warn before dropping selected comparisons.
- When displaying multiple chains together, label snapshots by chain and as-of, and avoid applying a single chain's slot to entire rollup.
- Treat provider-sourced market data and on-chain program-state data as different observation streams with different freshness and precision.
- Empty event history does not prove absence of past events; label lookback and effective starting date.
- If code verification and public-source availability are distinct, preserve that distinction in copy.

### Reproducibility path

For each factor entry, trace UI field → normalized observation → source record → scoring rule/version. Preserve stable IDs for protocol, chain, token, controller, assessment and event where available. Design evidence URLs to remain accessible across UI revisions; never use ephemeral autogenerated array index as a permanent record key.

### Handling dynamic text

Technical evidence, protocol/asset metadata, on-chain identifiers and Markdown should be rendered with existing safe escaping/sanitization. External URLs must be validated; no arbitrary HTML injection from indexer or provider data. This is especially important in evidence inspectors, embedded method documentation and search results.

---

# 18 — Responsive experience and density policies

Use responsive **content strategy**, not just a column grid collapse.

| Width | Primary behavior |
|---|---|
| ≥ 1600px | Expand analytical canvas sensibly, make main chart/visual a clear focal point. Do not leave most width unused while table text is microscopic. |
| 1280–1599px | Full research canvas, one- or two-column major visualizations, sticky headers where justified. |
| 1024–1279px | Reduce nav clutter; reorganize secondary tools; keep charts readable and table controls concise. |
| 768–1023px | Stack secondary analytics, use accessible horizontally scrollable matrix/table, sticky first column when reliable. |
| < 768px | Top route menu, vertical metrics, full-width charts, drawers for evidence, searchable compact list with row expansion instead of squeezing 10+ table columns. |

### Mobile specifics

- **Summary:** headline → main exposure bar → supporting figures → protocol card/list view; no miniature 12-column table.
- **Risk:** choose factor to browse; protocol list with a meaningful badge/descriptor; a tabular matrix may remain accessible via horizontal scroll.
- **Value/Activity:** vertically stacked charts with readable labels/touch targets and data-table fallback.
- **Tokens:** search and filter prominently, token rows show identity and most relevant permissions with full four-factor details on open; accepted-by details are collapsed.
- **Compare:** 2-protocol view can use aligned stacked comparison rows; 3–4 protocols can use horizontal scroll with sticky headings, never microscopic columns.
- **Methodology:** reading-column-first, sticky/dropdown contents, horizontally scrollable tables and code blocks, printable full text.
- **Protocol detail:** effective-controller story first, followed by tabs/sections and on-demand evidence.

Touch targets target ≥44×44 CSS px where practical; preserve mobile zoom. Avoid horizontal body overflow. Browser back, focus restoration, deep links, tooltips, drawers and charts must work on touch and keyboard.

---

# 19 — Accessibility and inclusive semantics

Target **WCAG 2.2 AA**, acknowledging that an actual audit is required to substantiate conformance.

- Contrast ≥4.5:1 for normal text and ≥3:1 for large text; non-text UI boundaries and chart markers should satisfy applicable non-text contrast requirements. Test actual color-on-surface combinations in the current dark UI and in any existing/approved light theme.
- Semantic landmarks (`header`, `nav`, `main`, `footer`), logical H1→H2→H3 hierarchy, descriptive page titles and skip-to-content link.
- Every risk/icon badge must have a machine-readable full factor + rating; color and glyph alone never communicate essential meaning.
- Tables need `caption`, explicit row/column headers, logical sort state, descriptions for complex column groups and readable screen-reader traversal.
- Keyboard operation for search, filters, legends, tabs, charts, export, comparison, dialogs and evidence inspector; visible focus; focus trapping/restoration in dialogs; no keyboard traps.
- Charts must have accessible summaries plus underlying value tables. Pattern/line style differences for visually adjacent categories if colors are similar.
- Alerts about stale/partial data should be announced appropriately but not continuously; skeletons should not steal focus.
- Motion honors reduced-motion and avoids flashes.
- Text scaling at 200% and reflow at narrow viewport without loss of information.
- Dates, percentages, currency and program addresses exposed to assistive technology with coherent labels, not just abbreviations.
- RTL and localization are not requested; avoid implementation patterns that make future localization unnecessarily hard (hardcoded concatenated strings).

---

# 20 — Interaction patterns and shared component system

## Recommended shared primitives (adapt to actual framework)

| Component | Role and must-have behavior |
|---|---|
| `AppShell` / `TopNavigation` | Layout, responsive nav, chain selector, active state, global search |
| `SnapshotContext` | As-of, chain-specific slot/block, coverage and methodology state |
| `ResearchPageHeader` | Title, purpose, optional narrow contextual note, consistent but **not identical page composition** |
| `KeyFinding` | Dominant conclusion with exact denominator and deep link, *not generic promotional hero* |
| `MetricDisplay` | Figure, unit, as-of, population, missing status and tooltip |
| `RiskRating` | Rating/color/icon/label + factor and evidence affordance |
| `StageLabel` | Separate stage semantic color and explanation |
| `ProtocolIdentity`, `TokenIdentity` | Logo fallback, verified name, chain/category, links |
| `ControllerSummary` | Short human explanation with signer threshold and actual delay |
| `FilterBar` | Responsive essential filters, reset, URL sync, advanced disclosure |
| `DataTable` | Sort, pagination/virtualization when useful, accessible headers, column customizer, empty states |
| `ResearchInspector` | Fact → reason → source → limitation with stable deep-link state |
| `ChartFrame` | Title, unit, coverage, legend, data table, as-of, time window |
| `CompositionBar` | Mixed categories with exact weighted denominator and accessible table |
| `EvidenceLink` | External target safety, copyable ID, accessible label |
| `ChangeEvent` | Previous/current, observed/detected time, source and method note |
| `SelectionCompare` | Multi-object selection, stable URL, compare modes, focus management |
| `DocumentationTOC` | Active anchors, nested heading tracking, mobile menu and print support |
| `StateView` | Honest loading/error/empty/partial/unknown/unavailable/stale variants |

**Do not force all page sections through a single card template.** The shared system should ensure consistency of primitives while enabling distinct editorial compositions.

### Interaction requirements

- Select/change categories and chains without losing user context unexpectedly.
- Tooltips explain facts; dialogs/inspectors contain substantial evidence. Mobile never depends on hover.
- Data-table state persists in URL if shareable, in local state otherwise; don't create meaningless URL churn for every hover.
- Chart pointer/touch interactions must be stable under scroll and responsive resizing.
- Selected table rows and compare state have clear visible affordances.
- Use loading skeletons where useful, but avoid placeholders that visually resemble actual risk conclusions.
- One primary call to action per major panel, not four arrow links in every tiny card.

---

# 21 — Performance, SEO, security and delivery quality

## Performance

- Preserve existing SSR/SSG architecture where applicable; identify current rendering model first.
- Avoid unnecessary chart libraries for simple bars/segments; reuse proven chart library for time series. Optimize large tables with virtualization only where row count justifies it.
- Lazy-load secondary charts and deep evidence when appropriate; loading states must preserve layout dimensions.
- Never prefetch huge raw account dumps for every leaderboard row. Load details on demand and cache by immutable snapshot/source identifiers.
- Use responsive logos/assets, properly sized raster/vector sources, avoid animated background assets and large ornamental images.
- Aim for Core Web Vitals targets under representative conditions (e.g. **LCP ≤2.5s, INP ≤200ms, CLS ≤0.1 at p75**) where measurable, while reporting actual tested values and connection assumptions. Do not assert performance targets are achieved without measurement.
- Profile expensive filtering and long-table re-render paths; maintain smooth keyboard/search interaction.

## SEO and research discoverability

- Correct **Celebtir** title and metadata; distinct page titles/descriptions for research routes; canonical URLs where appropriate.
- Indexable, shareable protocol/token/methodology pages when the existing architecture supports it; avoid leaking private/unreviewed data.
- Stable heading structure, semantic data tables and meaningful link text; avoid hiding all useful content behind client-only transient overlays.
- Provide source/methodology dates in public research pages; structured metadata only if accurate.
- No forced analytics, trackers, third-party fonts/CDNs or cookies that contradict existing “no cookies” statement without explicit approval and an updated disclosure.

## Security and trust

- Safe handling of external explorer/source links (`rel` safeguards when opening new tabs), sanitized external Markdown, length limits for remote identifiers, and robust URL validation.
- Never expose private keys or sensitive operator credentials in client assets, copy examples or public error logs.
- If public contributor/error submission form is added, rate limits/abuse prevention and privacy statement need separate approval.

---

# 22 — Technical delivery plan with review gates

This redesign should be implemented as a sequence of **reviewable, non-destructive working-tree changes**, not an all-at-once rewrite.

## Phase 0: inventory and architecture audit (required)

- Identify framework, routing, styling method, existing components, charting/data libraries, icons, assets, accessibility/testing tools.
- Verify authoritative scoring and active methodology versions; inventory scope of Solana, Hyperliquid, HIP-3 and token data.
- Capture baseline screenshots and a coverage matrix (route × viewport × state). Record known data gaps and actual source names.
- Produce a proposed component/data dependency map, migration risk list and selected representative fixtures.
- **Stop and report** if required research data does not exist; do not invent it to satisfy visual ideas.

## Phase 1: composition prototype & design tokens — **owner visual gate**

- Implement semantic Highwatch color/type/space tokens; asset/favicons and new AppShell.
- Build two distinctly composed screens: **Summary hero + primary exposure**, and **Risk matrix + inspector**. Also create one documentation excerpt to prove reading typography works.
- Take before/after screenshots at 1440 and 390 widths; include a high-density state and light-background logo test.
- Evaluate visual hierarchy/readability before expanding shared patterns. **Do not propagate uniform boxed layouts.**

## Phase 2: core investigation paths

- Finish Summary, Risk, protocol profile, controller detail and cross-links to methodology/evidence.
- Add real dataset-driven exposure composition and evidence detail state; stabilize URL/navigation behavior.
- Unit and integration tests for all risk/copy/coverage mappings.

## Phase 3: analytical views

- Redesign Value and Activity around chart-first/coverage-aware compositions.
- Redesign Tokens as directory + detail; test long accepted-by lists and issuer duplicates.
- Review HIP-3 existing route/data before implementing dedicated experience.

## Phase 4: comparative and chronological research

- Build Compare's first-load guidance, selected comparison layout and stable share links.
- Implement Changes timeline plus methodological change distinctions.
- Improve archived/shared-controller supporting routes.

## Phase 5: knowledge and system pages

- Convert Methodology into navigable research manual with sticky TOC, factor chapters, correct active thresholds, versioning and print styles.
- Refresh About, glossary, API/docs, feeds, footer, loading, empty, error and unavailable states as actually applicable.

## Phase 6: consistency, regression and handoff

- Verify accessible UI, responsive breakpoints, chart accuracy and semantic statuses.
- Run performance checks and screenshots across all required routes/states; fix major inconsistencies.
- Deliver review package: changed files, notes, screenshots, tests, unresolved issues, links to evidence fixtures, a design-vs-requirements checklist.
- **Leave the worktree uncommitted**, with no deployment or destructive cleanup. Ask for explicit approval before any commit.

### Implementation safeguards

- Separate presentation logic from domain scoring and data acquisition. Prefer stable mapping functions with tests.
- No mass refactoring or new state-management system without demonstrated necessity.
- No new API requirement merely to show a visual garnish; gate unavailable UI cleanly.
- Preserve all previous functionality and automation/integration endpoints; if adapting an API is unavoidable, document backward compatibility and secure approval before action.
- Do not remove tests, raw fixtures, archived records or evidence sources because they are not directly displayed.
- Before/after screenshots must use identical viewport, device scaling, route, chain selection, snapshot fixture and theme.

---

# 23 — QA acceptance matrix (must be executed, not merely checked verbally)

| Test ID | Case | Expected result |
|---|---|---|
| `QA-01` | Brand on header, metadata, favicon, docs | **Celebtir**, supplied approved symbol, correct alpha, usable dark/light versions |
| `QA-02` | Summary on 1440/1920 desktop | Main effective-control finding dominates, exposed denominator readable, no equal-weight KPI wall |
| `QA-03` | Summary on 390 mobile | Main finding and protocol navigation visible without body overflow or microtext |
| `QA-04` | Risk cell click/keyboard | Inspector shows fact, rule/version, evidence, missing caveat, focus returns on close |
| `QA-05` | All rating states | Low/Medium/High/Unknown/NA distinct and correctly sourced; no composite risk score |
| `QA-06` | Stage display | Stage and Unrated not conflated with risk/severity; Stage rules unaffected |
| `QA-07` | Value snapshot-only coverage | Useful current exposure composition; no invented risk history or dead giant chart |
| `QA-08` | Value total and segmented categories | Correct denominators, sources, rounding, overlap/exclusions disclosed |
| `QA-09` | Activity partial volume coverage | Shows numerator/denominator (e.g. 3/12) next to relevant total; missing ≠ zero |
| `QA-10` | Activity sparse/incomplete charts | Gaps remain gaps; no interpolated fake observations; accessible table exists |
| `QA-11` | Tokens directory dense sample | Search/filter, stable IDs, issuer/chain distinctions and manageable accepted-by lists |
| `QA-12` | Token permission drill-down | Four factors + observed authority + protocol roles + evidence; no generalized “unsafe token” |
| `QA-13` | Compare 0 selections | Purposeful guided selection; no unstyled debug chips or blank wasteland |
| `QA-14` | Compare 1/2/3/4 selections | Add/remove works; aligned rows, differing facts highlighted, URL share restores |
| `QA-15` | Compare incompatible types/versions | Displays unsupported/non-comparable states honestly, no fabricated matching ratings |
| `QA-16` | Changes empty/history cases | Recorded-period language, differentiated detected vs chain-event time, verifiable links |
| `QA-17` | Methodology deep links | Sticky TOC, active version, exact thresholds, readable code and print-safe tables |
| `QA-18` | Protocol evidence journey | From factor rating to effective authority and source account/slot in ≤2–3 deliberate steps |
| `QA-19` | HIP-3 route | Builder/deployer terminology correct; no Solana scoring/slot leakage or fake risk stages |
| `QA-20` | Global chain switch | Correct chain-specific metadata and coverage; incompatible filter state reset gracefully |
| `QA-21` | Table interactions | Search/sort/filter/columns/links persist correctly; screen-reader headers and focus visible |
| `QA-22` | A11y keyboard and touch | Dialogs, charts, dropdowns and evidence usable; reduced motion; contrast tested |
| `QA-23` | Responsive states | 1920/1440/1280/768/390 widths; no clipping/overflow, readable text |
| `QA-24` | Source/rating invariance | Raw values and factor/Stage outputs compare identical pre/post visual refactor |
| `QA-25` | Error/empty/stale/unknown states | No fake success, loading value, or misleading zero/Low state |
| `QA-26` | Routes and supporting content | Archives, controllers, API, About, Glossary, source feeds and footer links remain valid |
| `QA-27` | Accessibility/SEO/performance | Documented automated + manual audit with unresolved caveats; no unsubstantiated compliance claims |
| `QA-28` | Change control | No commits or deploys; diff audited; user review package provided |

## Visual review rubric (human judgment, 1–5)

Assess **first-impression hierarchy**, **scannability**, **data honesty**, **Highwatch distinctiveness**, **density calibration**, **navigational clarity**, **typographic readability**, **mobile utility** and **cross-page consistency**. A score below 4 on hierarchy/readability/data honesty should block spreading the design system to further pages.

In review, explicitly ask:

- At 100% zoom, is the main finding obvious without squinting?
- Does the user know what is *measured* versus what is *inferred*?
- Is there a clear path to evidence?
- Does the page have a different central visual role from adjacent pages?
- Are there fewer pointless boxes and less microcopy?
- Does the brand come from composition and proportion, not decoration?
- Can the dense table still serve power users?

---

# 24 — Content inventory / completeness checklist

**Every item below must be audited against actual routes and datasets.** If it does not currently exist, record it as **not implemented / future scope**, not silently “done”.

### Cross-platform

- [ ] Official naming, symbol, favicon, dark/light compatibility, metadata
- [ ] Brand token system, type scale, spacing, surfaces, controls and interaction patterns
- [ ] Desktop / laptop / tablet / mobile responsive behaviors
- [ ] Navigation, chain selection, search, keyboard interaction, footer
- [ ] Snapshot/source freshness, chain slot/block, methodology version, observation coverage
- [ ] No fabricated data, duplicate counting, stale claims or ratings derived in UI
- [ ] Empty/zero/Unknown/NA/unavailable/stale/error/loading states
- [ ] Accessible charts/tables, contrast, focus, reduced motion, print styles
- [ ] Cross-page deep linking, stable identifiers, back/forward and shareable research links
- [ ] Performance, safe text rendering, safe external links, SEO/discoverability

### Research pages

- [ ] Summary: primary governance-control story, exposure bar, supporting metrics, leaderboard, evidence-led insights
- [ ] Risk: six independent factors, Stage separate, filterable matrix, factor descriptions, inspector and evidence
- [ ] Value: weighted exposure by controller, properly sourced TVL, protocol and asset distributions, history gating
- [ ] Activity: source/coverage-aware KPIs, valid charts, tables, provider gaps, sparklines
- [ ] Tokens: search/type/permission filters, issuer/symbol disambiguation, token detail, permission evidence, accepted-by roles
- [ ] HIP-3: dedicated relevant deployer/market concepts, coverage/no-fake-stage fallback, chain-specific facts
- [ ] Compare: useful 0-selection state, 2–4 selections, difference logic, URL sharing, non-comparable rows
- [ ] Changes: observation/event timeline, prior/current values, detection/time semantics, methodology changes
- [ ] Methodology: versioned rules, sticky TOC, principles, six factors, four token factors, Stages, examples, source limits, print
- [ ] Protocol profiles: actual control architecture, fact→rule→source, Stage requirements, token exposure and historical changes
- [ ] Token profiles: mint/freeze/transfer/provenance, issuer, usage, evidence and change history

### Supporting destinations

- [ ] Shared controller / dependency listings and verified relationships
- [ ] Archived protocols and stable historical deep links
- [ ] About, glossary, API/documentation, data feeds, `llms.txt`, error reporting
- [ ] Not-found/invalid protocol/unsupported chain/deleted comparison gracefully handled
- [ ] Screen-reader and keyboard evidence workflows including dialogs and sources
- [ ] Regression fixture confirms authoritative scores and values unchanged
- [ ] Preview screenshots and final review **before committing**

---

# 25 — “Do not do this again” anti-pattern register

1. **Do not** replace the current navy cards with larger navy cards and call that a redesign.
2. **Do not** fit every page into one fixed 900–1000px content column on high-resolution displays.
3. **Do not** solve poor hierarchy with more gold, gradients, stars or decorative animation.
4. **Do not** use ten-pixel subtitles or excessively compressed tables merely to show more cells.
5. **Do not** put an equal-size KPI card around every number or give every section equal prominence.
6. **Do not** show long “accepted by” lists in default token-directory rows.
7. **Do not** reserve giant chart containers for nonexistent snapshots or giant blank areas on Compare.
8. **Do not** mislabel protocol High rating as global protocol insecurity, or token mint authority as proven fraud.
9. **Do not** invent product metrics, trend history, user alerts, stages, sources or rankings for aesthetic completeness.
10. **Do not** make every page look like Summary; each workflow has a different optimal form.
11. **Do not** lose research exactness through oversimplified graphical badges or ambiguous metaphors.
12. **Do not** copy L2BEAT pixel for pixel or reuse its name/branding; benchmark its **research UX principles**.
13. **Do not** make methodology visually “simple” by removing criteria, thresholds, exceptions or reproducibility details.
14. **Do not** change backend/domain scoring in the course of a front-end facelift.
15. **Do not** remove old routes, tests, evidence files or snapshot history to achieve layout goals.

---

# 26 — Reviewer deliverables and final handoff

A complete Codex review response must include:

1. **Inventory:** actual route tree, data capabilities/limitations, initial design debt, selected official logo path and methodology version.
2. **Before/after images** of Summary, Risk, Value, Activity, Tokens, Compare (empty + populated), Changes (normal + empty), Methodology (top, representative factor, mobile), HIP-3 (if implemented), and protocol/token detail pages.
3. **Comparison storyboard:** one complete user path from Summary → protocol → risk evidence → Methodology; another Tokens → protocols; one Compare shared link.
4. **Implementation diff summary:** new/refactored components, token files, route changes, interaction changes, potential compatibility risks.
5. **Data fidelity report:** saved fixture, metric parity checks, rating/stage parity, missing-data cases, cross-chain semantics, date/provider coverage.
6. **QA results:** route and responsive snapshots, unit/integration/e2e checks, accessibility checks, lint/typecheck/build, performance where measured, remaining known issues.
7. **Review blockers:** unsupported data/features, pending decisions, anomalies and concerns. State honestly what could not be validated.
8. **Change-control statement:** no commit or deployment; working tree left available for owner review.

## Minimum approval gate

Do not ask for blanket signoff on a giant visual rewrite without first receiving feedback on the **Summary hero, Risk matrix + evidence inspector, Tokens directory, and Compare empty state**. Those four screens prove whether the new system is truly better across four different workflow types.

---

# 27 — External research references (principles and concepts; not assets to copy)

- L2BEAT homepage and scaling/research organization: <https://l2beat.com/> and <https://l2beat.com/scaling/summary>. Reference **hierarchy, contextual explanations, project drill-down, risk matrix, and discoverability**.
- L2BEAT risk analysis: <https://l2beat.com/scaling/risk>. Reference **readable, descriptive factor observations**, not its Ethereum-specific factor definitions.
- L2BEAT project structure example: <https://l2beat.com/scaling/projects/sophon>. Reference **sectioned technical research and project-level evidence**.
- L2BEAT changelog: <https://l2beat.com/changelog>. Reference their product evolution toward shareable comparisons and readable Markdown research; **no obligation to clone features**.
- Hyperliquid official HIP-3 documentation: <https://hyperliquid.gitbook.io/hyperliquid-docs/hyperliquid-improvement-proposals-hips/hip-3-builder-deployed-perpetuals>. Reference **market/deployer responsibilities and terminological accuracy**; check current chain configuration before asserting specifics.
- Hyperliquid HIP-3 deployer actions: <https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/hip-3-deployer-actions>. Reference **observed control capability possibilities**; capability availability depends on actual deployed protocol version/state.

The current application screenshots and **the actual repository's methodology/data contracts are authoritative** for what Celebtir currently displays. External sources are inspiration/reference for interface and chain-specific concepts, not a replacement for the project's scoring system.

---

# 28 — Final instruction to Codex

**Redesign the whole experience, not merely its theme.**

Deliver a restrained, visually confident **Silver Watch** research platform where a first-time user can understand the most important control findings immediately and a technical researcher can follow any claim to its source. Each page must have its own information hierarchy, with consistent components only where they improve clarity. Refuse the temptation to fill every gap with another card, chart, placeholder or decoration.

**Protect data integrity, active methodology, deep links and working functionality. Build and validate the first composition gates, extend carefully, report actual evidence and issues, and do not commit or deploy until expressly approved.**

---

# 29 — Completed specification validation (author-side review)

**Validation performed:** 21 October 2026. This checks the *instruction document's completeness and consistency* against the supplied screenshots and stated product requirements. It **does not** mean the application or its codebase has been audited, implemented or tested.

| Evidence / requirement | Covered in | Review disposition |
|---|---|---|
| Summary screenshot — equal KPIs, repeated cards, 71.4% control-exposure finding, crowded leaderboard/insights | §§01, 02, 05, 23 | Covered: new hierarchy, dynamic denominator, composition, concise leaderboard, nonempty insight prioritization |
| Risk screenshot — six-factor matrix, unreadable tiny badges, Stage separation | §§01, 06, 14, 17, 23 | Covered: inspector, semantic status, evidence and no score aggregation |
| Value screenshot — repeated breakdown and missing governance history | §§01, 07, 17, 23 | Covered: one dominant composition, provider time-series distinction and snapshot-only fallback |
| Activity screenshot — volume only partially reported, uniform charts | §§01, 08, 17, 23 | Covered: coverage-first metrics, accuracy of gaps, accessible trends |
| Tokens screenshot — many small rows and long “accepted by” chains | §§01, 09, 18, 23 | Covered: grouped searchable explorer, terse rows, detailed permissions/usage |
| Compare screenshot — almost empty first state and raw candidate chips | §§01, 11, 23 | Covered: guided first selection and verified 2–4 way research |
| Methodology long-page screenshot — weak document navigation and dense prose | §§01, 13, 18, 23 | Covered: sticky TOC, rule chapters, provenance, versions, print |
| HIP-3 in navigation but no supplied HIP-3 body screenshot | §§01, 10, 17, 23 | **Explicit limitation:** concept and acceptance provided; actual route/schema requires audit before coding |
| Changes, protocol detail, token detail, supporting routes not fully shown | §§12, 14, 15, 23, 24 | **Explicit limitation:** desired UX defined, implementation/coverage to be confirmed from repo |
| Approved Celebtir name, Silver Watch insignia, silver/gold/blue/white, restrained Elvish influence | §§00, 03, 20, 23 | Covered: logo asset authority and practical favicon/print constraints |
| L2BEAT-like impact without copying branding or methodology | §§00, 02, 03, 27, 28 | Covered: hierarchy, project profiles, factor detail and comparisons |
| Solana/Hyperliquid semantics, snapshot IDs, provenance, incompatible comparisons | §§00, 04, 10, 17, 23 | Covered: explicit population/scope and chain-specific metadata |
| All data integrity, methods, Stage, no composite, status distinctions | §§00, 16, 17, 23 | Covered: invariants and evidence parity tests |
| Accessibility, responsive behavior, performance, content tone and error handling | §§16, 18–21, 23 | Covered: testable requirements rather than aesthetic generalities |
| Implementation sequencing, visual review gates and no-commit rule | §§22–26, 28 | Covered: review package and stop conditions |

### Document integrity checks performed

- Verified the numbered major sections are continuous from **00 through 29**.
- Verified the fenced code blocks have balanced opening and closing markers.
- Checked for all principal routes, rating states, chain contexts, visual tokens, audit responsibilities and owner review instructions.
- Explicitly separated design references from **uninspected repository facts**; HIP-3 and supporting routes must be inspected during Phase 0.
- Resolved a possible scope conflict: **dark UI remains default**; a new light-mode toggle is **not** necessary simply because the approved insignia must render on light backgrounds.
- Checked that the document does not direct Codex to fabricate missing history, alter domain scoring, repurpose Solana stages for HIP-3, or auto-commit.

### Remaining unknowns requiring actual repository access

1. Exact active methodology thresholds/version and stored evaluation provenance.
2. Current route implementations and whether protocol/token/controller detail screens already exist.
3. HIP-3 data availability, observed deployer powers, and whether any HIP-3-specific scoring methodology has been adopted.
4. How all-chain TVL is aggregated and whether provider values double-count overlapping assets.
5. Exact approved logo source format and optical quality of transparent/favicon variants.
6. Supported APIs, exported evidence, Markdown pages, document versions, URL state and design/test tooling.
7. Browser-specific layout and accessibility results; they require running the frontend, not inspecting screenshots alone.

**Validation verdict:** The specification covers the visible page problems, product-wide design organization, all major routes and research integrity controls. Remaining unknowns are **implementation discovery tasks**, not omissions to fill with guesses. **No visual design direction is authorized to override live facts or source methodology.**
