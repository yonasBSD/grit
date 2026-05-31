# t6428-merge-conflicts-sparse — fix log (2026-05-30)

## Result
- Target: `tests/t6428-merge-conflicts-sparse.sh`
- Before: 1/2 passing. After: **2/2 passing**.

## Root cause
`git sparse-checkout set --no-cone README` was destroying UNTRACKED worktree files.
In `grit/src/commands/sparse_checkout.rs`, `apply_sparse_patterns()` always called
`remove_untracked_outside_sparse()`, which walked the whole worktree and did
`fs::remove_file` on any untracked file whose path was not "included" by the sparse
patterns. The t6428 setup writes plain untracked files `expected-index` and
`expected-merge` at the repo root; the sparse-checkout set then deleted them, so the
later `test_cmp expected-index index_files` / `test_cmp expected-merge numerals`
failed with "No such file or directory" and `git ls-files -o` returned the wrong count.

This violates Git. Upstream `clean_tracked_sparse_directories`
(`git/builtin/sparse-checkout.c:115`):
- returns early when patterns are NOT cone mode (removes nothing),
- in cone mode only removes whole TRACKED directories that have gone out of scope,
- keeps (with a warning) any such directory that still contains untracked content,
- never deletes an individual untracked file.

The merge output itself was already correct (numerals re-materialized with proper
2-way conflict markers; index matched `expected-index`). The only defect was the
untracked-file destruction.

## Change (grit/src/commands/sparse_checkout.rs only — CLI binary crate)
1. Gated the cleanup pass on `effective_cone`: in non-cone mode it now does nothing,
   matching Git's early return. This alone fixes t6428 (the test uses `--no-cone`).
2. Rewrote `remove_untracked_outside_sparse()` to mirror
   `clean_tracked_sparse_directories`:
   - Never touches individual files (tracked out-of-scope files are already removed
     via SKIP_WORKTREE earlier in `apply_sparse_patterns`; untracked files are kept).
   - Only considers TRACKED directories (those with an indexed path under them) as
     removal candidates; entirely-untracked directories are left alone.
   - Removes an out-of-scope tracked directory only when it is now empty (no untracked
     content survived); otherwise keeps it and emits
     `warning: directory '<dir>' contains untracked files, but is not in the
     sparse-checkout cone` to stderr.

## Verification (release binary, isolated env)
- t6428-merge-conflicts-sparse: 1/2 -> **2/2**.

### Regression guards (baseline rebuilt from this worktree's HEAD for an apples-to-apples diff)
- t6435-merge-sparse: 6/6 (guard, OK)
- t1091-sparse-checkout-builtin: 55 -> 56 pass (0 regressions; +1: "set from subdir in non-cone mode throws an error")
- t1092-sparse-checkout-compatibility: 47 -> 47 (0 regressions; the diagnosis "48" was from a different base SHA)
- t7012-skip-worktree-writing: 10 -> 11
- t7002-mv-sparse-checkout: 4 -> 13
- t3602-rm-sparse-checkout: 7 -> 13
- t1011-read-tree-sparse-checkout: 21 -> 21 (no change)

No previously-passing subtest regressed in any guarded file (verified via before/after
subtest diff for t1091 and t1092).

## Quality gates
- `cargo fmt`: clean.
- `cargo test -p grit-lib --lib`: 228 passed, 0 failed.
- `cargo clippy -p grit-cli`: no new findings on changed lines (the reported `unwrap()`
  errors are pre-existing in grit-lib, not in the changed CLI file).
