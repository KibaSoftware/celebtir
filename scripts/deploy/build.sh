#!/usr/bin/env bash
# Shared application build for both deployment choices. Run as celebtir.
set -euo pipefail
cd "$1"
node scripts/deploy/release-data.mjs methodology "$PWD"
rust_version=$(node -e 'const fs=require("fs");const v=/^channel\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"/m.exec(fs.readFileSync("rust-toolchain.toml","utf8"));if(!v)process.exit(1);console.log(v[1])')
pnpm_version=$(node -e 'const p=require("./site/package.json").packageManager;if(!/^pnpm@[0-9]+\.[0-9]+\.[0-9]+$/.test(p))process.exit(1);console.log(p)')
node -e 'if(Number(process.versions.node.split(".")[0])<22)throw new Error("Node 22+ required; run Install")'
rustup toolchain install "$rust_version" --profile minimal
npm install --prefix "$HOME/.local" --global "$pnpm_version"
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo +"$rust_version" build --locked --release -p celebtir
pnpm -C site install --frozen-lockfile
