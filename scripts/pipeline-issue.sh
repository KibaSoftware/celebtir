#!/usr/bin/env bash
# OPS-4: keep exactly one open issue labelled `pipeline-failure`.
#   pipeline-issue.sh open [log-file] [run-url]   open it, or add a comment
#   pipeline-issue.sh close                       close it after a clean run
set -euo pipefail
action="${1:?open or close}"
label="pipeline-failure"
existing="$(gh issue list --label "$label" --state open --json number --jq '.[0].number' || true)"

case "$action" in
  open)
    log="${2:-}"
    url="${3:-}"
    body="The daily pipeline did not complete cleanly on $(date -u +%Y-%m-%d)."
    [ -n "$url" ] && body="$body Run: $url"
    if [ -n "$log" ] && [ -f "$log" ]; then
      body="$body"$'\n\n```\n'"$(tail -n 60 "$log")"$'\n```'
    fi
    if [ -n "$existing" ]; then
      gh issue comment "$existing" --body "$body"
    else
      gh label create "$label" --color B60205 --force >/dev/null 2>&1 || true
      gh issue create --title "Daily pipeline failure" --label "$label" --body "$body"
    fi
    ;;
  close)
    if [ -n "$existing" ]; then
      gh issue close "$existing" --comment "Closed by a clean run on $(date -u +%Y-%m-%d)."
    fi
    ;;
  *) echo "usage: $0 open [log] [run-url] | close" >&2; exit 64 ;;
esac
