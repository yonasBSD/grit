# t7602 merge octopus many

## 2026-06-05

- Claimed ticket `f9b67b`.
- Target test: `tests/t7602-merge-octopus-many.sh`.
- The remaining cases check octopus merge progress output, redundant-head reduction, and pretty ref names in fast-forward-plus-octopus output.
- Initial harness run was 2/5. The octopus commit graph was correct, but successful octopus merges printed commit-summary output instead of Git's `git-merge-octopus.sh` progress lines and diffstat.
- Added octopus progress output for first-head fast-forward and simple-merge steps, and changed final octopus success output to the strategy line plus diffstat.
- Honored `GIT_MERGE_VERBOSITY=0` for the merge commit summary line so reduced-head merges keep the strategy/diffstat output without the `[branch sha] subject` line.
- `./scripts/run-tests.sh t7602-merge-octopus-many.sh --verbose --timeout 180`: 5/5.
- `cargo fmt && cargo check -p grit-cli`: passed, with the known `diff.rs` unused-assignment warning.
- `cargo clippy --fix --allow-dirty`: exited 0, with the existing clippy warning backlog.
- `cargo test -p grit-lib --lib`: 252 passed, 2 known ignore glob failures.
- Regression harnesses: `t7600-merge.sh` 83/83 and `t7606-merge-custom.sh` 4/4.
