// UI-5, HUI-2: evidence links open the chain's explorer. Solana addresses go
// to Solscan; 0x addresses to HyperEVMScan, or to the Hyperliquid explorer
// for HyperCore (HIP-3 deployers). The pipeline writes the same URLs into
// every Evidence record; tests check they agree.

const BASE = "https://solscan.io";
const HYPEREVM = "https://hyperevmscan.io";
const HYPERCORE = "https://app.hyperliquid.xyz/explorer";

export type ExplorerChain = "solana" | "hyper_evm" | "hyper_core";

export const accountUrl = (address: string, chain?: string) => {
  if (chain === "hyper_core") return `${HYPERCORE}/address/${address}`;
  if (chain === "hyper_evm" || (chain === undefined && address.startsWith("0x"))) {
    return `${HYPEREVM}/address/${address}`;
  }
  return `${BASE}/account/${address}`;
};
export const txUrl = (signature: string) => `${BASE}/tx/${signature}`;
