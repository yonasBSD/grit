# t1092 sparse-checkout compatibility

## 2026-06-05

- Claimed ticket `0e0b0d`.
- Starting from a clean GitButler workspace on `grit-t1-progress`.
- Refreshing `t1092-sparse-checkout-compatibility.sh` before inspecting failures.
- Baseline remained `63/106`.
- Moved sparse checkout warning detection before worktree updates in `checkout`; harness improved to
  `67/106`, clearing false warnings for files checkout created inside the sparse cone.
- Taught checkout to recognize `--patch` after the tree-ish, accepted hyphen-leading commit
  messages (`commit -m "-a"`), skipped absent skip-worktree entries during `commit -a`, allowed
  `add --refresh` to refresh sparse entries, and honored `add --sparse .` for out-of-cone paths.
- Latest harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `70/106`.
  Ticket remains open; next direct failure is still within `status/add: outside sparse cone`.
- After commit `c8a1bc3`, direct verbose execution showed subtest 15 now passes; first direct
  failure moved to subtest 18 (`diff with renames and conflicts`).
- Found that `checkout <current-branch>` rebuilt the index whenever staged changes made it differ
  from HEAD. That rejected staged D/F changes in the full checkout while sparse checkouts skipped
  the path. Adjusted the already-on-branch path to preserve staged work unless forced, sparse
  reapply is needed, or the index is empty.
- A follow-up direct run showed sparse checkout still failed the same loop because current-branch
  checkout re-applied sparse rules even with staged D/F changes. Narrowed current-branch checkout
  further: only force or an empty index rebuilds; ordinary `checkout <current-branch>` preserves
  staged work.
- Direct execution then passed subtest 18 and failed subtest 19. The remaining mismatch was a
  tracked D/F descendant (`folder2/0/1/1`) still marked skip-worktree in sparse repos after
  restoring `folder2/0/1` as a file from another tree. Added a path-checkout helper that clears
  skip-worktree on tracked descendants that can no longer exist on disk.
- Direct execution then passed subtest 19 and failed subtest 22. `blame` was allowing a missing
  working-copy path to proceed when the index knew the path; Git's no-revision working-copy blame
  lstat check fails immediately for missing sparse paths. Tightened that guard.
- Direct execution then passed subtest 22 and failed subtest 26. `reset base -- nonexistent-file`
  should be a no-op for an explicit non-HEAD tree-ish, while `reset HEAD -- nonexistent` remains an
  error. Narrowed the unmatched pathspec behavior accordingly.
- Direct execution now passes through subtest 35 and stops at read-tree subtest 36. Canonical
  harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `74/106`.
- The read-tree failure was `read-tree -m -u base HEAD update-folder2` rejecting sparse checkouts
  because `require_uptodate` treated missing skip-worktree entries as local changes. Missing
  skip-worktree paths are intentionally up to date in sparse checkouts, so `read-tree` now accepts
  them while still checking present files.
- Direct execution now passes through subtest 41 and stops at subtest 42
  (`merge, cherry-pick, and rebase`). Canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `78/106`.
- The subtest 42 merge failure was caused by sparse-index placeholders being collapsed before the
  merge commit tree was written; expanding the index for tree writing preserves out-of-cone files.
- The next subtest 42 failure was sparse-index cherry-pick. Cherry-pick applied sparse rules to all
  out-of-cone paths and then tried to write sparse-directory placeholders during checkout. Changed
  cherry-pick to clear skip-worktree for paths changed by the replay, expand placeholders before
  commit-tree writing, and skip sparse stage-0 entries/placeholders during worktree checkout.
- Focused sparse-index cherry-pick of `update-folder1` now succeeds and materializes `folder1/a`
  while keeping unchanged out-of-cone entries sparse.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `80/106`.
  Direct execution now passes subtest 42 and exposes later conflict-resolution failures.
