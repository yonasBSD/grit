# t3426-rebase-submodule — stale index stat after rebase

Ticket: e0d326. Subsystem group: rebase-submodule (lib-submodule-update.sh driven).

## Starting state
Fresh run: 11/29 passing. 18 failing subtests (git_rebase 1-6, 11-13; git_rebase_interactive
15-20, 25-27). All were the "added submodule creates empty directory" / "modified submodule does
not update work tree" family driven through `lib-submodule-update.sh`.

## Root cause
All 18 failures died at `test_superproject_content`, specifically the second assertion:

```sh
git diff-files --ignore-submodules >actual && test_must_be_empty actual
```

`git diff-files` reported `.gitignore`, `.gitmodules`, `file1`, `file2` as Modified (destination
OID all-zeros) even though `git status` showed a clean working tree. Running `git status` (which
refreshes the index) made the next `git diff-files` clean — classic stale-stat-cache signature.

The culprit was `reset_index_to_head` in `grit/src/commands/rebase.rs` (called at the end of
`finish_rebase` to "leave the index matching the new tip"). It rebuilt the index from HEAD's tree
via `tree_to_index_entries`, which produces entries with **zeroed** ctime/mtime/dev/ino/size, then
wrote the index WITHOUT refreshing stat info from the work tree. This final write clobbered the
good stat data that the pick/fast-forward paths had carefully populated, so every entry had no
mtime/size and `git diff-files` flagged them all as modified.

## Fix
Call the existing `refresh_index_stat_cache_from_worktree(repo, &mut index)` before
`repo.write_index` in `reset_index_to_head` (one line + comment). This re-stats each entry against
the work tree (content-verified) so the stat cache matches, mirroring Git refreshing the index
after rebase.

## Result
29/29 passing. `cargo test -p grit-lib --lib` passes modulo the 2 known pre-existing
`ignore::gitignore_glob_tests` failures (unrelated to this ticket). No new clippy warnings.
