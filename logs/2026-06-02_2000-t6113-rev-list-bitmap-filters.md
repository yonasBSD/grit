## t6113-rev-list-bitmap-filters

Goal: make `tests/t6113-rev-list-bitmap-filters.sh` fully pass as the next rev-list/revision
traversal t6 item.

Initial CSV state: 14 tests, 13 passed, 1 failing.

Worktree: `/private/tmp/grit-t6-family` on branch `wf/t6-family`.

Notes:
- Starting after the tracking/refs/ref formatting group checkpoint was committed and rebased onto
  `origin/main`.
- Failure was test 14, `bitmap traversal with --unpacked`: Grit emitted only the loose commit,
  root tree, and new blob, omitting packed blobs reachable from the new unpacked commit's tree.
- Root cause: `--unpacked` filtered the commit walk correctly, but the object walk also received
  the packed-object set and suppressed packed trees/blobs. Git uses `--unpacked` here to select
  unpacked commits, then emits the full object closure for those commits.
- Fix: keep filtering packed commits for `--unpacked`, but do not pass the packed-object set into
  tree/blob traversal.
- Verification: `cargo check -p grit-cli` and `cargo build --release -p grit-cli` passed with the
  existing warning backlog. `./scripts/run-tests.sh t6113-rev-list-bitmap-filters.sh --verbose`
  passed 14/14. Companion `./scripts/run-tests.sh t6000-rev-list-misc.sh --verbose` improved from
  8/23 to 9/23.
