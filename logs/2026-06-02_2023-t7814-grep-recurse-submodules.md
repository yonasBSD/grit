# t7814-grep-recurse-submodules

## 2026-06-02 20:23

- Claimed `t7814-grep-recurse-submodules.sh` after `t7401-submodule-summary.sh` reached 25/25.
- Starting baseline from `data/test-files.csv`: 17/27 passing, 10 failing.

## 2026-06-02 20:44

- Fixed `git grep -ePATTERN` parsing before Clap sees grep arguments.
- Fixed cwd-relative output for recursive grep from subdirectories, including paths outside the
  current directory rendered with `../`.
- Preserved pathspec filtering at parent gitlinks while letting direct gitlink matches search the
  selected submodule contents; descendant wildcard pathspecs remain constrained.
- Opened historical tree gitlinks by trying both current worktree `.git` files and
  `.git/modules/<tree-path>`, which restores grep over moved submodule history.
- Routed cached/tree object reads through `Repository::read_replaced`, so replace refs are scoped
  independently for the superproject and each submodule.
- Emitted the existing trace2 promisor fetch counter for partial-clone submodule object reads.

## Verification

- `cargo fmt`
- `cargo build --release -p grit-cli`
- Direct: `cd tests && sh t7814-grep-recurse-submodules.sh -v > ../t7814.current.out 2>&1`
  reached all 27 non-TODO cases passing, with only 2 upstream TODO known breakages remaining.
- Harness: `./scripts/run-tests.sh t7814-grep-recurse-submodules.sh --verbose` passed with
  `t7814-grep-recurse-submodules (27/34)`, `failing=0`, and `todo=7`.
