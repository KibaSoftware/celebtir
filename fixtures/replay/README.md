# Replay fixtures (SNAP-4)

Published inputs and outputs kept byte for byte, so every methodology version
keeps re-scoring exactly as it was published:

- `v0.1/golden-2026-10-17/`: the offline golden pipeline's facts and scores as
  published under methodology v0.1, before the collector learned program-keyed
  sources, program-derived addresses and stake-pool authorities (launch audit).
- `v0.2/2026-10-21/`: the 2026-10-21 snapshot rebuilt under methodology v0.2
  (`output/qa-v02-data`), the dataset the first hosted deployment was seeded with.

Never regenerate these files; tests re-score `facts.json` with the recorded
methodology and require the committed `scores.json`.
