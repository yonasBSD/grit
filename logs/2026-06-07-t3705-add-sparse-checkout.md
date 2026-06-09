# t3705-add-sparse-checkout — `git add --refresh` sparse guard

Ticket: 9f924b (subsystem group sparse-index, thread C)

## Problem

`git add --refresh <sparse_entry>` did not refuse to operate on a skip-worktree
index entry. It refreshed the entry (updating its cached mtime) and exited 0,
where Git must emit the sparse error+hint, exit non-zero, and leave the cached
stat data untouched.

Failing subtest:
- not ok 10 - git add --refresh does not update sparse entries

## Root cause

`run_refresh` in `grit/src/commands/add.rs` had no sparse handling at all
(`_sparse: &AddSparseState` was unused). It refreshed every matching stage-0
entry regardless of the skip-worktree bit or sparse-cone membership.

## Fix (mirrors `refresh()` in git/builtin/add.c, lines 122-158)

Git's `refresh()` passes `REFRESH_IGNORE_SKIP_WORKTREE`:
1. `refresh_index` itself only skips entries with the **skip-worktree bit set**.
   An out-of-cone path whose work-tree file exists is still refreshed and counts
   as "seen".
2. After refresh, any pathspec that matched no refreshed entry but does match a
   skip-worktree path OR is `!path_in_sparse_checkout` goes into
   `only_match_skip_worktree`, which triggers `advise_on_updating_sparse_paths`
   and `ret = 1`.

Implemented in `run_refresh`:
- Extracted `refresh_index_entry(ie, abs_path)` helper.
- No-pathspec path: skip entries where `ie.skip_worktree()` (unless `--sparse`).
- Pathspec path: per-pathspec track `seen` (an entry was actually refreshed) vs
  `matched_sparse` (skip-worktree, or out-of-cone via `add_update_blocked`).
  If nothing was refreshed but a sparse path matched -> collect for advice.
  If nothing matched at all -> die "pathspec did not match any files" (existing).
- Emit `emit_sparse_path_advice` and `exit(1)` for the sparse case, after
  writing the index for any non-sparse entries that were refreshed.
- `--sparse` (`args.sparse` / `include_sparse`) disables both guards.

## Regression caught and fixed mid-change

First attempt used `add_update_blocked` as the *skip* condition in the refresh
loop, which also skipped out-of-cone-but-materialized entries. That broke
t1092 subtest 15 "status/add: outside sparse cone" (line 514:
`git add --refresh folder1/a` must succeed silently for a present out-of-cone
file). Corrected to skip only `skip_worktree()` entries during refresh, matching
`REFRESH_IGNORE_SKIP_WORKTREE`; the out-of-cone classification is only a fallback
for pathspecs that refreshed nothing.

## Results

- t3705-add-sparse-checkout: 20/20 (was 19/20).
- t1092-sparse-checkout-compatibility: 104/106 (unchanged — remaining failures
  are subtests 11 and 18, tracked separately in ticket 03ecca, not mine).
- `cargo test -p grit-lib --lib`: only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures.
- No new clippy/build warnings in add.rs.
