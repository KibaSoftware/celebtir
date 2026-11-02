# Celebtir

**DeFi Risk Intelligence**  
Independent. Evidence-based. Transparent.

Application version **0.1.1** — see [CHANGELOG.md](CHANGELOG.md). Application, methodology and data-schema versions are independent.

Celebtir is an open, evidence-linked scorecard of DeFi control and trust assumptions. It explains who can upgrade code, change parameters, pause operations, control token supply and influence prices. Readers can inspect the underlying evidence, compare protocols and explore control graphs.

Stages and factor ratings describe observed control arrangements. They are not a smart-contract security audit, a combined safety score or an investment recommendation. Solana assessments, native HyperEVM contract facts and unrated HyperCore HIP-3 facts have distinct coverage boundaries.

## Start locally

Requirements: Node.js 22, the pnpm version declared in [site/package.json](site/package.json), and Rust 1.88 through [rust-toolchain.toml](rust-toolchain.toml) for the CLI. Windows commands below use PowerShell; shell-based Rust/release checks use Git Bash on Windows.

```powershell
pnpm.cmd -C site install --frozen-lockfile

# Use the reproducible demo dataset explicitly for development and tests.
$env:CELEBTIR_DATA = Join-Path (Get-Location) 'fixtures/site-data'
pnpm.cmd -C site dev
```

Open the local URL printed by Next.js. The site is a static export; to inspect a production build:

```powershell
pnpm.cmd -C site build
Set-Location site
node scripts/serve-out.mjs 3100
```

`CELEBTIR_DATA` selects the dataset. When unset, the loader uses `data/` if a stored snapshot exists, otherwise `fixtures/site-data/`. The checked-in historical data is not a live feed; its snapshot date and methodology remain visible. Do not overwrite published days or historical replay fixtures.

## Validate changes

From the repository root, with the same `CELEBTIR_DATA` used for the build:

```powershell
pnpm.cmd -C site lint
pnpm.cmd -C site typecheck
pnpm.cmd -C site test
pnpm.cmd -C site build
pnpm.cmd -C site budget
pnpm.cmd -C site e2e
$env:CARGO_TARGET_DIR = 'target/review'
$env:CARGO_INCREMENTAL = '0'
cargo test --workspace
```

Typechecking generates Next.js types before invoking TypeScript, so it works on a fresh checkout. `pnpm test` first copies the selected dataset's public API files and feeds, so schema-link tests do not require a prior build. DOM tests provide a ResizeObserver stand-in for jsdom; actual graph layout and resizing are covered by browser tests. Browser tests use installed Chrome. Windows visual baselines are included; Linux pixel-baseline capture and other release checks are documented in [AGENTS.md](AGENTS.md). The complete local gate is `bash scripts/gates.sh`; CI additionally runs coverage and Lighthouse checks. Passing local checks does not establish hosted publication or visual acceptance.

Release checks also require `CELEBTIR_REPO_URL=https://github.com/<owner>/<repo>` when building, so the About page has its corrections link. `CELEBTIR_SITE_URL` supplies the public site origin for absolute metadata URLs. CI sets the repository URL explicitly; local browsing can run without these release settings.

## Repository layout

| Path | Purpose |
| --- | --- |
| `crates/` | Rust collectors, registry, deterministic scoring, changes and CLI |
| `site/` | Next.js interface, shared presentation and browser/component tests |
| `registry/` | Sourced protocol, token and controller declarations; drafts remain inactive |
| `methodology/` | Immutable published scoring versions and changelog |
| `schemas/` | Public data contracts |
| `fixtures/` | Curated offline captures, demo data and replay expectations |
| `data/` | Preserved historical snapshots, histories and generated public data; local collector cache is ignored |
| `scripts/` | Local gates, collection pipelines and host deployment helpers |
| `docs/` | Design, deployment and evidence-review documentation |

[AGENTS.md](AGENTS.md) is the maintained project entry point for current decisions, commands and unresolved work.

## Local files and deployment

`.env`, the owner's root `deploy.ps1`, agent worktrees, browser sessions, `output/`, local collector caches, dependencies and build/test artifacts are ignored. Ignore rules preserve local files; they do not delete them. Review reports may link to ignored `output/` evidence that is only available in the originating workspace.

The ignored deployment entry point is deliberately not distributed by a clone. Its local commands and the versioned host helpers are described in [docs/deployment.md](docs/deployment.md). Deployment-entry tests require that owner's local script; ordinary CI does not call it. Runtime secrets must not be committed.

GitHub's daily collection, bot data commits, Pages publication and schedule monitor are disabled by default through the repository variable `CELEBTIR_ENABLE_GITHUB_DAILY`. Only set it to `true` after an explicit decision to use that publication path. Source CI and registry-review checks remain enabled. The push-triggered `CI checks (no deployment)` workflow has read-only repository permissions; its offline deployment tests use temporary directories and mocked services without publishing or connecting to the host. Creating the first commit does not activate registry drafts or authorize deployment.

## Licenses

Code is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE). See [LICENSE-DATA.md](LICENSE-DATA.md) for data licensing and third-party source attribution.
