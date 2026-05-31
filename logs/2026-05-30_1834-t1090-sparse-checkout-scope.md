# t1090-sparse-checkout-scope — 2026-05-30

## Result
- Target file t1090-sparse-checkout-scope.sh: 6/7 -> 7/7 (green).
- Only subtest 7 ("in partial clone, sparse checkout only fetches needed blobs") was failing.

## Root cause
`git -C client config core.sparsecheckout 1` sets the value to the git boolean
`1` (true). grit's sparse-checkout code paths matched only the literal string
`"true"` (`eq_ignore_ascii_case("true")`), so for the value `1` sparse mode was
silently treated as disabled during the partial-clone detached checkout of
`FETCH_HEAD`. This caused:
1. No skip-worktree bits set, so the worktree-write loop wrote `b` and `c/c`,
   each lazily fetching its blob (`read_object_for_checkout` ->
   `try_lazy_fetch_promisor_object`).
2. `sparse_checkout_patterns_for_hydration` returning `None`, so `switch_to_tree`
   took the full-tree `hydrate_tree_blobs_from_promisor` branch and fetched
   every blob into a promisor pack.

A residual single-blob over-fetch also came from defaulting
`core.sparseCheckoutCone` to true and feeding cone-mode matching for a non-cone
file (`!/*` + `/a`), which over-includes `b`.

## Changes
- grit-lib/src/sparse_checkout.rs
  - `apply_sparse_checkout_skip_worktree`: read `core.sparsecheckout` and
    `core.sparsecheckoutcone` via `get_bool(...).and_then(|r| r.ok())` instead
    of `eq_ignore_ascii_case("true")`.
  - `clear_skip_worktree_from_present_files`: same `get_bool` fix for
    `core.sparsecheckout`.
- grit/src/commands/checkout.rs
  - `sparse_checkout_config_enabled`: `get_bool` fix.
  - `sparse_checkout_patterns_for_hydration`: `get_bool` fix for enable + cone;
    only treat the file as cone when `ConePatterns::try_parse(&content).is_some()`
    (mirrors `effective_cone` in apply_sparse_checkout_skip_worktree), so the
    non-cone matcher is used for `!/*` + `/a` and only blob `a` is fetched.

## Verification
- t1090: 7/7.
- cargo test -p grit-lib --lib: 228 passed, 0 failed.
- cargo fmt: clean.
- clippy: pre-existing deny-level unwrap lints across the crate; none on the
  changed lines. Not blocking per instructions.
- Regression set (run with this worktree's release binary):
  - t5616-partial-clone: 44 -> 45 (improved, as predicted).
  - t5601-clone: 66/115 unchanged.
  - t1011-read-tree-sparse-checkout: 21/23 unchanged.
  - t1091-sparse-checkout-builtin: 55/77 unchanged.
  - t1092-sparse-checkout-compatibility: 47/106 unchanged.
  - t6435-merge-sparse-directory: 1/2 unchanged (verified against main worktree
    baseline, also 1/2).
- No eprintln/debug output added on stderr.
