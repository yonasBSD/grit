# t1092 sparse-checkout compatibility

## 2026-06-05

- Claimed ticket `0e0b0d`.
- Starting from a clean GitButler workspace on `grit-t1-progress`.
- Refreshing `t1092-sparse-checkout-compatibility.sh` before inspecting failures.
- Baseline remained `63/106`.
- Moved sparse checkout warning detection before worktree updates in `checkout`; harness improved to
  `67/106`, clearing false warnings for files checkout created inside the sparse cone.
- Taught checkout to recognize `--patch` after the tree-ish, accepted hyphen-leading commit
  messages (`commit -m "-a"`), skipped absent skip-worktree entries during `commit -a`, allowed
  `add --refresh` to refresh sparse entries, and honored `add --sparse .` for out-of-cone paths.
- Latest harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `70/106`.
  Ticket remains open; next direct failure is still within `status/add: outside sparse cone`.
