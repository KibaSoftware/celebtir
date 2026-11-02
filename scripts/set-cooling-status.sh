#!/usr/bin/env bash
# Sets the `registry-cooling` commit status from registry-cooling.sh (REG-6).
#   set-cooling-status.sh <base-ref> <sha>
set -euo pipefail
base="$1"
sha="$2"
result="$(bash "$(dirname "$0")/registry-cooling.sh" "$base")"
case "$result" in
  success)        state=success; desc="24-hour cooling period over" ;;
  not-applicable) state=success; desc="no registry changes" ;;
  pending*)       state=pending; desc="registry change: ${result#pending } more hour(s) of cooling" ;;
  *)              state=error;   desc="unexpected: $result" ;;
esac
gh api "repos/$GITHUB_REPOSITORY/statuses/$sha" \
  -f state="$state" -f context="registry-cooling" -f description="$desc" >/dev/null
echo "registry-cooling: $state ($desc)"
