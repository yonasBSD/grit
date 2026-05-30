# t3602-rm-sparse-checkout: 7/13 -> 13/13

## Root cause
All 6 failing subtests (3602.2, .3, .4, .7, .11, .13) were not `rm` bugs. They
failed because applying sparse-checkout patterns deleted the tests' untracked
setup helper files that live outside the sparse cone (`b_error_and_hint`,
`sparse_entry_b_error`, `sparse_error_header`, `sparse_hint`). Each subtest's
`rm` output was already byte-correct once those helpers survived.

The culprit was `remove_untracked_outside_sparse` in
`grit/src/commands/sparse_checkout.rs`. Its per-file block deleted any untracked
file/symlink outside the sparse definition via `fs::remove_file`.

This does not match upstream Git. `clean_tracked_sparse_directories`
(`git/builtin/sparse-checkout.c`) only removes whole *tracked* sparse
directories that have gone out of cone scope, runs only in cone mode, and
explicitly preserves any directory that still contains untracked/ignored files
(warning "directory '%s' contains untracked files"). Upstream never deletes
individual untracked files.

## Change
Removed the per-file untracked-deletion block in
`remove_untracked_outside_sparse` (the `meta.is_file()/symlink` guard through
the `fs::remove_file` + `remove_empty_dirs_up_to` call). The directory-recursion
+ empty-dir-removal block is kept: a tracked out-of-cone directory becomes empty
because the entry loop already removed its tracked worktree files via
`set_skip_worktree` + `remove_file`, so the empty-dir check still removes it. A
directory that still holds untracked files stays non-empty and is preserved,
matching upstream.

## Results
- t3602-rm-sparse-checkout: 7/13 -> 13/13 (target, fully green)
- cargo test -p grit-lib --lib: 225 passed, 0 failed
- cargo fmt: clean; cargo clippy: no warnings on changed lines

## Regression / sibling checks (original release binary vs fixed binary)
- t6435-merge-sparse (regression guard): 6/6 -> 6/6 (no regression)
- t7012-skip-worktree-writing: 10/11 -> 11/11 (+1)
- t1091-sparse-checkout-builtin: 55 -> 56 ok (+1)
- t3705-add-sparse-checkout: 15 -> 17 ok (+2)
- t1092-sparse-checkout-compatibility: 47 -> 47 ok (no change)
- t7817-grep-sparse-checkout: 8/8 (no regression)
