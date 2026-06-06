# t6415-merge-dir-to-symlink — ticket c8fa0b

Date: 2026-06-06T18:18Z
Ticket: c8fa0b — tests/t6415-merge-dir-to-symlink.sh
Result: 24/24 passing (was 21/24 on fresh re-run; earlier group tickets had
already lifted it from 13/24).

## Failing subtests at start of this session

- 2: checkout does not clobber untracked symlink
- 14: do not lose untracked in merge (resolve)
- 16: do not lose modifications in merge (resolve)

## Root causes & fixes

### Test 2 — `grit/src/commands/checkout.rs` (`check_untracked_overwrite`)

After `git rm --cached a/b`, `a/b` is an untracked symlink -> `b-2`. The target
tree (`start`) materializes `a/b/c/d` as a directory entry. grit's
untracked-overwrite check only ran `untracked_leading_path_in_the_way` when
`!abs_path.exists() && !abs_path.is_symlink()`. But `a/b/c/d` resolves *through*
the untracked symlink `a/b` to the still-present `a/b-2/c/d`, so `abs_path.exists()`
returned true and the leading-path (check_leading_path / ERROR_NOT_UPTODATE_FILE)
detection was skipped — checkout silently succeeded (exit 0) instead of failing.

Fix: hoist the `untracked_leading_path_in_the_way` check above the
`abs_path.exists()` branching so an untracked non-directory ancestor (file or
symlink) is always treated as a blocker, matching upstream `check_leading_path`
which lstats each leading component. The function already uses `symlink_metadata`
(lstat), so a symlink ancestor is correctly flagged as a non-directory.

### Tests 14 & 16 — `grit/src/commands/merge.rs` (`attempt_trivial_in_index_merge`)

`git merge -s resolve main` was taking the "really trivial in-index merge" path
("Wonderful.") and committing, silently clobbering untracked `a/b/c/e` (14) or
modified `a/b/c/d` (16). Upstream git's trivial merge runs through `unpack_trees`
(read_tree_trivial), which performs worktree `verify_uptodate` / `verify_absent`;
for both of these D/F transitions (`a/b` dir -> symlink) it prints "Nope." and
falls through to the real strategy. grit's `trivial_three_way_index` resolved the
paths cleanly at the *tree* level and never consulted the worktree.

Fix: after computing the sorted trivial-merge index, call the shared
`bail_if_merge_would_overwrite_local_changes` (the same validation the real merge
uses) with the HEAD/ours tree map. If it would overwrite a dirty tracked file or
remove a directory holding untracked content, print "Nope." and return Ok(false)
so the merge falls through to the resolve strategy. The resolve strategy then
correctly fails (16, "Entry not uptodate") or completes the D/F merge (14). Both
now match upstream and are `test_must_fail` as expected.

## Regression sweep (isolated --data-dir, not affecting canonical data/tests)

- t6402-merge-rename 46/46, t6400-merge-df 7/7, t3030-merge-recursive 26/26
- t7605-merge-resolve 4/4, t6401-merge-criss-cross 4/4, t6407-merge-binary 3/3
- t6424-merge-unrelated-index-changes 19/19, t7601-merge-pull-config 65/65
- t2007-checkout-symlink 4/4, t2023-checkout-m 5/5, t6436-merge-overwrite 18/18
- t7611-merge-abort 19/19
- t7600-merge 82/83, t6422-merge-rename-corner-cases (only pre-existing #26),
  t7602-merge-octopus-many 3/5 — all unchanged vs canonical TOML (pre-existing).
- t2021-checkout-overwrite #2/#3 currently fail, BUT verified by rebuilding with my
  two source files stashed: those failures reproduce WITHOUT my changes (a
  concurrent agent's `git add -A` "corrupted cache-tree" regression on dir->file).
  Not mine, not this ticket.

grit-lib --lib: 269 passed, only the 2 known ignore::gitignore_glob failures.
