# t6041-bisect-submodule — ticket 177015

## Result
14/14 passing (was 10/14).

## Failing subtests fixed
- 7: git_bisect: replace submodule with a directory must fail
- 8: git_bisect: replace submodule containing a .git directory with a directory must fail
- 9: git_bisect: replace submodule with a file must fail
- 10: git_bisect: replace submodule containing a .git directory with a file must fail

## Root cause
The test's `git_bisect` helper runs `test_must_fail git checkout <branch>` where the
target branch turns a populated `sub1` submodule into a tracked directory (or file).
Upstream git refuses this checkout (the submodule's work-tree files are untracked from
the superproject's view and would be overwritten). grit's `git checkout` of a branch
SUCCEEDED instead of failing, only emitting `warning: unable to rmdir 'sub1'` and leaving
the submodule dir behind.

grit already had `refuse_populated_submodule_tree_replacement` (used by the rebase path),
but its guard in `checkout_index_to_worktree_inner` required `!populate_gitlinks`, so it
never fired for a plain branch checkout (which passes `populate_gitlinks=true`).

## Fix (grit/src/commands/checkout.rs)
- Added `refuse_populated_submodule_tree_replacement_inner(..., require_populated_on_disk)`.
  When `require_populated_on_disk` is true, only refuse for old gitlinks whose work tree is
  actually populated on disk (has `.git` or non-`.git` content), matching git's
  verify_clean_submodule (empty/absent submodule dirs are still allowed to transition).
- Changed the guard in `checkout_index_to_worktree_inner`: when
  `refuse_submodule_replacement && preserve_dropped_gitlink_dirs`, run the populated-on-disk
  variant for the branch-checkout path (`populate_gitlinks=true`, t6041) and keep the
  unconditional variant for the rebase-style path (`populate_gitlinks=false`, t3426/t6042).

## Regression check (old binary vs new, same machinery via lib-submodule-update.sh)
- t6041: 10/14 -> 14/14.
- t2013-checkout-submodule: 62 -> 69 passing (fixed the non-recurse `git_test_func: replace
  submodule with a directory/file must fail` cases 53-56/67-70). Remaining failure (subtest
  38, `--recurse-submodules` missing-commit) is PRE-EXISTING and untouched by this change
  (recurse sets preserve_dropped_gitlink_dirs=false, so my guard does not run).
- t3426-rebase-submodule: 11/29 unchanged.
- t3513-revert / t3512-cherry-pick / t1013-read-tree / t6438-df-conflicts / t3906-stash: no regressions.
- t7112-reset-submodule: 78 -> still 78 genuine passing (total rose 78->82 because more
  subtests now run; the 4 new "not ok" are all `# TODO known breakage` test_expect_failure).
- t6437-submodule-merge subtests 16 & 22 fail with AND without my change — PRE-EXISTING.

cargo test -p grit-lib --lib: only the 2 known ignore::gitignore_glob_tests failures.
