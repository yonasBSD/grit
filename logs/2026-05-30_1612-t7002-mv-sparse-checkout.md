# t7002-mv-sparse-checkout — 4/22 → 22/22

Branch: `wf/p5/t7002-mv-sparse-checkout`
Base: `b1215d00a`

## Result
- Target test t7002-mv-sparse-checkout: **22/22** (was 4/22).
- grit-lib unit tests: 225 passed, 0 failed.
- clippy: no new warnings on changed lines.

## Changes

### 1. `grit/src/commands/sparse_checkout.rs` — stop deleting untracked files
`apply_sparse_patterns` ran a `remove_untracked_outside_sparse` pass that deleted
UNTRACKED working-tree files falling outside the sparse patterns. Real git's
`sparse-checkout set/reapply` never removes untracked files (cone or non-cone).
Removed the call and the now-dead helper (and the unused `HashSet` import).
Tracked-file removal + empty-dir cleanup is still handled by the main entry loop.
This unblocked ~12 subtests that `cat` helper files at the worktree root after
applying sparsity.

### 2. `grit/src/commands/mv.rs` — git-exact error messages
All collision/conflict `bail!` strings now use git's exact form
`fatal: <reason>, source=<src>, destination=<dst>` (no single quotes). `main.rs`
strips/re-emits a leading `fatal:` with exit 128, so this yields byte-identical
stderr and the correct exit code. (subtests 14,17,18,19 and the tails of 20,22)

### 3. `grit/src/commands/mv.rs` — sparse-advice list order
The "paths outside sparse-checkout" advice was sorted+deduped. Git's
`only_match_skip_worktree` is `STRING_LIST_INIT_DUP` printed in insertion order
(src then dst per blocked move). Removed the sort/dedup. (subtests 6,7)

### 4. `grit-lib/src/sparse_checkout.rs` — cone parent direct files
In expanded cone mode (`/sub/` + `!/sub/*/`), a FILE directly inside a cone
parent dir (e.g. `sub/d`) is in-cone; only sub-DIRECTORIES are excluded.
`path_in_expanded_cone` wrongly excluded such files. (subtest 7)

### 5. `grit/src/commands/mv.rs` — non-cone tri-state matcher
mv used the cone-only prefix matcher for sparse decisions. Non-cone sparse-checkout
needs git's last-match-wins parent-directory walk (UNDECIDED → check parent), so
an unanchored `y/` matches a `y` dir at any depth and `!x/y/z` excludes
`x/y/z/new-a`. Added a `path_in_sparse` helper routing non-cone through
`grit_lib::ignore::path_in_sparse_checkout`. (subtests 8,9)

### 6. `grit/src/commands/mv.rs` — "destination exists in the index" for dir moves
When moving a directory into an out-of-cone (SKIP_WORKTREE_DIR/SPARSE)
destination with `--sparse` and without `--force`, refuse if any expanded
destination file already exists in the index (git's per-entry guard). Reports
the first colliding pair in index order. (subtest 19)

### 7. `grit/src/commands/mv.rs` — remove emptied source dir
Mirror git's `remove_empty_src_dirs`: after a directory move relocates every
tracked entry, recursively remove the orphaned source directory (clearing
leftover untracked empty subdirs like `sub/dir/deep`). (subtests 21,22)

## Regression checks (baseline binary @ b1215d00a vs this branch)
- t6435-merge-sparse: 6/6 (guard, unchanged green)
- t3602-rm-sparse-checkout: 7 → 13 (improved)
- t6428-merge-conflicts-sparse: 1 → 2 (improved)
- t3705-add-sparse-checkout: 15 → 17 (improved)
- t1091-sparse-checkout-builtin: 55 → 56 (improved)
- t1092-sparse-checkout-compatibility: 47 → 48 (improved)
- t7012-skip-worktree-writing: 11 → 11 (unchanged)
- t1090-sparse-checkout-scope: 6 → 6 (unchanged)
- t1011-read-tree-sparse-checkout: 21 → 21 (unchanged)
- t7817-grep-sparse-checkout: 8 → 8 (unchanged)
- t9590-mv-cross-directory: 3 → 3 (unchanged)

No regressions; several siblings improved.
