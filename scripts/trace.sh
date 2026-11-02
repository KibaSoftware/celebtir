#!/usr/bin/env bash
# Requirement traceability check (see AGENTS.md).
#  1. Every requirement ID in the spec is owned by a slice in AGENTS.md.
#  2. Every Must ID in a done ([x]) slice, unless marked with '*', has at least
#     one test whose name contains it: SCO-3 -> sco_3_...
set -euo pipefail
cd "$(dirname "$0")/.."

# HOPS-3: both original and wave 2 IDs live in the consolidated register.
specs=("AGENTS.md")
backlog="AGENTS.md"
fail=0

# "ID PRIORITY" pairs from the spec's requirement tables.
ids=$(cat "${specs[@]}" | grep -E '^\| [A-Z]+-[0-9]+ \|' \
  | awk -F'|' '{id=$2; p=$(NF-1); gsub(/ /,"",id); gsub(/ /,"",p); print id, p}')

slices=$(grep -E '^- \[.\] \*\*' "$backlog")

while read -r id prio; do
  if ! grep -qE "(^|[^A-Z])${id}\*?([^0-9]|$)" <<<"$slices"; then
    echo "missing from backlog: $id"
    fail=1
  fi
done <<<"$ids"

done_ids=$(grep -E '^- \[x\] \*\*' "$backlog" \
  | grep -oE '(^|[^A-Z])[A-Z]+-[0-9]+\*?' | grep -oE '[A-Z]+-[0-9]+\*?' || true)

search_dirs=()
for d in crates site/src site/tests site/e2e scripts/tests; do
  [ -d "$d" ] && search_dirs+=("$d")
done

checked=0
for tok in $done_ids; do
  [[ "$tok" == *'*' ]] && continue
  prio=$(awk -v id="$tok" '$1 == id {print $2}' <<<"$ids")
  [ "$prio" = "Must" ] || continue
  name=$(tr 'A-Z-' 'a-z_' <<<"$tok")
  checked=$((checked + 1))
  if ! grep -rIqE "\b${name}[_ ]" --exclude-dir=node_modules --exclude-dir=target "${search_dirs[@]}" 2>/dev/null; then
    echo "no test named ${name}_* for done requirement $tok"
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  echo "trace: FAILED"
  exit 1
fi
echo "trace: ok ($(wc -l <<<"$ids") spec IDs in backlog, $checked done Must IDs have tests)"
