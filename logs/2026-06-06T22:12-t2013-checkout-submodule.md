# t2013-checkout-submodule

Ticket: d7df5d67-3cc2-4d6c-962a-f39fab2414bc (created fresh; file had no open ticket — regressed after prior ticket closed)

## First run

`./scripts/run-tests.sh t2013-checkout-submodule.sh` => 68/74.

TAP breakdown: 70 real tests. 3 known breakages remain (tests 15, 22, 35 — upstream
`test_expect_failure`), 1 known breakage vanished (test 23). One real failure:

- test 38: `git checkout -f --recurse-submodules: updating to a missing submodule commit fails`

The non-forced sibling (test 18) already passed.

## Diagnosis

Reproduced test 38 by hand. Sequence: clone fixture, checkout `add_sub1`, init `sub1`,
then `git checkout -f --recurse-submodules invalid_sub1` (whose recorded sub1 commit
`1234...7890` does not exist). The command correctly failed with
"failed to checkout submodule at 'sub1' ...", and the index stayed at `add_sub1`
(diff-index --cached empty). But `git diff-files --ignore-submodules` reported
`.gitignore .gitmodules file1 file2` as modified.

Cause: with `force`, `checkout_index_to_worktree_inner` (grit/src/commands/checkout.rs)
rewrites every regular file in the work-tree write loop (`force_write_all`), bumping
their mtimes, and only afterwards calls `checkout_gitlink_worktree_entry` for the gitlink
— which bails on the missing commit. Since the function returns Err, the subsequent index
write (which would refresh the cached stat) in `switch_to_tree` never runs. Result: stale
stat vs unchanged index => spurious `M` in diff-files. File contents were actually
identical across branches; only the stat drifted.

Git avoids this: unpack-trees runs a dry-run `check_submodule_move_head` for each gitlink
before applying anything, so a missing submodule commit aborts the whole checkout
atomically with no work-tree mutation.

## Fix

Added `check_submodule_targets_available(repo, old_index, new_index, work_tree)` in
grit/src/commands/checkout.rs. Called at the very top of
`checkout_index_to_worktree_inner` (before any removal/write), guarded by
`populate_gitlinks && RECURSE_SUBMODULES.with(|r| r.get())`. For each changed stage-0
gitlink whose target OID differs from the old index and whose submodule is initialized
(`.git/modules/<path>/HEAD` exists), it opens the submodule object store via
`Odb::new(modules_git.join("objects"))` and bails if `odb.exists(&oid)` is false. The
error message names the path (`'sub1'`) so the test's `test_grep sub1 err` passes.

Uninitialized gitlinks (no local HEAD) are skipped — they stay empty placeholders and are
never moved. Unchanged gitlinks are skipped — no move happens.

## Result

`./scripts/run-tests.sh t2013-checkout-submodule.sh` => 70/74, fully_passing = true
(failing = 0; the 4 non-ok lines are the 3 expected known breakages + 1 vanished, all
counted correctly by the harness).

Regression checks (isolated --data-dir, all still fully passing): t2018-checkout-branch,
t2020-checkout-detach, t1013-read-tree-submodule, t6438-submodule-directory-file-conflicts,
t7400-submodule-basic, t7406-submodule-update, t2000-conflict-when-checking-files-out,
t2007-checkout-symlink.

Unit tests: only the 2 known pre-existing `ignore::gitignore_glob_tests` failures (unrelated).
No new clippy warnings in checkout.rs.
