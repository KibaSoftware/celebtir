#!/usr/bin/env bash
# Builds fixtures/site-data/: the demo dataset the site uses until real
# snapshots exist, made by running the real pipeline offline on the golden
# recording for three days.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -q -p celebtir
bin="${CARGO_TARGET_DIR:-target}/debug/celebtir"
tmp="$(mktemp -d)"
for d in 2026-10-15 2026-10-16 2026-10-17; do
  bash scripts/daily.sh --date "$d" --registry fixtures/golden/registry --data "$tmp" --bin "$bin" \
    --offline fixtures/golden/replay --tx-sample 5 --history-limit 5 > /dev/null
done
rm -rf fixtures/site-data
mkdir -p fixtures/site-data
cp -r "$tmp/latest" "$tmp/history" "$tmp/feeds" "$tmp/api" fixtures/site-data/
rm -rf "$tmp"
echo "fixtures/site-data rebuilt"
