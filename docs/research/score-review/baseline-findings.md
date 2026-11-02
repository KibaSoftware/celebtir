# Production baseline findings

The live public API and rendered methodology agree on **v0.2, snapshot 2026-10-26**. This is not the owner's expected v0.3 release. Initial captures were taken on 27 October and the public summary and browser routes were rechecked on 28 October. The contract-named output folder retains `2026-10-26`; actual capture timestamps are recorded separately.

Checkout HEAD: `b114b281ae6a57d3d6ff198963087e999323a5eb`, branch main. The checkout was already dirty: README deletion and almost the whole application untracked. A HEAD comparison cannot identify the deployed source revision or isolate this review. Preserve the initial status in `output/reviews/score-review-2026-10-26/logs/reconnaissance.json`; use protected input hashes and the explicit file manifest for this review.

## Observed data

| Domain | Published coverage | Stage interpretation |
|---|---|---|
| Solana protocols | 12 assessed | 10 Stage 0, 2 Unrated; zero Stage 1/2 |
| HyperEVM protocols | 4 unassessed | No Solana Stage |
| Tokens | 29 captured public token API records | Four-factor token model; no Stage |
| HIP-3 | Separate facts-only route | Not included in protocol Stage denominator; raw HIP-3 API not exposed at discovered paths |

The two Unrated Solana records are binance-staked-sol and forward-sol. Exact API records, schema files, response headers and hashes are retained under `hosted/` and `http-captures.json`. The API uses `/data/v2/`; this transport schema version does not mean methodology v0.2. The local build explicitly uses `output/qa-v03-data`, methodology v0.3. These inputs are distinct and must not be combined in a distribution or represented as an equivalent release.

The initial artifact comparison contains 52 checks and 47 mismatches. JSON object-key order is normalized; arrays and all values are preserved. A separate ordering/semantic comparison is in `artifact-equivalence.json`. Different versioned outputs are expected to differ, but the intended-release discrepancy remains unresolved. `/changes/` was an initial reconnaissance guess returning 404; the actual implemented `/changelog/` route returned 200 in the hosted route matrix. This is not evidence of a broken application navigation link.

## Source and deployment diagnosis

All 12 complete published Solana protocol score objects reproduce exactly with the current Rust scorer and the published v0.2 methodology over the captured public facts. No scoring-rule bug was demonstrated. The read-only diagnostic adds labels without replacing ratings.

`scripts/deploy/run.sh`'s `--update-locked` path selects the latest stored snapshot and rebuilds site data; it does not recollect or rescore it. `scripts/deploy/update.sh` builds the new CLI and invokes that application-only path. Therefore an application update can correctly preserve a v0.2 stored snapshot even when the scorer default is now v0.3. This is a code-supported possible mechanism, **not proof of the remote command or timer history**. The deployed Git SHA/workflow/build binding is not exposed by the inspected public artifacts. SSH, service journals and timers were not inspected.

Owner decision: “No, I expected v0.3 to be live.” Treat this as a release discrepancy, not permission to rewrite historical data or deploy. The same 26 October snapshot was still public on 28 October; investigate the collection/publication journal and daily timer before calling it current.

## Reproduction and remaining gates

Captured facts were reconstructed from protocol/token APIs into the isolated `replay/facts.json`; null metrics were omitted, never replaced with zero. `replay/scores.json` is the current Rust v0.2 replay. `logs/replay.log` records execution. Raw HIP-3 data and full remote run metadata are not available through this public capture, so those cannot be claimed equivalent.

Read-only next action: bind the host's current release symlink/source SHA, latest stored snapshot methodology, publication journal and daily timer logs. Then prepare an isolated v0.3 release candidate using the intended reviewed data and validate its publication artifacts. Any new collection, rescoring/publication decision or deployment requires its own authorized scope; no historical overwrite is proposed here.

G0 remains **BLOCKED** on deployed source binding and reconciliation of the owner-expected version. The browser/API pair is internally consistent; this is not a claim that the website renders v0.3 scores with a v0.2 label. No synthetic-data publication was observed, but inaccessible remote inputs are not independently certified.
