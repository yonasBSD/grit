# t2500-untracked-overwriting — fix log (2026-05-31)

Branch: `wf/p6/t2500-untracked-overwriting`
Worktree: `/Users/schacon/grit-worktrees/wf-p6-t2500-untracked-overwriting`

## Result
- Baseline: 3/8 success subtests passing (1, 6, 9). Subtests 7 & 8 are
  `test_expect_failure` known breakages (rebase --autostash / stash), out of scope.
- After: **8/8** success subtests passing. 7 & 8 remain expected breakages.
  Final harness line: `# Tests: 10  Pass: 8  Fail: 0  Skip: 0`.

## Failing subtests and fixes

### Subtests 2 & 3 — reset --merge/--keep preserve untracked files/dirs
File: `grit/src/commands/reset.rs`, `check_merge_reset_worktree` `(None, Some(_))` arm.
The premature uptodate check bailed with `Entry '<path>' would be overwritten by
merge` when an untracked worktree path obstructs a path present in the target tree
but absent from the index. Upstream `verify_absent` distinguishes a directory from a
file and emits `Updating '<path>' would lose untracked files in it` (dir) /
`Updating '<path>' would lose untracked files.` (file). Now matched via
`std::fs::symlink_metadata` / `is_dir()`. (The existing `find_untracked_obstruction`
call downstream already had the right message but was unreachable — the earlier check
fired first.)

### Subtest 4 — checkout -m does not nuke untracked file
File: `grit/src/commands/checkout.rs`.
`merge_branch_working_tree` materialized the destination tree via
`checkout_index_to_worktree(force_write_all=true)` with no untracked-overwrite check,
silently clobbering an untracked file the target branch also tracks. Factored the
untracked-conflict scan out of `check_dirty_worktree` into a new
`pub(crate) check_untracked_overwrite` (untracked-only; no tracked-file local-change
detection, so `-m` still merges local edits) and called it before the destructive
checkout.

### Subtest 5 — git rebase --abort and untracked files
File: `grit/src/commands/rebase.rs`, `do_abort`.
The abort laid down the orig-HEAD tree via `checkout_merged_index`, which performs no
`verify_absent` at any force level, overwriting an untracked file. Added an
up-front obstruction check using `reset::find_untracked_obstruction` (made
`pub(crate)`) against the orig-HEAD restore index, before HEAD/index/worktree are
mutated, so the abort is a no-op on obstruction.

### Subtest 10 — git am --skip and untracked dir vs deleted file
Two distinct root causes:
1. `grit/src/commands/format_patch.rs`, `collect_commits_for_format_patch`:
   `git format-patch -N <commit>` was unconditionally rewriting a single committish
   to `<commit>..HEAD`, so `format-patch -1 simple` formatted the wrong commit.
   Per git-format-patch docs, an explicit `-N` count makes the committish the positive
   endpoint itself. Guarded the rewrite with `max_count.is_none()`. Verified
   `grit format-patch -1 --stdout simple` now matches real git (Subject `another`),
   and `-1/-2 HEAD`, `-1 HEAD~1`, and the no-count `<since>..HEAD` range all match git.
2. `grit/src/commands/read_tree.rs`, `checkout_index_entries` removal loop:
   when a tracked path is removed but the worktree now holds a directory containing
   untracked files there, refuse with `Updating '<path>' would lose untracked files in
   it` (Git `verify_clean_subdirectory`) instead of `rm -rf`. This is the removal-side
   analog of the existing write-side check; reused `worktree_has_untracked_under_path`.

## Quality gates
- `cargo fmt`: clean.
- `cargo test -p grit-lib --lib`: 228 passed, 0 failed.
- `cargo clippy -p grit-lib -p grit-cli`: no new warnings on changed lines.
- Regression guards: t7110-reset-merge 21/21, t5813-proto-disable-ssh 81/81,
  t5563-simple-http-auth 17/17. t5547-push-quarantine 5/6 — the one failure
  (subtest 5, colon path separator) is pre-existing on the baseline binary, unrelated.
- Sibling regression diff (worktree vs baseline main binary), identical counts (no
  regression): t4014-format-patch 68/215, t7102-reset 36/38, t7201-co 28/46,
  t4150-am 36/87, t4151-am-abort 11/20, t3407-rebase-abort 15/17.

## Files changed
- grit/src/commands/reset.rs
- grit/src/commands/checkout.rs
- grit/src/commands/rebase.rs
- grit/src/commands/format_patch.rs
- grit/src/commands/read_tree.rs
