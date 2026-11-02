#!/usr/bin/env bash
# Root-only bootstrap, called over ssh -tt. Never modifies another site's files.
set -euo pipefail
archive=$1
hash=$2
release=$3
email=$4
root=/opt/celebtir
[[ "$release" =~ ^[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}$ ]]
[[ "$hash" =~ ^[a-f0-9]{64}$ ]]
[ "$(id -u)" -eq 0 ]
source /etc/os-release
[ "$ID" = ubuntu ] || { echo 'This bootstrap supports Ubuntu only.' >&2; exit 1; }
if [ "$VERSION_ID" = 25.04 ]; then
  echo 'WARNING: Ubuntu 25.04 is end-of-life. Plan a supported OS upgrade separately.' >&2
fi
printf '%s  %s\n' "$hash" "$archive" | sha256sum -c -
# Bootstrap packages only when needed. Do not upgrade system packages or rewrite
# apt sources on the existing Goblinz host (including EOL Ubuntu repositories).
missing=()
for pair in 'curl:curl' 'xz:xz-utils' 'cc:build-essential' 'pkg-config:pkg-config' 'nginx:nginx' 'certbot:certbot'; do
  command -v "${pair%%:*}" >/dev/null || missing+=("${pair#*:}")
done
dpkg-query -W -f='${Status}' libssl-dev 2>/dev/null | grep -q 'install ok installed' || missing+=(libssl-dev)
if [ "${#missing[@]}" -gt 0 ]; then
  apt-get update
  apt-get install -y --no-upgrade "${missing[@]}" ca-certificates
fi
nginx -t
systemctl enable --now nginx
if ! id celebtir >/dev/null 2>&1; then
  useradd --system --user-group --home-dir "$root/home" --shell /usr/sbin/nologin celebtir
fi
install -d -o celebtir -g celebtir -m 755 "$root" "$root/home" "$root/shared" "$root/shared/generations" "$root/releases" "$root/web" "$root/tools"
touch "$root/shared/update.lock"
chown celebtir:celebtir "$root/shared/update.lock"
exec 9>"$root/shared/update.lock"
flock -n 9 || { echo 'Another Celebtir deployment/update is running.' >&2; exit 1; }
src="$root/releases/$release"
mkdir "$src"
cleanup_bootstrap() {
  result=$?
  if [ "$result" -ne 0 ] && [ -f "$root/shared/publication.pending" ]; then
    source "$src/scripts/deploy/publication.sh"
    if ! publication_recover; then echo 'Recovery failed; keeping all releases for repair.' >&2; exit 1; fi
  fi
  if [ "$result" -ne 0 ] && [ "${nginx_touched:-no}" = yes ]; then
    if [ -f "$config.before-$release" ]; then cp -a "$config.before-$release" "$config";
    else rm -f /etc/nginx/sites-enabled/celebtir.conf "$config"; fi
    nginx -t && systemctl reload nginx || echo 'Nginx recovery requires inspection.' >&2
  fi
  if [ "$archive" = "/tmp/celebtir-$release/payload.tar.gz" ]; then rm -f -- "$archive"; fi
  # Failed builds must not accumulate compiler trees. Once this release is the
  # active source, retain it for repair/retry even if certificate setup failed.
  if [ "$result" -ne 0 ] && [ "$(readlink -f "$root/source" 2>/dev/null || true)" != "$src" ]; then
    case "$src" in "$root"/releases/"$release") rm -rf -- "$src" ;; esac
  fi
  exit "$result"
}
trap cleanup_bootstrap EXIT
tar -xzf "$archive" -C "$src" --no-same-owner
printf 'release=%s\npayload_sha256=%s\n' "$release" "$hash" > "$src/DEPLOYMENT.txt"
# Normalize source scripts packaged from Windows; no published data is rewritten.
find "$src/scripts" -type f -name '*.sh' -exec sed -i 's/\r$//' {} +
chown -R celebtir:celebtir "$src"
install -d -o root -g celebtir -m 750 /etc/celebtir
if [ ! -f /etc/celebtir/collector.env ]; then
  echo 'Enter a Solana RPC URL for daily collection (input hidden; blank uses public RPC).'
  read -r -s -p 'Solana RPC URL: ' rpc </dev/tty; echo
  read -r -s -p 'HyperEVM RPC URL (optional): ' hyper </dev/tty; echo
  read -r -s -p 'Etherscan API key (optional): ' etherscan </dev/tty; echo
  (umask 027; printf 'CELEBTIR_RPC_URL=%q\nCELEBTIR_HYPEREVM_RPC_URL=%q\nCELEBTIR_ETHERSCAN_KEY=%q\n' "$rpc" "$hyper" "$etherscan" > /etc/celebtir/collector.env)
  chown root:celebtir /etc/celebtir/collector.env
  chmod 640 /etc/celebtir/collector.env
  unset rpc hyper etherscan
else
  read -r -p 'Replace saved Solana RPC URL? [y/N]: ' replace_rpc </dev/tty
  if [[ "$replace_rpc" = y || "$replace_rpc" = Y ]]; then
  # Replace only the requested Solana credential; retain other provider settings.
  source /etc/celebtir/collector.env
  echo 'Paste the complete Solana RPC URL including its API key, without quotes or VARIABLE=.'
  read -r -s -p 'Replacement Solana RPC URL: ' rpc </dev/tty; echo
  (umask 027; printf 'CELEBTIR_RPC_URL=%q\nCELEBTIR_HYPEREVM_RPC_URL=%q\nCELEBTIR_ETHERSCAN_KEY=%q\n' "$rpc" "${CELEBTIR_HYPEREVM_RPC_URL:-}" "${CELEBTIR_ETHERSCAN_KEY:-}" > /etc/celebtir/collector.env.new)
  chown root:celebtir /etc/celebtir/collector.env.new
  chmod 640 /etc/celebtir/collector.env.new
  mv -f /etc/celebtir/collector.env.new /etc/celebtir/collector.env
  unset rpc CELEBTIR_RPC_URL CELEBTIR_HYPEREVM_RPC_URL CELEBTIR_ETHERSCAN_KEY
  fi
fi
# Install Node in Celebtir's own tools directory; do not change Goblinz's Node.
if [ ! -x "$root/tools/node/bin/node" ]; then
  case "$(uname -m)" in x86_64) arch=x64 ;; aarch64) arch=arm64 ;; *) echo 'Unsupported CPU'; exit 1 ;; esac
  tmp=$(mktemp -d)
  curl -fsSLo "$tmp/SHASUMS256.txt" https://nodejs.org/dist/latest-v22.x/SHASUMS256.txt
  node_file=$(awk -v arch="$arch" '$2 ~ ("-linux-" arch "\\.tar\\.xz$") {print $2}' "$tmp/SHASUMS256.txt")
  [[ "$node_file" =~ ^node-v22\.[0-9]+\.[0-9]+-linux-(x64|arm64)\.tar\.xz$ ]]
  curl -fsSLo "$tmp/$node_file" "https://nodejs.org/dist/latest-v22.x/$node_file"
  (cd "$tmp"; grep "  $node_file$" SHASUMS256.txt | sha256sum -c -)
  install -d "$root/tools/node.install-$release"
  tar -xJf "$tmp/$node_file" -C "$root/tools/node.install-$release" --strip-components=1
  "$root/tools/node.install-$release/bin/node" --version
  if [ -d "$root/tools/node" ]; then rm -rf -- "$root/tools/node"; fi
  mv "$root/tools/node.install-$release" "$root/tools/node"
  rm -f "$tmp/$node_file" "$tmp/SHASUMS256.txt"; rmdir "$tmp"
fi
worker_home="$root/home"
worker_path="$root/tools/node/bin:$worker_home/.local/bin:$worker_home/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
# Authenticate a read-only batch request before spending time compiling.
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" bash -ec 'set -a; source /etc/celebtir/collector.env; set +a; node "$1/scripts/deploy/rpc-preflight.mjs"' bash "$src"
# Tests use the installed Node to exercise actual retention, not a no-op mock.
runuser -u celebtir -- env PATH="$worker_path" bash "$src/scripts/tests/deploy_test.sh"
if [ ! -x "$worker_home/.cargo/bin/rustup" ]; then
  curl -fsSLo "$src/rustup-init.sh" https://sh.rustup.rs
  runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" sh "$src/rustup-init.sh" -y --no-modify-path --profile minimal --default-toolchain 1.88.0
fi
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" node --test "$src/scripts/tests/storage.test.mjs" "$src/scripts/tests/rpc-preflight.test.mjs" "$src/scripts/tests/release-data.test.mjs"
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" bash "$src/scripts/deploy/build.sh" "$src"
# Retain the runnable binary, not Cargo's compiler artifacts. Daily runs only
# execute this binary; a later source deployment builds a new one.
install -d -o celebtir -g celebtir "$src/bin"
install -o celebtir -g celebtir -m 755 "$src/target/release/celebtir" "$src/bin/celebtir"
case "$src" in "$root"/releases/"$release") rm -rf -- "$src/target" ;; *) exit 1 ;; esac
if [ ! -e "$root/shared/data" ]; then
  seed="$root/shared/generations/seed-$release"
  mkdir -p "$seed/snapshots"
  printf 'Fresh installation; no QA data seeded. Payload SHA256: %s\n' "$hash" > "$seed/INSTALL-PROVENANCE.txt"
  chown -R celebtir:celebtir "$seed"
  ln -s "$seed" "$root/shared/data"
fi
[ -L "$root/shared/data" ] || { echo 'Existing data is not a managed symlink; refusing to replace it.' >&2; exit 1; }
set +e
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" bash "$src/scripts/deploy/run.sh" "$src" --locked
status=$?
set -e
case "$status" in 0|2) ;; *) exit "$status" ;; esac
install -d -m 755 /var/lib/celebtir-acme
config=/etc/nginx/sites-available/celebtir.conf
if [ -e /etc/nginx/sites-enabled/celebtir.conf ] || [ -L /etc/nginx/sites-enabled/celebtir.conf ]; then
  [ "$(readlink /etc/nginx/sites-enabled/celebtir.conf)" = "$config" ] || {
    echo 'Refusing to replace an unmanaged enabled Celebtir configuration.' >&2; exit 1;
  }
fi
if [ -e "$config" ] && ! grep -q '^# Managed by Celebtir deployment$' "$config"; then
  echo "Refusing to overwrite unmanaged $config" >&2; exit 1
fi
# Reject a conflicting server_name in any other enabled configuration.
if grep -R -E 'server_name[^;]*(celebtir\.com)' /etc/nginx/sites-enabled /etc/nginx/conf.d 2>/dev/null | grep -v '/celebtir.conf:'; then
  echo 'Existing Celebtir virtual host found; reconcile it before deploying.' >&2; exit 1
fi
if [ -f "$config" ]; then cp -a "$config" "$config.before-$release"; fi
nginx_touched=yes
write_config() {
  local tls=$1
  cat > "$config" <<'NGINX'
# Managed by Celebtir deployment
server {
    listen 80;
    server_name celebtir.com www.celebtir.com;
    location ^~ /.well-known/acme-challenge/ { root /var/lib/celebtir-acme; }
    location / { return 301 https://celebtir.com$request_uri; }
}
NGINX
  if [ "$tls" = yes ]; then
    cat >> "$config" <<'NGINX'
server {
    listen 443 ssl;
    server_name www.celebtir.com;
    ssl_certificate /etc/letsencrypt/live/celebtir.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/celebtir.com/privkey.pem;
    return 301 https://celebtir.com$request_uri;
}
server {
    listen 443 ssl;
    server_name celebtir.com;
    ssl_certificate /etc/letsencrypt/live/celebtir.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/celebtir.com/privkey.pem;
    root /opt/celebtir/current;
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    index index.html;
    add_header X-Content-Type-Options nosniff always;
    # Keep the existing prelaunch indexing policy until the owner enables indexing.
    add_header X-Robots-Tag "noindex, nofollow" always;
    location / { try_files $uri $uri/ =404; }
    location /data/v2/ {
        add_header Access-Control-Allow-Origin "*" always;
        add_header X-Robots-Tag "noindex, nofollow" always;
        add_header X-Content-Type-Options nosniff always;
        try_files $uri =404;
    }
    location ~ \.(md|txt|atom)$ {
        types { application/atom+xml atom; }
        default_type text/plain;
        add_header Access-Control-Allow-Origin "*" always;
        add_header X-Robots-Tag "noindex, nofollow" always;
        add_header X-Content-Type-Options nosniff always;
        try_files $uri =404;
    }
    location /_next/static/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        add_header X-Robots-Tag "noindex, nofollow" always;
        add_header X-Content-Type-Options nosniff always;
    }
    error_page 404 /404.html;
    location = /404.html { internal; }
}
NGINX
  fi
}
reload_checked() {
  if ! nginx -t; then
    if [ -f "$config.before-$release" ]; then cp -a "$config.before-$release" "$config"; else rm -f /etc/nginx/sites-enabled/celebtir.conf "$config"; fi
    echo 'Nginx validation failed; restored previous configuration.' >&2
    exit 1
  fi
  systemctl reload nginx
}
if [ -s /etc/letsencrypt/live/celebtir.com/fullchain.pem ]; then write_config yes; else write_config no; fi
ln -sfn "$config" /etc/nginx/sites-enabled/celebtir.conf
reload_checked
# Webroot issuance leaves other virtual hosts untouched. DNS must already point
# both names here and public TCP 80/443 must be allowed by host/provider firewalls.
certbot certonly --webroot -w /var/lib/celebtir-acme --cert-name celebtir.com -d celebtir.com -d www.celebtir.com --email "$email" --agree-tos --non-interactive --keep-until-expiring
write_config yes
reload_checked
install -d /etc/letsencrypt/renewal-hooks/deploy
printf '#!/bin/sh\nnginx -t && systemctl reload nginx\n' > /etc/letsencrypt/renewal-hooks/deploy/celebtir-nginx
chmod 755 /etc/letsencrypt/renewal-hooks/deploy/celebtir-nginx
# Own renewal timer: reuse installed certbot without replacing its package or
# altering other certificates. Certbot serializes concurrent renewal commands.
certbot_bin=$(command -v certbot)
cat > /etc/systemd/system/celebtir-certificate.service <<EOF
[Unit]
Description=Renew Celebtir certificate
[Service]
Type=oneshot
ExecStart=$certbot_bin renew --cert-name celebtir.com --quiet
EOF
cat > /etc/systemd/system/celebtir-certificate.timer <<'EOF'
[Unit]
Description=Check Celebtir certificate twice daily
[Timer]
OnCalendar=*-*-* 00,12:00:00 UTC
RandomizedDelaySec=3600
Persistent=true
[Install]
WantedBy=timers.target
EOF
cat > /etc/systemd/system/celebtir-update.service <<'EOF'
[Unit]
Description=Collect and publish Celebtir daily snapshot
Wants=network-online.target
After=network-online.target
[Service]
Type=oneshot
User=celebtir
Group=celebtir
ExecStart=/bin/bash /opt/celebtir/source/scripts/deploy/run.sh
TimeoutStartSec=2h
Nice=10
IOSchedulingClass=best-effort
IOSchedulingPriority=7
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/celebtir
ProtectHome=true
EOF
cat > /etc/systemd/system/celebtir-update.timer <<'EOF'
[Unit]
Description=Daily Celebtir collection at 02:17 UTC
[Timer]
OnCalendar=*-*-* 02:17:00 UTC
Persistent=true
[Install]
WantedBy=timers.target
EOF
systemctl daemon-reload
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" node "$src/site/scripts/check-api.mjs" https://celebtir.com
expected_id=$(cat "$root/current/deployment-id.txt")
[ "$(curl --connect-timeout 10 --max-time 30 -fsS "https://celebtir.com/deployment-id.txt?release=$expected_id")" = "$expected_id" ]
curl -fsSI https://www.celebtir.com/ | grep -i '^location: https://celebtir.com/'
source "$src/scripts/deploy/publication.sh"
publication_commit
nginx_touched=no
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" node "$src/scripts/deploy/storage.mjs" prune sources
# Verification and cleanup finish under the same lock; only then enable timers.
flock -u 9
systemctl enable --now celebtir-update.timer celebtir-certificate.timer
systemctl is-active --quiet celebtir-update.timer celebtir-certificate.timer
echo 'Celebtir deployed. Inspect journalctl -u celebtir-update.service for scheduled runs.'
