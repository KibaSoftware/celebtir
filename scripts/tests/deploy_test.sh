#!/usr/bin/env bash
# Offline tests: real publication/retention; collection/build/HTTP are doubles.
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
base=$(mktemp -d)
echo "Deployment test fixtures: $base"
mkdir "$base/symlink-probe"
ln -s "$base/symlink-probe" "$base/probe-link"
if [ ! -L "$base/probe-link" ]; then
  echo 'Real Unix symlinks required; copy emulation cannot validate publication.' >&2
  exit 77
fi
export REAL_NODE="$(command -v node)" REAL_MV="$(command -v mv)"
for scenario in success partial collection_failure build_failure same_day public_failure wrong_release switch_failure empty_tokens fresh_deploy same_day_deploy partial_deploy stale_methodology invariant_failure corrections_failure; do
  root="$base/$scenario"
  export ROOT_TEST="$root" SCENARIO="$scenario"
  src="$root/releases/20261023T120000Z-abcdef12"
  old_source="$root/releases/20261022T120000Z-abcdef12"
  old_data="$root/shared/generations/seed-20261022T120000Z-abcdef12"
  old_web="$root/web/20261022T120000Z-123"
  mkdir -p "$src/scripts/deploy" "$src/site/out/data/v2" "$src/bin" "$old_data/snapshots" "$old_web" "$old_source" "$root/tools/node/bin" "$root/home"
  ln -s "$old_data" "$root/shared/data"
  ln -s "$old_web" "$root/current"
  ln -s "$old_source" "$root/source"
  printf 'old\n' > "$old_web/index.html"
  printf 'original\n' > "$old_data/evidence"
  mkdir "$old_data/latest"
  printf '{"protocols":{"p":{}},"tokens":{"t":{}}}\n' > "$old_data/latest/facts.json"
  touch "$root/env"
  sed -e "s|root=/opt/celebtir|root='$root'|" -e "s|/etc/celebtir/collector.env|$root/env|" "$repo/scripts/deploy/run.sh" > "$root/run.sh"
  sed "s|const root = '/opt/celebtir';|const root = '$root';|" "$repo/scripts/deploy/storage.mjs" > "$src/scripts/deploy/storage.mjs"
  cp "$repo/scripts/deploy/publication.sh" "$src/scripts/deploy/publication.sh"
  sed "s|/opt/celebtir/shared/generations|$root/shared/generations|" "$repo/scripts/deploy/release-data.mjs" > "$src/scripts/deploy/release-data.mjs"
  mkdir -p "$src/methodology" "$src/crates/celebtir-score/methodology"
  printf 'version = "0.3"\n' > "$src/methodology/v0.3.toml"
  cp "$src/methodology/v0.3.toml" "$src/crates/celebtir-score/methodology/v0.3.toml"
  printf '{"methodology_version":"0.3"}\n' > "$old_data/latest/scores.json"
  cat > "$src/scripts/daily.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ "$SCENARIO" = collection_failure ]; then exit 1; fi
while [ "$1" != --data ]; do shift; done
mkdir -p "$2/snapshots/$(date -u +%F)/raw"
printf 'unused\n' > "$2/snapshots/$(date -u +%F)/raw/perpDexs.json"
printf 'fresh\n' > "$2/fresh"
printf '{"methodology_version":"0.3"}\n' > "$2/latest/scores.json"
if [ "$SCENARIO" = empty_tokens ]; then printf '{"protocols":{"p":{}},"tokens":{}}\n' > "$2/latest/facts.json"; exit 2; fi
if [ "$SCENARIO" = partial ] || [ "$SCENARIO" = partial_deploy ]; then exit 2; fi
SH
  cat > "$root/tools/node/bin/pnpm" <<'SH'
#!/usr/bin/env bash
if [ "$SCENARIO" = build_failure ]; then exit 1; fi
printf 'new\n' > site/out/index.html
printf 'not found\n' > site/out/404.html
version=0.3
if [ "$SCENARIO" = stale_methodology ]; then version=0.2; fi
printf '{"snapshot_date":"%s","methodology_version":"%s"}\n' "$(date -u +%F)" "$version" > site/out/data/v2/summary.json
SH
  cat > "$root/tools/node/bin/node" <<'SH'
#!/usr/bin/env bash
if [[ "$1" = */storage.mjs ]]; then exec "$REAL_NODE" "$@"; fi
if [[ "$1" = */release-data.mjs ]]; then exec "$REAL_NODE" "$@"; fi
if [ "$1" = celebtir-invariants.mjs ] && [ "$SCENARIO" = invariant_failure ]; then exit 1; fi
if [ "$1" = site/scripts/check-release.mjs ] && [ "$SCENARIO" = corrections_failure ]; then exit 1; fi
if [ "$1" = scripts/serve-out.mjs ]; then exec sleep 30; fi
if [ "${2:-}" = https://celebtir.com ] && [ "$SCENARIO" = public_failure ]; then exit 1; fi
exit 0
SH
  cat > "$root/tools/node/bin/curl" <<'SH'
#!/usr/bin/env bash
if [[ "$*" = *https://* ]]; then
  if [ "$SCENARIO" = wrong_release ]; then echo stale; else cat "$ROOT_TEST/current/deployment-id.txt"; fi
else cat site/out/deployment-id.txt; fi
SH
  cat > "$root/tools/node/bin/mv" <<'SH'
#!/usr/bin/env bash
if [ "$SCENARIO" = switch_failure ] && [[ "$*" = *current.restore* ]] && [ ! -f "$ROOT_TEST/fail-once" ]; then
  touch "$ROOT_TEST/fail-once"; exit 1
fi
exec "$REAL_MV" "$@"
SH
  printf '#!/usr/bin/env bash\nexit 0\n' > "$src/bin/celebtir"
  chmod +x "$root/tools/node/bin/"* "$src/bin/celebtir"
  if [ "$scenario" = same_day ] || [ "$scenario" = same_day_deploy ]; then
    mkdir "$old_data/snapshots/$(date -u +%F)"
    printf '{"failures":[]}\n' > "$old_data/snapshots/$(date -u +%F)/run.json"
    printf '{"original":true}\n' > "$old_data/snapshots/$(date -u +%F)/facts.json"
    stored_version=0.3
    if [ "$scenario" = same_day_deploy ]; then stored_version=0.2; fi
    printf '{"methodology_version":"%s"}\n' "$stored_version" > "$old_data/snapshots/$(date -u +%F)/scores.json"
  fi
  mode=daily
  case "$scenario" in fresh_deploy|same_day_deploy|partial_deploy|stale_methodology) mode=--deploy-locked ;; esac
  set +e
  bash "$root/run.sh" "$src" "$mode" > "$root/test.log" 2>&1
  status=$?
  set -e
  case "$scenario" in
    collection_failure|build_failure|public_failure|wrong_release|switch_failure|empty_tokens|invariant_failure|corrections_failure|partial_deploy|stale_methodology)
      [ "$status" -ne 0 ]
      [ "$(readlink "$root/current")" = "$old_web" ]
      [ "$(readlink "$root/shared/data")" = "$old_data" ]
      [ "$(readlink "$root/source")" = "$old_source" ]
      [ "$(find "$root/shared/generations" -mindepth 1 -maxdepth 1 -type d | wc -l)" -eq 1 ] ;;
    *)
      if [ "$scenario" = partial ]; then [ "$status" -eq 2 ]; else [ "$status" -eq 0 ]; fi
      [ "$(cat "$root/current/index.html")" = new ]
      [ "$(cat "$root/shared/data/evidence")" = original ]
      [ ! -e "$old_web" ] && [ ! -e "$old_data" ]
      [ ! -e "$root/shared/data/snapshots/$(date -u +%F)/raw" ]
      if [ "$scenario" = same_day ]; then [ ! -e "$root/shared/data/fresh" ]; else [ -e "$root/shared/data/fresh" ]; fi
      if [ "$scenario" = same_day_deploy ]; then
        grep -q '"original":true' "$root/shared/data/snapshot-revisions/"*/snapshot/facts.json
        grep -q '"methodology_version":"0.2"' "$root/shared/data/snapshot-revisions/"*/snapshot/scores.json
      fi ;;
  esac
  [ ! -e "$root/shared/publication.pending" ]
  echo "PASS $scenario"
done
