#!/usr/bin/env bash
# Complete deployment on an installed host: current application and fresh data.
set -euo pipefail
archive=$1
hash=$2
release=$3
root=/opt/celebtir
[[ "$release" =~ ^[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}$ ]]
[[ "$hash" =~ ^[a-f0-9]{64}$ ]]
[ "$(id -u)" -eq 0 ]
for link in source current shared/data; do
  [ -L "$root/$link" ] && [ -d "$root/$link" ] || { echo 'Installed deployment required; run deploy.ps1 -Mode Install first.' >&2; exit 1; }
done
[ -f /etc/celebtir/collector.env ]
[ -f /etc/nginx/sites-enabled/celebtir.conf ]
[ -f /etc/systemd/system/celebtir-update.service ]
[ -f /etc/systemd/system/celebtir-update.timer ]
[ -f /etc/systemd/system/celebtir-certificate.timer ]
worker_home="$root/home"
worker_path="$root/tools/node/bin:$worker_home/.local/bin:$worker_home/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
for tool in "$root/tools/node/bin/node" "$root/tools/node/bin/npm" "$worker_home/.cargo/bin/rustup"; do
  [ -x "$tool" ] || { echo 'Installed tool missing; run deploy.ps1 -Mode Install.' >&2; exit 1; }
done
printf '%s  %s\n' "$hash" "$archive" | sha256sum -c -
exec 9>"$root/shared/update.lock"
flock -n 9 || { echo 'Another deployment/update is running.' >&2; exit 1; }
src="$root/releases/$release"
mkdir "$src"
cleanup() {
  result=$?
  if [ "$archive" = "/tmp/celebtir-$release/payload.tar.gz" ]; then rm -f -- "$archive"; fi
  # A pending journal owns its candidate releases; preserve them for recovery.
  if [ ! -f "$root/shared/publication.pending" ] && [ "$(readlink -f "$root/source")" != "$src" ]; then
    rm -rf -- "$src"
  fi
  exit "$result"
}
trap cleanup EXIT
tar -xzf "$archive" -C "$src" --no-same-owner
find "$src/scripts" -type f -name '*.sh' -exec sed -i 's/\r$//' {} +
printf 'release=%s\npayload_sha256=%s\nmode=deploy\n' "$release" "$hash" > "$src/DEPLOYMENT.txt"
chown -R celebtir:celebtir "$src"
source "$src/scripts/deploy/publication.sh"
publication_recover
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" bash -ec 'set -a; source /etc/celebtir/collector.env; set +a; node "$1/scripts/deploy/rpc-preflight.mjs"' bash "$src"
runuser -u celebtir -- env PATH="$worker_path" bash "$src/scripts/tests/deploy_test.sh"
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" node --test "$src/scripts/tests/release-data.test.mjs"
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" bash "$src/scripts/deploy/build.sh" "$src"
install -d -o celebtir -g celebtir "$src/bin"
install -o celebtir -g celebtir -m 755 "$src/target/release/celebtir" "$src/bin/celebtir"
rm -rf -- "$src/target"
bash "$src/scripts/deploy/migrate-api.sh" "$root/tools/node/bin/node"
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" bash "$src/scripts/deploy/run.sh" "$src" --deploy-locked
# run.sh verifies the public API and exact release before committing/pruning.
runuser -u celebtir -- env HOME="$worker_home" PATH="$worker_path" node "$src/scripts/deploy/storage.mjs" prune sources
flock -u 9
systemctl enable --now celebtir-update.timer celebtir-certificate.timer
systemctl is-active --quiet celebtir-update.timer celebtir-certificate.timer
echo 'Complete deployment verified: fresh snapshot, current methodology and application; daily timers active.'
