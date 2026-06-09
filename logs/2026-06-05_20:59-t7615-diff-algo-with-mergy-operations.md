# t7615 diff algorithm with merge operations

## 2026-06-05

- Claimed ticket `7205d6`.
- Target test: `tests/t7615-diff-algo-with-mergy-operations.sh`.
- Initial harness run was 5/7. The merge cases already honored `-Xdiff-algorithm=histogram` and `diff.algorithm=histogram`; the cherry-pick cases still used the default Myers text merge.
- Threaded an optional diff algorithm through `grit-lib::merge_trees::merge_trees_three_way` and its textual merge sites.
- Updated cherry-pick strategy option parsing to accept `diff-algorithm=<name>` plus the documented `histogram` and `patience` aliases, with `diff.algorithm` config as fallback.
- `cargo build --release -p grit-cli`: passed with known warnings.
- `./scripts/run-tests.sh t7615-diff-algo-with-mergy-operations.sh --verbose --timeout 180`: 7/7.
- Regression harnesses: `t3501-revert-cherry-pick.sh` 21/21 and `t3508-cherry-pick-many-commits.sh` 14/14.
- `cargo clippy --fix --allow-dirty`: exited 0 with the existing clippy warning backlog and failed-autofix diagnostics; reverted one unrelated autofix in `promisor_remote`.
- `cargo test -p grit-lib --lib`: 252 passed, 2 known ignore glob failures.
- `cargo fmt && cargo check -p grit-cli`: passed, with known warnings.
