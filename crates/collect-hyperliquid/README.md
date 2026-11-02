# celebtir-collect-hyperliquid

The Hyperliquid collector for [Celebtir](https://github.com/Kibadachi/celebtir), wave 2 (ADR 024):

- **HyperEVM:** JSON-RPC reads at one pinned block. It reads the EIP-1967 proxy slots, then a Safe's threshold, owners and modules, or a `TimelockController`'s delay and roles. Verified source comes from Sourcify, then Etherscan v2.
- **HIP-3:** every builder-deployed perp DEX from the info API (`perpDexs`), and each deployer's stake.

A read that fails becomes `Unavailable`; nothing is guessed. Tests replay the recorded responses in `fixtures/hyperliquid/` and never touch the network.
