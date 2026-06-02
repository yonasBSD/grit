# t7409-submodule-detached-work-tree

## 2026-06-02 21:24

- Claimed `t7409-submodule-detached-work-tree.sh` after `t7402-submodule-rebase.sh` reached 6/6.
- Starting baseline from `data/test-files.csv`: 1/3 passing, 2 failing.
- Direct run failed test 2 because `submodule add` used the nested-operation helper for its
  superproject `add --dry-run` probe. That stripped `GIT_DIR`/`GIT_WORK_TREE`, so a detached
  worktree with no `.git` directory could not be rediscovered.
- Added a superproject subprocess helper and used it for `submodule add` staging/probe commands,
  while leaving nested clone/checkout operations isolated from the superproject env.
- Direct run then reached 2/3. Test 3 failed during pull because local upload-pack inherited the
  client repo's `GIT_DIR`, causing the server side to run pack-objects against the client object
  store instead of the remote.
- Stripped `GIT_DIR` and `GIT_WORK_TREE` from local upload-pack server child processes in both
  local fetch and file protocol v2 helpers.
- Direct `cd tests && sh t7409-submodule-detached-work-tree.sh -v` passed all 3 tests.
- Harness `./scripts/run-tests.sh t7409-submodule-detached-work-tree.sh` passed 3/3 and refreshed
  `data/test-files.csv` plus the dashboards.
