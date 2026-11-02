#!/usr/bin/env bash
# The daily pipeline (OPS-2): validate → collect → score → diff → compact →
# build site data. The workflow runs this, then commits data/ and deploys.
#
#   scripts/daily.sh [--date YYYY-MM-DD] [--registry DIR] [--data DIR] [--offline DIR] [--bin PATH] [extra collect args...]
#
# Exit codes: 0 all good; 2 some protocols failed (data still published);
# anything else stops the run before data/ changes, so the deployed site keeps
# yesterday's data.
set -euo pipefail

date="$(date -u +%Y-%m-%d)"
registry="registry"
data="data"
bin="celebtir"
methodology=()
collect_extra=()
while [ $# -gt 0 ]; do
  case "$1" in
    --date) date="$2"; shift 2 ;;
    --registry) registry="$2"; shift 2 ;;
    --data) data="$2"; shift 2 ;;
    --bin) bin="$2"; shift 2 ;;
    --methodology) methodology=(--methodology "$2"); shift 2 ;;
    *) collect_extra+=("$1"); shift ;;
  esac
done

destination="$data/snapshots/$date"
if [ -e "$destination" ]; then
  echo "snapshot $date already exists; replay into a separate --data directory" >&2
  exit 1
fi
mkdir -p "$data/snapshots"
out=$(mktemp -d "$data/.candidate-$date.XXXXXX")
step() { echo "::group::$1"; }
end() { echo "::endgroup::"; }

step "validate"
"$bin" validate --registry "$registry"
end

step "collect"
set +e
"$bin" collect --registry "$registry" --out "$out" --date "$date" \
  --cache "$data/cache/tx-cache.json" --previous "$data/latest/facts.json" "${collect_extra[@]}"
collect_status=$?
set -e
end
case "$collect_status" in
  0|2) ;;
  *) echo "collect failed with exit $collect_status; nothing published" >&2
     echo "failed candidate retained at $out" >&2
     exit "$collect_status" ;;
esac

step "score"
"$bin" score --facts "$out/facts.json" --out "$out/scores.json" "${methodology[@]}"
end

# Publish only a fully collected and scored candidate; never replace a day.
if [ -e "$destination" ]; then
  echo "snapshot appeared during collection; candidate retained at $out" >&2
  exit 1
fi
mv -T -n "$out" "$destination"
if [ -e "$out" ]; then echo "snapshot publish refused; candidate retained" >&2; exit 1; fi
out="$destination"

step "diff"
previous=""
for dir in $(ls -1d "$data"/snapshots/*/ 2>/dev/null | sort); do
  name="$(basename "$dir")"
  if [[ "$name" < "$date" && -f "$dir/facts.json" && -f "$dir/scores.json" ]]; then
    previous="$dir"
  fi
done
if [ -n "$previous" ]; then
  "$bin" diff "${previous%/}" "$out"
else
  # The first snapshot reports its recorded upgrade history (F6).
  "$bin" diff --initial "$out"
fi
end

step "compact and build site data"
"$bin" compact --data "$data" --as-of "$date"
"$bin" build-site-data --data "$data" --as-of "$date"
end

"$bin" summarize "$out"
exit "$collect_status"
