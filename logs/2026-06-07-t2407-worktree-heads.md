# t2407-worktree-heads — ticket c03e13

## Result
12/12 passing (was 11/12). Subtest 6 "refuse to overwrite: worktree in rebase
with --update-refs" fixed.

## Root cause
A plain (non-interactive, non-rebase-merges, non-autosquash) merge-backend
`git rebase --update-refs <upstream>` never wrote the
`rebase-merge/update-refs` state file. grit only inserted `update-ref`
decorations into the todo on the autosquash / interactive paths
(`build_autosquash_with_update_refs`). The plain `else` arm that builds the todo
used `rebase_state_todo_lines`, which emits only `pick` lines.

Because `write_rebase_update_refs` derives its records from the `update-ref`
lines in the finalized todo body, an empty/absent set of such lines meant the
`update-refs` file was never created. Consequently the worktree-occupancy check
in `worktree_refs::occupied_branch_refs` (which already reads
`rebase-merge/update-refs`) found nothing, and `git branch -f can-be-updated HEAD`
was wrongly allowed instead of being refused with
`cannot force update the branch 'can-be-updated' used by worktree at .../wt-3`.

## Fix (grit/src/commands/rebase.rs)
- Added `rebase_update_refs_todo_lines`: builds the non-interactive todo via
  `build_autosquash_with_update_refs(repo, git_dir, commits, /*autosquash=*/false,
  /*update_refs=*/true)` and formats the resulting items (pick / update-ref /
  ref-comment) into todo lines.
- In the todo-building match, added an arm before the plain `else`: when
  `rebase_update_refs_enabled(&args, &config)` and the chosen backend is Merge,
  use the new helper so `update-ref` steps are interleaved after each pick.
  This makes `write_rebase_update_refs` populate `rebase-merge/update-refs`, and
  the replay loop's existing `RebaseReplayStep::UpdateRef` handling records the
  scheduled refs as worktree-reserved while the rebase is paused on a conflict.

Mirrors Git's `todo_list_add_update_ref_commands`, which runs for plain
`--update-refs` rebases too, not only interactive ones.

## Verification
- Manual repro: after `rebase --update-refs conflict-3` stops on conflict, the
  `update-refs` file is written and `git branch -f can-be-updated HEAD` is
  refused (exit 1, correct message).
- `./scripts/run-tests.sh t2407-worktree-heads.sh` => 12/12.
- `cargo test -p grit-lib --lib`: 276 passed; only the 2 known-pre-existing
  `ignore::gitignore_glob_tests` failures remain (unrelated to this ticket).

Note: a bare direct invocation of the .sh shows subtests 11/12 failing, but that
is an environment artifact of the ad-hoc run (lib-rebase.sh / $EDITOR /
$TEST_DIRECTORY setup); the canonical harness reports 12/12.
