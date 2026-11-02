#!/usr/bin/env bash
# Tests for scripts/trace.sh, run by scripts/gates.sh.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"

# Builds a throwaway repo with one ID per original requirement family and runs trace.sh in it.
run_trace() {
  local dir
  dir=$(mktemp -d)
  mkdir -p "$dir/scripts" "$dir/crates"
  cp "$root/scripts/trace.sh" "$dir/scripts/"
  printf '| AAA-1 | r | t | Must |\n' >"$dir/AGENTS.md"
  printf '| HOPS-3 | r | t | Must |\n' >>"$dir/AGENTS.md"
  printf '%s\n' "$1" >>"$dir/AGENTS.md"
  echo 'fn aaa_1_x() {}' >"$dir/crates/t.rs"
  local rc=0
  bash "$dir/scripts/trace.sh" >"$dir/out" 2>&1 || rc=$?
  cat "$dir/out"
  rm -rf "$dir"
  return $rc
}

hops_3_wave_2_ids_must_be_in_backlog() {
  if out=$(run_trace '- [x] **1** a — AAA-1'); then
    echo "FAIL hops_3: trace passed with HOPS-3 missing from backlog"
    return 1
  fi
  grep -q "missing from backlog: HOPS-3" <<<"$out"
  run_trace $'- [x] **1** a — AAA-1\n- [ ] **2** b — HOPS-3' >/dev/null
  echo "ok hops_3_wave_2_ids_must_be_in_backlog"
}

hops_3_wave_2_ids_must_be_in_backlog
