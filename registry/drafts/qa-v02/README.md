# QA v0.2 registry review packet

Drafted 2026-10-21. Not loaded by the registry. **Do not activate before 2026-10-22 19:15 UTC and explicit owner review.** Recheck current authorities before merging.

- `authority-pdas.json`: 18 mint declarations. All derive to the authority in the original snapshot and in fresh finalized RPC reads at slot 459735190; controlling programs are executable. SPL/Sanctum pool seeds, Marinade state/st_mint and Wormhole mint_signer are cited. Run `cargo run -p celebtir-collect-solana --example verify_pda_draft -- registry/drafts/qa-v02/authority-pdas.json` for offline derivation checks.
- `on-chain-accounts.json`: ordered account keys and raw public RPC result. `programdata-response.json` is the subsequent upgrade-header read; its keys are in `programdata-request.json`. No ratings were changed from these drafts.
- `jupiter-lend-oracles.json`: 39 composite oracle accounts, owner checked against Jupiter's oracle program; source arrays decoded from the vendored Oracle/Sources IDL. Vault relationships come from the public Lend API and need review against vault configurations before activation.
- `gmx-oracles.json`: 78 token configurations decoded from the Store-referenced token map, including enabled flags and expected-provider indices. TokenMapHeader layout comes from the vendored IDL; 328-byte TokenConfig / 64-byte FeedConfig layouts come from the cited source. Provider semantics and individual feed decoding still need review.
- `jupiter-perps-doves.json`: primary and Doves account references decoded from five custody accounts with the vendored IDL. Doves primary/fallback/aggregation roles remain unverified; no role is guessed.
- `cbBTC.json`: proposed use of the existing bridged category for wrapped BTC, without asserting a bridge controller.

The Sanctum list now names the captured fwdSOL mint **dumSOL**. This identity drift needs separate review; the draft retains the snapshot symbol and verified mint address.

Evidence endpoint: public `https://api.mainnet-beta.solana.com`, finalized commitment, 2026-10-21. Captures are bounded account reads; no transactions were submitted. Unknown oracle scores remain Unknown until supported registry entries are reviewed and activated.
