#!/usr/bin/env bash
# Portable migration/recovery test with nginx/systemctl/readlink doubled.
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
base=$(mktemp -d)
trap 'rm -rf -- "$base"' EXIT
node_bin=$(command -v node)
for scenario in success validation_failure reload_failure; do
  work="$base/$scenario"
  mkdir "$work"
  export work scenario
  cat > "$work/config" <<'CONF'
# Managed by Celebtir deployment
server {
    listen 443 ssl;
    location /data/v1/ {
        add_header Access-Control-Allow-Origin "*" always;
        try_files $uri =404;
    }
}
CONF
  cp "$work/config" "$work/original"
  cp "$repo/scripts/deploy/nginx-api.mjs" "$work/nginx-api.mjs"
  sed -e "s|config=/etc/nginx/sites-available/celebtir.conf|config='$work/config'|" "$repo/scripts/deploy/migrate-api.sh" > "$work/migrate.sh"
  readlink() { printf '%s\n' "$work/config"; }
  chown() { :; }
  nginx() {
    local n=0
    [ ! -f "$work/validations" ] || n=$(cat "$work/validations")
    n=$((n+1)); echo "$n" > "$work/validations"
    [ "$scenario" != validation_failure ] || [ "$n" -ne 2 ]
  }
  systemctl() {
    local n=0
    [ ! -f "$work/reloads" ] || n=$(cat "$work/reloads")
    n=$((n+1)); echo "$n" > "$work/reloads"
    [ "$scenario" != reload_failure ] || [ "$n" -ne 1 ]
  }
  export -f readlink chown nginx systemctl
  if bash "$work/migrate.sh" "$node_bin"; then
    [ "$scenario" = success ]
    grep -q '/data/v1/' "$work/config"
    grep -q '/data/v2/' "$work/config"
    [ "$(cat "$work/reloads")" = 1 ]
    bash "$work/migrate.sh" "$node_bin"
    [ "$(cat "$work/reloads")" = 1 ]
  else
    [ "$scenario" != success ]
    cmp "$work/config" "$work/original"
  fi
  echo "PASS $scenario (service commands mocked)"
done
