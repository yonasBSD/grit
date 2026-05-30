# t1091-sparse-checkout-builtin

Branch: wf/p5/t1091-sparse-checkout-builtin
Base: b1215d00a

## Result
- Before: 55/77 passing
- After: 76/77 passing (the remaining failure is test 33 `sparse-checkout reapply`,
  which is upstream `test_expect_failure` / known breakage — it still fails as
  expected and was NOT flipped because it does not pass).
- `cargo test -p grit-lib --lib`: 225 passed.
- Regression guard t6435-merge-sparse: 6/6.
- No regressions in sibling sparse files (t7012 11/11, t1090 6/7, t1092 50/106 [+2],
  t7817 8/8, t3602 13/13, t3705 18/20 [+3]). t2402-worktree-list test 26 fails but is
  a PRE-EXISTING failure at base (verified by building base worktree.rs).

## Changes

### grit-lib/src/sparse_checkout.rs
- `ConePatterns::try_parse_with_warnings`: mirror Git `dup_and_filter_pattern`
  (strip escape backslashes, truncate trailing `/*`) when computing recursive keys,
  so escaped duplicate patterns like `/foo/\*/` are detected as repeated; accept
  multi-level negative parent patterns like `!/foo/bar/*/`.
- `parse_expanded_cone_recursive_dirs` and `parse_expanded_cone_parent_recursive`
  now unescape cone pattern bodies (new `unescape_cone_pattern_path`) so on-disk
  escaped patterns match/list the literal directory name (`zbad\dir`).

### grit-lib/src/index.rs
- `directory_in_cone`: test directory-recursive cone inclusion (trailing slash) so a
  fully-excluded top-level directory collapses to a single sparse-index placeholder.

### grit/src/commands/sparse_checkout.rs (most fixes)
- cmd_list: parse the file in cone mode via try_parse_with_warnings, print
  structural warnings, list leaf recursive cone dirs, C-quote names (quote_c_style).
- cmd_disable: materialize a full checkout (apply_full_checkout: clear all
  skip-worktree, write all blobs, never re-collapse) and record false config keys
  like Git set_config(MODE_NO_PATTERNS).
- apply_sparse_patterns: expand sparse-index placeholders before applying; keep
  not-up-to-date / unmerged excluded files (compute_sparse_side_effect_paths shared
  with the warning); run clean_tracked_sparse_directories (ignore-aware) to remove
  out-of-cone sparse dirs / warn on untracked content.
- sanitize_set_paths: follow Git sanitize_paths ordering; gate dir-pattern and file
  checks on !skip_checks; warn on non-cone single-file add; reject tracked files in
  cone set/add.
- cmd_add: sanitize (file check) before the cone-format check.
- cmd_reapply: warn about side effects before applying.
- cmd_clean: rewrite to mirror Git sparse_checkout_clean (bail on unmerged; collapse
  out-of-cone dirs by skip-worktree state; honor --dry-run/--force/--verbose).
- cmd_check_rules: C-unquote quoted input lines, re-quote matched paths on output.
- remove_untracked_outside_sparse: stop deleting untracked files (Git leaves them);
  normalize_rel_path only rewrites `\` on Windows (literal backslash filenames).
- main.rs: emit Git "unknown option" wording for mistyped set/add flags.

### grit/src/commands/worktree.rs
- worktree add: apply inherited sparse patterns (only when a sparse-checkout file
  exists) so out-of-cone paths are excluded in the new worktree.
- worktree remove: skip-worktree / sparse-directory-placeholder entries are not
  treated as dirty, so removing a sparse worktree no longer errors.

## Tests fixed
16, 19, 20, 27, 31, 32, 39, 49, 51, 53, 54, 55, 56, 57, 59, 60, 61, 73, 75, 76, 77.
Test 33 remains a known breakage (test_expect_failure).
