# t2021-checkout-overwrite: 4/9 -> 9/9

Date: 2026-05-31
Branch: wf/p6/t2021-checkout-overwrite

## Failing subtests (before)
3, 5, 6, 7 (one cluster) and 9 (separate). Two root causes.

## Cluster A: leading-path conflict (tests 3, 5; cascades 6, 7)

`check_dirty_worktree` (grit/src/commands/checkout.rs) only recorded an
untracked conflict when the FULL new-entry path existed on disk. When the
target tree turns an untracked file/symlink back into a directory (e.g.
target wants `a/b/c/d` while `a/b` is an untracked file or symlink on disk),
the full path `a/b/c/d` is absent, so no conflict was recorded and the
checkout destructively clobbered `a/b`.

Fix: new helper `untracked_leading_path_in_the_way` walks the ancestor
directory components of each absent new entry and, using
`std::fs::symlink_metadata` (so dangling/relative symlinks are detected),
flags the first ancestor that exists as a non-directory (regular file OR
symlink) and is genuinely untracked (not in `old_paths`). This mirrors
upstream git unpack-trees.c `check_leading_path` -> `check_ok_to_remove`
(a non-directory leading path => `ERROR_NOT_UPTODATE_FILE`). Tracked
dir->file/symlink transitions are untouched because we skip ancestors that
are present in the old index. `untracked_conflicts` is now sorted+deduped
so a single blocking ancestor is reported once.

## Cluster B: --overwrite-ignore was unwired (test 9)

The `overwrite_ignore` arg was declared but never read. Wired it through a
new `OVERWRITE_IGNORE` thread-local (same pattern as `RECURSE_SUBMODULES`),
set from `args.overwrite_ignore` in `run()`. This avoids touching the ~11
`switch_to_tree` call sites and the two rebase.rs callers (which keep
current behavior).

When set, `check_dirty_worktree` builds a
`grit_lib::ignore::IgnoreMatcher::from_repository(repo)` and, before pushing
an untracked conflict, skips it when:
- the in-the-way path is itself ignored, or
- it is an untracked directory whose every untracked child is ignored
  (helper `dir_only_holds_ignored`, mirroring `verify_clean_subdirectory`'s
  `read_directory` with standard excludes). A tracked or non-ignored
  untracked child keeps the conflict.

The ignore exception is applied to the leading-path push, the
`dir_untracked_conflicts` push, and the final `untracked_conflicts` push.

## Result
t2021: Tests 9 Pass 9 Fail 0.

## Regression checks (all unchanged vs baseline)
- t7110-reset-merge: 21/21
- t2000-conflict-when-checking-files-out: 14/14 (catalog showed 13; improved/stale)
- t2003-checkout-cache-mkdir: 10/10
- t2007-checkout-symlink: 4/4
- t5403-post-checkout-hook: 14/14
- t3426-rebase-submodule: 3/29 (unchanged)
- t3501-revert-cherry-pick: 20/21 (unchanged)
- t1011-read-tree-sparse-checkout: 23/23
- t7607-merge-state: 0/1 (unchanged)
- t5547-push-quarantine: 6/6
- t5813-proto-disable-ssh: 81/81

cargo fmt clean; cargo test -p grit-lib --lib: 228 passed; clippy: no new
warnings on checkout.rs.

## Files changed
- grit/src/commands/checkout.rs
