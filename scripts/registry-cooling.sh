#!/usr/bin/env bash
# REG-6: a registry change may merge only 24 hours after its last commit.
#   registry-cooling.sh <base-ref> [now-epoch]
# Prints `success`, `pending <hours-left>`, or `not-applicable`, and exits 0;
# exits 1 only on bad usage.
set -euo pipefail
base="${1:?base ref}"
now="${2:-$(date -u +%s)}"
cooling_hours=24

if git diff --quiet "$base"...HEAD -- registry/; then
  echo "not-applicable"
  exit 0
fi
last="$(git log -1 --format=%ct HEAD)"
elapsed=$(( now - last ))
needed=$(( cooling_hours * 3600 ))
if [ "$elapsed" -ge "$needed" ]; then
  echo "success"
else
  left=$(( (needed - elapsed + 3599) / 3600 ))
  echo "pending $left"
fi
