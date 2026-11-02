#!/usr/bin/env bash
# Every gate from AGENTS.md; stops at the first failure.
set -euo pipefail
cd "$(dirname "$0")/.."
export CELEBTIR_REPO_URL="${CELEBTIR_REPO_URL:-https://github.com/KibaSoftware/celedir}"
node --test scripts/tests/invariants.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -q
bash scripts/trace.sh
bash scripts/tests/trace_test.sh
cargo build -q -p celebtir-score --target wasm32-unknown-unknown
if [ -f site/package.json ]; then
  # One command per line: set -e ignores failures inside an && chain.
  pnpm -C site lint
  pnpm -C site typecheck
  pnpm -C site test
  pnpm -C site build
  node site/scripts/check-candidate.mjs site/out
  # UI-14, UI-15: every protocol page has its Markdown twin and share card.
  test -f site/out/llms.txt
  for f in site/out/protocols/*/index.html; do
    id=$(basename "$(dirname "$f")")
    test -f "site/out/protocols/$id.md"
    test -f "site/out/og/$id.png"
    # API-1, API-4: its API file and Stage badge.
    test -f "site/out/data/v2/protocols/$id.json"
    test -f "site/out/badges/$id.svg"
  done
  test -f site/out/data/v2/summary.json
  test -f site/out/data/v2/schemas/summary.schema.json
  # UI-27 and QA-4: JS budget, then end-to-end, visual and axe tests.
  pnpm -C site budget
  pnpm -C site e2e
fi
echo "gates: ok"
