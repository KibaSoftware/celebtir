#!/usr/bin/env bash
# CHG-4: events are permanent. Fails if any events.json that exists at the
# base revision was modified or deleted since. Usage: check-events-append-only.sh <base-ref>
set -euo pipefail
base="${1:-origin/main}"
changed=$(git diff --name-status "$base"...HEAD -- 'data/snapshots/*/events.json' | grep -v '^A' || true)
if [ -n "$changed" ]; then
  echo "events.json files are append-only (CHG-4); these were rewritten:"
  echo "$changed"
  echo "Record a correction as a new event with \"corrects\": \"<old id>\" instead."
  exit 1
fi
echo "events: append-only ok"
