# t1013 read-tree submodule

## 2026-06-05

- Claimed ticket `a0366b`.
- Starting from a clean GitButler workspace on `grit-t1-progress`.
- Target test: `tests/t1013-read-tree-submodule.sh`, which delegates its cases to
  `tests/lib-submodule-update.sh`.
- Reproduced the initial failure at subtest 6: recursive `read-tree -u -m
  --recurse-submodules` refused to remove a clean submodule checkout as
  "untracked".
- Fixed recursive gitlink removal to distinguish pure submodule deletion from
  replacement, keeping replacement untracked-file guards intact.
- Added fallback tracked-path detection from a local `.gitmodules` URL when a
  copied submodule worktree has a stale gitfile and no local module index.
- Allowed recursive submodule add to overwrite an ignored untracked path, while
  still rejecting non-ignored paths.
- Refreshed index stat data after `read-tree -u --reset` checkout so immediate
  `diff-files` checks see a clean worktree.
- Moved non-recursive gitlink-to-file rejection into validation so failed
  switches do not partially update `.gitmodules` or other worktree files.
- Canonical harness now passes:
  `./scripts/run-tests.sh t1013-read-tree-submodule.sh --verbose --timeout 180`
  -> `68/68`.
