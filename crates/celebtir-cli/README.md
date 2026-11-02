# celebtir

The command-line tool behind [Celebtir](https://github.com/celebtir/celebtir), an open, evidence-linked scorecard of who controls each Solana DeFi protocol: who can upgrade its programs, how fast, who holds its admin keys and where its prices come from. Every rating links to the account it was read from.

## Install

You need Rust 1.88 or newer ([rustup.rs](https://rustup.rs)).

```bash
cargo install celebtir
```

Check what you installed. The version line names the commit it was built from:

```bash
celebtir --version
```

## Use

Commands read a registry folder (`registry/` by default: one YAML file per protocol, plus `tokens.yaml`, `oracles.yaml` and `controllers.yaml`) and write JSON.

| Command | What it does |
| --- | --- |
| `celebtir validate [--chain]` | Check the registry against its schema and rules; `--chain` also checks every program ID on mainnet |
| `celebtir collect` | Read the chain, OtterSec and DefiLlama for every protocol; write `facts.json` and `run.json` |
| `celebtir score` | Rate a `facts.json` with the methodology; no network |
| `celebtir diff` | Compare two days' snapshots and list the changes |
| `celebtir explain <protocol>` | Print one protocol's ratings with their reasons and evidence |
| `celebtir build-site-data` | Build the files the website and the `/data/v1` API serve |

Add `--format json` to any command for machine-readable output. Exit codes: `0` success, `1` invalid input (registry, arguments or files), `2` some protocols failed while the rest completed, `3` the RPC request budget ran out.

Set `CELEBTIR_RPC_URL` to your own Solana RPC endpoint. Without it, `collect` uses the rate-limited public endpoint, which is fine for a single protocol (`--protocol <id>`) but not for a full daily run.

## Licence

Code: MIT or Apache-2.0, at your option. Data published by Celebtir: CC BY 4.0.

Ratings describe on-chain configuration only. They are not an audit and not investment advice.
