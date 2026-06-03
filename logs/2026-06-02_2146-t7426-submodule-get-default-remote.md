# t7426-submodule-get-default-remote

## 2026-06-02 21:46

- Claimed `t7426-submodule-get-default-remote.sh` after
  `t7418-submodule-sparse-gitmodules.sh` reached 9/9.
- Starting baseline from `data/test-files.csv`: 14/15 passing, 1 failing.

## 2026-06-02 21:49

- Reproduced the remaining failure in test 4: from `super/subdir`,
  `submodule--helper get-default-remote ../subpath` failed with `could not get a repository
  handle`.
- Root cause: the helper used the superproject worktree root as the anchor for the user-supplied
  path, losing the caller's current directory for `../subpath`.
- Fixed the helper path by anchoring `get-default-remote` at `std::env::current_dir()` and then
  mapping the resolved path back to the superproject-relative submodule path.
- Verified `cd tests && sh t7426-submodule-get-default-remote.sh -v` passes 15/15.
- Refreshed harness data with `./scripts/run-tests.sh t7426-submodule-get-default-remote.sh`,
  passing 15/15 and updating `data/test-files.csv` plus dashboards.
