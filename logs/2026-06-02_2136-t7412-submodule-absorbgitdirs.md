# t7412-submodule-absorbgitdirs

## 2026-06-02 21:36

- Claimed `t7412-submodule-absorbgitdirs.sh` after
  `t7409-submodule-detached-work-tree.sh` reached 3/3.
- Starting baseline from `data/test-files.csv`: 10/12 passing, 2 failing.
- Direct baseline showed failures in:
  - `git fsck` after absorbing `sub1`, where the superproject index gitlink
    OID was incorrectly treated as a required local object.
  - recursive `submodule update --init --recursive`, where a clean parent
    submodule with a temporary in-tree `.git` directory was checked out again
    and printed output.
- Updated `fsck` to skip gitlink entries from index/resolve-undo seeds and
  updated submodule update's already-current fast path to allow recursive
  callers while still checking worktree cleanliness.
- `cargo build --release -p grit-cli` completed with the existing warning backlog.
- Direct `sh t7412-submodule-absorbgitdirs.sh -v` passed 12/12.
- Harness `./scripts/run-tests.sh t7412-submodule-absorbgitdirs.sh` passed 12/12 and refreshed
  `data/test-files.csv` plus dashboards.
- Validation completed: `cargo fmt`, `cargo check -p grit-cli`, `cargo test -p grit-lib --lib`
  (238/238), and `cargo clippy --fix --allow-dirty`; clippy's unrelated edits in
  `grit-lib/src/config.rs` and `grit-lib/src/filter_process.rs` were reverted.
