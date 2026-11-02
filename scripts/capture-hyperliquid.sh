#!/usr/bin/env bash
# Records the Hyperliquid fixtures under fixtures/hyperliquid/ (HOPS-2,
# ADR 009). Uses only the public info API and HyperEVM RPC (or
# CELEBTIR_HYPEREVM_RPC_URL). Re-capturing changes blocks and maybe values,
# so commit re-captures deliberately and re-run the tests.
set -euo pipefail
cd "$(dirname "$0")/.."
cap() { cargo run -q -p celebtir-collect-hyperliquid --example capture -- "$@"; }

cap info perp-dexs '{"type":"perpDexs"}' \
  --note "Every HIP-3 DEX: deployer, oracle updater, sub-deployers, caps (HIP-1)"
cap info delegator-summary-xyz \
  '{"type":"delegatorSummary","user":"0x88806a71d74ad0a510b350545c9ae490912f0888"}' \
  --note "Stake of the xyz DEX deployer (HIP-4)"

# Safe singletons from safe-global/safe-deployments: a code-size check shows
# whether each is deployed on chain 999 before controllers.yaml lists it.
cap evm safe-singletons '[
  ["eth_getCode", ["0x3E5c63644E683549055b9Be8653de26E0B4CD36E", "BLOCK"]],
  ["eth_getCode", ["0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552", "BLOCK"]],
  ["eth_getCode", ["0x29fcB43b46531BcA003ddC8FCB67FFE91900C762", "BLOCK"]],
  ["eth_getCode", ["0x41675C099F32341bf84BFc5382aF534df5C7461a", "BLOCK"]]
]' --note "Safe v1.3.0 L2, v1.3.0, v1.4.1 L2, v1.4.1 singletons (HREG-3)"
