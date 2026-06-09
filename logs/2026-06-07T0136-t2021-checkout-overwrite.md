# t2021-checkout-overwrite — cache-tree corruption on dir→file transition

Ticket: 40d548 (created fresh; no prior open/closed ticket existed)
Date: 2026-06-07

## Starting state
`tests/t2021-checkout-overwrite.sh`: 7/9 passing.

Failing subtests (verbose run):
- test 2 "create a commit where dir a/b changed to file" —
  `git add -A` aborted with `error: corrupted cache-tree has entries not present in index`.
- test 3 "checkout commit with dir must not remove untracked a/b" — cascaded from
  test 2 leaving the index still tracking `a/b` as a directory; `git rm --cached a/b`
  then failed with `not removing 'a/b' recursively without -r`.

## Root cause
The harness sets `GIT_TEST_CHECK_CACHE_TREE=true`, so grit runs
`write_tree::verify_cache_tree` on every index write.

When the tracked directory `a/b/` (containing `a/b/c/d`) was replaced by a file
`a/b`, `Index::invalidate_cache_tree_for_path` only marked the nodes along the
path invalid (`entry_count = -1`) — it never **removed** the leftover subtree
node `a/b` (and its descendant `a/b/c`). That stale descendant node kept a
positive `entry_count`, so during verification
`entry_count + pos > cache.len()` for node `a/b/c`, producing the corruption error.

## Fix
Rewrote `Index::invalidate_cache_tree_for_path` to mirror Git's
`do_invalidate_path` (`git/cache-tree.c`):
- invalidate `entry_count` at each ancestor node along the path, and
- when the final path component names an existing subtree node, drop that
  subtree node entirely (`children.retain(|c| c.name != final_component)`).

This is the dir→file case: invalidating `a/b` removes the `b` subtree node
(with its `c` child) from node `a`, so no stale descendant survives.

File changed: `grit-lib/src/index.rs` (only `invalidate_cache_tree_for_path`
plus a new private recursive helper `do_invalidate_cache_tree_path`).

## Result
`./scripts/run-tests.sh t2021-checkout-overwrite.sh` → 9/9 (stable across re-runs).

Regression checks (all unchanged from recorded baselines):
- t0090-cache-tree: 22/22
- t3700-add: 58/58
- t7001-mv: 54/54
- t7501-commit-basic: 146/146
- t2013-checkout-submodule: 70/74 (unchanged)

Unit tests: `cargo test -p grit-lib --lib` → 273 passed; only the 2 known
pre-existing `ignore::gitignore_glob_tests` failures remain (unrelated).

## Note (not my regression)
t3030-merge-recursive subtest 21 ("read-tree -m fails with 4 trees") fails on
the current shared binary: `read_tree.rs` allows up to 4 trees (`bail!("too many
trees (max 4)")`) where upstream caps at 3. That is in `grit/src/commands/read_tree.rs`
(another agent's area), entirely separate from the cache-tree invalidation path
I changed — `read-tree -m` arg-count validation runs before any index mutation.
Left untouched.
