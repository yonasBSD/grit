# t4060-diff-submodule-option-diff-format — ticket 1df24a

Date: 2026-06-06T21:27Z
Agent: schacon+claude-t5

## Result

51/51 passing (was 49/51 at fresh run; 48/51 at ticket creation — subtest 44
"diff --submodule=diff with .git file" was already fixed by an earlier ticket in
the submodule group).

Fixed subtests:
- 50: diff --submodule=diff recurses into nested submodules
- 51: diff --submodule=diff recurses into deleted nested submodules

## Root causes (three distinct bugs)

### Bug 1 — deleted submodule with absent worktree not reported (subtest 50)

`grit-lib/src/diff.rs::diff_index_to_worktree_with_options` gitlink branch:
`submodule_worktree_is_unpopulated_placeholder` returns `true` for a directory
that does not exist (`NotFound -> true`). That conflated two cases Git keeps
separate:
- empty directory present  -> placeholder before `git submodule update` (clean)
- directory entirely absent -> deleted submodule (a change)

Git's `diff-lib.c::check_removed` `lstat`s the gitlink path first: a missing dir
is a removal (`D`, new mode 000000) emitted *before* the "not checked out"
special case (which only applies when the dir exists). grit silently treated
`sm1` (worktree removed earlier in setup) as unchanged, dropping its
`Submodule sm1 ...(submodule deleted)` line.

Fix: in the gitlink branch, if `sub_head_oid.is_none() && !sub_dir.exists()`
(and not `simplify_gitlinks`), emit a Deleted entry.

### Bug 2 — nested submodule counted as untracked content of parent (subtest 50)

`git diff --submodule=diff` patch rendering computes per-submodule dirty flags
via `grit/src/commands/diff_index.rs::submodule_dirty_flags`, whose helper
`submodule_dir_has_untracked_files` recursed into ALL subdirectories of the
submodule — including a nested submodule's checkout — and counted those files as
untracked. Git's `is_submodule_modified` runs `git status --porcelain=2` inside
the submodule: a nested submodule is a tracked gitlink (a porcelain v2 `1`/`2`
line with sub-state `S..U`), not loose untracked files, so it does NOT add
`DIRTY_SUBMODULE_UNTRACKED` for files inside it. grit emitted a spurious
`Submodule sm2 contains untracked content`.

Fix: `submodule_dirty_flags` now delegates to grit-lib's
`submodule_porcelain_flags`, which already implements the nested-gitlink-aware
untracked walk plus staged+unstaged modified detection. Removed the now-dead
helpers `submodule_worktree_has_untracked`, `submodule_dir_has_untracked_files`,
`submodule_has_unstaged_changes`.

### Bug 3 — nested absorbed gitdir not located for deleted nested submodule (subtest 51)

When the parent submodule worktree is gone (`mv sm2 sm2-bak`), rendering its
deleted contents recurses into the nested gitlink `sm2/nested`. The nested
submodule's absorbed gitdir lives at `<super>/.git/modules/sm2/modules/nested`,
but `absorbed_submodule_gitdir` only tried `<super>/.git/modules/<name>` with the
full path `sm2/nested` resolved as a single name -> `.git/modules/sm2/nested`
(wrong), so nested's objects were unreachable and its deleted files were omitted.

Fix: `absorbed_submodule_gitdir` now walks the path hierarchy — resolve the
top-level segment's name from the superproject `.gitmodules`, then descend
`modules/<seg>` for each remaining path segment.

## Files changed
- grit-lib/src/diff.rs — deleted-submodule (absent worktree) detection in gitlink branch
- grit/src/commands/diff_index.rs — submodule_dirty_flags via porcelain flags; nested absorbed_submodule_gitdir; removed dead helpers; dropped unused MODE_TREE import

## Verification
- t4060: 51/51
- No regressions: t4041 47/47, t7506 40/40, t3040 11/11, t7400 124/124
- t2013 69/74 (was 62/74 at HEAD — improved by another agent's in-flight work, not me)
- grit-lib unit tests: 272 pass, only the 2 known pre-existing ignore::gitignore_glob_tests failures
- No clippy warnings in my files
