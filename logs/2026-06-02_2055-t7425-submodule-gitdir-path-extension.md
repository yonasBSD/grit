# t7425-submodule-gitdir-path-extension

## 2026-06-02 20:55

- Claimed `t7425-submodule-gitdir-path-extension.sh` after `t7408-submodule-reference.sh`
  reached 16/16.
- Starting baseline from `data/test-files.csv`: 18/23 passing, 5 failing.
- Direct run showed failures in clone cases that set
  `extensions.submodulePathConfig=true` through `git clone -c`; the config entry was written but
  `core.repositoryformatversion` stayed at 0, making subsequent commands reject the repo.
- Updated clone config application to bump `core.repositoryformatversion` to 1 when the v1-only
  submodule path extension is enabled through `-c`.
- Follow-up direct run reached 22/23. The remaining failure was a push `updateInstead` case where
  the remote ref moved, but the remote worktree/index stayed at the old tree and appeared dirty.
- Updated push `updateInstead` to verify cleanliness against the old tip, then refresh the remote
  branch worktree/index with a hard reset instead of using checkout in a way that detached `HEAD`.
- Direct `cd tests && sh t7425-submodule-gitdir-path-extension.sh -v` passed all 23 tests.
- Harness `./scripts/run-tests.sh t7425-submodule-gitdir-path-extension.sh` passed 23/23 and
  refreshed `data/test-files.csv` plus the dashboards.
