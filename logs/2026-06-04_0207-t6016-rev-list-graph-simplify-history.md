# t6016 rev-list graph simplify history

Claimed `t6016-rev-list-graph-simplify-history.sh` from `t6-plan.md` at 4/12 passing.

Initial focus:

- Run the current harness/direct test to identify failing graph simplification behavior.
- Read the local and upstream test plus the related rev-list/log documentation.
- Search existing rev-list graph/simplify code paths before changing traversal behavior.

Findings:

- The official harness was stale in `t6-plan.md`: current release started at 11/12, with only
  `--graph --full-history --simplify-merges --all -- bar.txt` failing.
- Grit dropped merge `A6` before graph rendering because path-limited simplify-merges filtered
  merges that were TREESAME to their first parent.
- Broadly preserving all simplify-merges merges regressed non-graph simplification (`t6012`,
  `t6111`), so the fix must be graph-specific.

Changes:

- Added a `RevListOptions` flag for graph-mode simplify-merges preservation.
- Set that flag from both normal log and graph log option construction when `--graph` is active.
- In graph mode only, keep path-limited simplify-merges merge nodes through the rev-list path
  filter, then protect the last TREESAME parent during graph merge-parent pruning so the graph can
  still draw the necessary lanes.

Validation:

- `cargo build --release -p grit-cli` completed with the existing warning backlog.
- `cargo check -p grit-cli` completed with the existing warning backlog.
- `cargo test -p grit-lib --lib` passed 238/238.
- `cargo clippy --fix --allow-dirty` completed with the existing warning backlog; unrelated
  clippy auto-fixes were restored before committing.
- `./scripts/run-tests.sh t6016-rev-list-graph-simplify-history.sh t6012-rev-list-simplify.sh t6111-rev-list-treesame.sh t6019-rev-list-ancestry-path.sh`
  passes 12/12, 42/42, 65/65, and 18/18.
