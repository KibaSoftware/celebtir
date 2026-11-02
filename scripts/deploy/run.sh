#!/usr/bin/env bash
# Runs as the dedicated celebtir user. Bootstrap holds the lock for --locked.
set -euo pipefail
root=/opt/celebtir
source_dir="${1:-$root/source}"
mode=${2:-daily}
case "$mode" in daily|--locked|--deploy-locked) ;; *) echo 'Invalid run mode' >&2; exit 1 ;; esac
if [ "$mode" = daily ]; then
  exec 9>"$root/shared/update.lock"
  flock -n 9 || { echo 'Another deployment/update is running.' >&2; exit 1; }
fi
worker_home="$root/home"
export PATH="$root/tools/node/bin:$worker_home/.local/bin:$worker_home/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
set -a
# This file is written only by root; never upload or source a local .env.
source /etc/celebtir/collector.env
set +a
export CELEBTIR_SITE_URL=https://celebtir.com
export CELEBTIR_REPO_URL="${CELEBTIR_REPO_URL:-https://github.com/KibaSoftware/celedir}"
export CARGO_BUILD_JOBS=1
cd "$source_dir"
source_dir="$(pwd -P)"
source scripts/deploy/publication.sh
publication_recover
# Recovery may have restored the scheduled source pointer. Explicit bootstrap
# sources still select the candidate release being installed.
if [ "$#" -eq 0 ]; then
  cd "$root/source"
  source_dir="$(pwd -P)"
fi
id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
candidate="$root/shared/generations/$id"
mkdir "$candidate"
published=0
preview=''
web=''
cleanup() {
  result=$?
  if [ -n "$preview" ]; then kill "$preview" 2>/dev/null || true; fi
  if [ "$published" -eq 0 ]; then
    if ! publication_recover; then
      echo 'Publication recovery failed; retaining all candidates for repair.' >&2
      exit 1
    fi
    node "$source_dir/scripts/deploy/storage.mjs" discard "$candidate" || true
    if [ -n "$web" ] && [ "$(readlink -f "$root/current" 2>/dev/null || true)" != "$web" ]; then
      node "$source_dir/scripts/deploy/storage.mjs" discard-web "$web" || true
    fi
  fi
  # These exact paths are disposable outputs within this managed source release.
  case "$(readlink -f "$source_dir")" in "$root"/releases/*) rm -rf -- "$source_dir/site/.next" "$source_dir/site/out" "$source_dir/site/public/data" "$source_dir/site/public/feeds" ;; esac
  exit "$result"
}
trap cleanup EXIT
cp -a "$root/shared/data/." "$candidate/"
today="$(date -u +%F)"
status=0
methodology=$(node scripts/deploy/release-data.mjs methodology "$source_dir")
if [ "$mode" != daily ]; then
  node scripts/deploy/release-data.mjs archive "$candidate" "$today" "$id"
fi
if [ ! -d "$candidate/snapshots/$today" ]; then
  set +e
  bash scripts/daily.sh --date "$today" --bin "$source_dir/bin/celebtir" --data "$candidate" --methodology "$methodology" --hyperliquid
  status=$?
  set -e
  case "$status" in 0|2) ;; *) echo 'Collection failed; unpublished candidate will be discarded.' >&2; exit "$status" ;; esac
  if [ "$mode" != daily ] && [ "$status" -ne 0 ]; then
    echo 'Complete deployment requires a successful collection; live release unchanged. Fix RPC/evidence collection failures and retry.' >&2
    exit 1
  fi
else
  echo "Snapshot $today already exists; preserving it and rebuilding derived outputs."
  "$source_dir/bin/celebtir" build-site-data --data "$candidate" --as-of "$today"
  status="$(node "$source_dir/scripts/deploy/storage.mjs" status "$candidate/snapshots/$today/run.json")"
fi
node "$source_dir/scripts/deploy/storage.mjs" check-export "$candidate/latest/facts.json"
node "$source_dir/scripts/deploy/storage.mjs" trim "$candidate"
export CELEBTIR_DATA="$candidate"
pnpm -C site build
pnpm -C site budget
node site/scripts/check-release.mjs site/out
node scripts/deploy/release-data.mjs check "$candidate" "$source_dir" "$today" site/out
test -s site/out/index.html
test -s site/out/404.html
test -s site/out/data/v2/summary.json
printf '%s\n' "$id" > site/out/deployment-id.txt
# Validate the generated API before switching any public files.
(cd site; node scripts/serve-out.mjs 13210) &
preview=$!
for attempt in $(seq 1 30); do
  kill -0 "$preview"
  if [ "$(curl -fsS http://127.0.0.1:13210/deployment-id.txt || true)" = "$id" ]; then break; fi
  sleep 1
done
[ "$(curl -fsS http://127.0.0.1:13210/deployment-id.txt)" = "$id" ]
node site/scripts/check-api.mjs http://127.0.0.1:13210
node celebtir-invariants.mjs http://127.0.0.1:13210
kill "$preview"
wait "$preview" 2>/dev/null || true
preview=''
web="$root/web/$id"
mv site/out "$web"
# Keep old published trees only while preparing the new report.
publication_begin "$candidate" "$web" "$source_dir"
# Initial bootstrap verifies HTTPS after Nginx/certificate setup. Scheduled
# updates must verify the exact public report before committing and pruning.
if [ "${2:-}" != --locked ]; then
  node site/scripts/check-api.mjs https://celebtir.com
  [ "$(curl --connect-timeout 10 --max-time 30 -fsS "https://celebtir.com/deployment-id.txt?release=$id")" = "$id" ]
  publication_commit
  node "$source_dir/scripts/deploy/storage.mjs" prune
fi
published=1
# Old deployments may have left this rollback alias; it no longer owns data.
if [ -L "$root/previous" ]; then rm "$root/previous"; fi
printf 'published=%s\ncollection_exit=%s\n' "$id" "$status" > "$root/shared/last-run.txt"
if [ "$status" -eq 2 ]; then
  echo 'Published with partial collection failures; inspect run.json and journal.' >&2
fi
exit "$status"
