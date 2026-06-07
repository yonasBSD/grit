# t4041-diff-submodule-option — ticket 44e9ab

Date: 2026-06-06T20:46Z

## Ticket
Single failing subtest at last scan:
- 46: `diff --submodule with .git file`

## Root cause
`git diff --submodule HEAD^` for this subtest compares the work tree against `HEAD^`
(which has `sm1` only) while the work tree has `sm2` (gitlink, `.git` file) and `sm1`
is gone. Expected:

```
Submodule sm1 <head6>...0000000 (submodule deleted)
Submodule sm2 0000000...<head7> (new submodule)
```

`git diff <tree>` (one rev vs work tree) routes through the shared
`grit_lib::diff::diff_tree_to_worktree`. Its gitlink branch only handled paths that
were present in the **tree** (`if let Some(te) = tree_entry`), and even then a deleted
submodule (work-tree dir gone) was emitted as a `Modified` gitlink entry with new
mode `160000`/new oid `0` rather than a proper deletion. Two gaps:

1. **New submodule** (gitlink in index, absent from the tree, e.g. `sm2`): the
   `tree_entry == None` case fell through and produced no entry at all.
2. **Deleted submodule** (gitlink in tree, work-tree dir missing, e.g. `sm1`):
   produced a `Modified`/`160000` entry instead of a `Deleted`/`000000` entry.

The plumbing `diff-index` command uses its own `diff_tree_vs_worktree` (in
`grit/src/commands/diff_index.rs`) which already handled both cases correctly — that
is why `git diff-index -p --submodule=log HEAD^` was already correct while
`git diff --submodule HEAD^` produced nothing.

(Note: an early dead-end during debugging was a red herring caused by my personal
`~/.gitconfig` having `diff.ignoreSubmodules = all`; the test harness isolates HOME so
that setting does not apply during the actual run.)

## Fix
`grit-lib/src/diff.rs`, `diff_tree_to_worktree` gitlink branch — rewrote it to a
`match (tree_entry, index_gitlink_oid)`:
- `(Some(te), _)`: if the submodule work-tree dir is gone → emit `Deleted`
  (new mode `000000`, new oid `0`). Otherwise keep the prior modified/dirty logic.
- `(None, Some(idx_oid))`: new submodule → emit `Added` (old `000000`/`0`, new
  `160000` with the submodule HEAD oid, falling back to the index gitlink oid).
- `(None, None)`: nothing.

This mirrors Git's `diff-lib.c` and the existing `diff-index` behaviour, so the
`--submodule` renderer (`write_submodule_diff_recursive`) emits
`(submodule deleted)` / `(new submodule)`.

## Result
- `t4041-diff-submodule-option`: 47/47 passing (was 46/47).
- Side effect: `t4060-diff-submodule-option-diff-format` improved 48 → 49 (shared fix).
- No regressions in: t4027 (20/20), t7506 (40/40), t4011-diff-tree/symlink, t4029.
- `cargo test -p grit-lib --lib`: only the 2 known pre-existing `ignore::gitignore_glob_tests` failures.
- No new clippy warnings in edited lines.

## Files changed
- `grit-lib/src/diff.rs`
- `data/tests/t4/t4041-diff-submodule-option.toml`
