#!/usr/bin/env bash
# Root-only, additive migration of the managed Celebtir virtual host.
set -euo pipefail
node_bin=$1
config=/etc/nginx/sites-available/celebtir.conf
enabled=/etc/nginx/sites-enabled/celebtir.conf
[ "$(readlink -f "$enabled")" = "$config" ] || { echo 'Unmanaged Celebtir Nginx link; refusing migration.' >&2; exit 1; }
nginx -t
candidate=$(mktemp "${config}.candidate.XXXXXX")
backup=$(mktemp "${config}.backup.XXXXXX")
changed=no
cleanup() {
  result=$?
  if [ "$result" -ne 0 ] && [ "$changed" = yes ]; then
    cp -p "$backup" "$config"
    nginx -t && systemctl reload nginx || echo 'Nginx recovery requires inspection.' >&2
  fi
  rm -f -- "$candidate" "$backup"
  exit "$result"
}
trap cleanup EXIT
cp -p "$config" "$backup"
"$node_bin" "$(dirname "$0")/nginx-api.mjs" "$config" "$candidate"
if cmp -s "$config" "$candidate"; then exit 0; fi
chmod --reference="$config" "$candidate"
chown --reference="$config" "$candidate"
changed=yes
mv -f "$candidate" "$config"
nginx -t
systemctl reload nginx
echo 'Added Celebtir API v2 CORS route; existing routes and TLS configuration preserved.'
