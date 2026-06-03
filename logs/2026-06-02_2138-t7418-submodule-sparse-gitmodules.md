# t7418-submodule-sparse-gitmodules

## 2026-06-02 21:38

- Claimed `t7418-submodule-sparse-gitmodules.sh` after
  `t7423-submodule-symlinks.sh` reached 6/6.
- Starting baseline from `data/test-files.csv`: 8/9 passing, 1 failing.

## 2026-06-02 21:44

- Reproduced the remaining failure in test 7: after `git -C super pull`, the superproject index
  recorded the new submodule gitlink, but `super/submodule` had not fetched that commit, so
  `submodule summary --for-status` emitted a missing-commit warning.
- Fixed `fetch` to begin/end the changed-submodule tip record and dispatch the typed recursive
  fetch implementation instead of the legacy all-submodules wrapper.
- Kept the implicit fetch recurse mode as Git's default/on-demand behavior while still letting
  submodule-specific fetch recurse config apply.
- Verified `cd tests && sh t7418-submodule-sparse-gitmodules.sh -v` passes 9/9.
- Refreshed harness data with `./scripts/run-tests.sh t7418-submodule-sparse-gitmodules.sh`,
  passing 9/9 and updating `data/test-files.csv` plus dashboards.
