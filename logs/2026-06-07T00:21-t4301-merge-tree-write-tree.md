# t4301-merge-tree-write-tree — rename/rename(2to1)/delete/delete + dir-rename rename/delete

Ticket: fbf6e2. File: tests/t4301-merge-tree-write-tree.sh.

## Starting state
42/44 passing. Failing subtests:
- 17: rename/rename(2to1)/delete/delete conflict
- 19: directory rename + rename/delete + modify/delete + directory/file conflict

## Subtest 17 — rename/rename(2to1)/delete/delete
Scenario: A renames foo->baz & deletes bar; B renames bar->baz & deletes foo.
Both sources renamed to the same dest `baz`, and each source is deleted on the
opposite branch. The existing 2to1 pre-pass (merge.rs ~7498) requires both sources
to survive on the opposite side, so it `continue`s here. Processing then fell into
the ours-rename rename/delete handler, which staged baz 2/3, content-merged
(Auto-merging baz), and pushed a `rename/rename` description — but that description
was silently dropped by the merge-tree -z formatter (it requires rr_ours/rr_theirs
dests), and the second rename/delete (for bar) was never emitted.

Per git/merge-ort.c handle_rename_via_dir `collision && source_deleted`
(rename/rename(2to1)/delete/delete): git reports a `rename/delete` for EACH renamed
source and leaves the two blobs as add/add stages 2/3 (no stage 1) at the dest.

Fix (grit/src/commands/merge.rs, ~8195): when the colliding theirs source is also
deleted on ours, emit a second `rename/delete` (bar renamed to baz in B, deleted in A)
plus an `add/add` description instead of the dropped `rename/rename`.

## Subtest 19 — directory rename + rename/delete into a directory
Scenario: A removes foo, renames olddir/->newdir/, adds newdir/bar/file; B modifies
foo and renames foo->olddir/bar (transitively newdir/bar via A's dir rename).
The Case-2 theirs-rename loop `continue`s (deferring to the D/F pass) when the dest
has a tree descendant, so the rename/delete for foo (deleted on A) was never emitted —
only the file/directory + modify/delete from apply_directory_file_conflicts showed.

Fix (apply_directory_file_conflicts, ~11140): when the file landing in `path/` got
there via a rename whose source the OTHER side deleted (and didn't itself rename),
emit the `rename/delete` (foo renamed to newdir/bar in B, but deleted in A) at the
bare directory-rename-adjusted destination, matching merge-ort's rename-collection
message ordering.

## Regression found & fixed
First cut used `kind: "rename/add"` for the 2to1 add/add, which the regular `git
merge -s recursive` path prints as `CONFLICT (rename/add)` — breaking t6422 #18
(`test_grep "CONFLICT (\(.*\)/\1)"` needs an X/X message). Switched to a new
`kind: "add/add"` that prints as `CONFLICT (add/add)` in both the merge command path
and the merge-tree -z/non-z formatters (merge_tree.rs now maps `add/add` like
`rename/add`).

## Result
t4301: 44/44 fully passing.

## Cross-checks (no regressions from my change)
- t6422: only #26 (submodule/directory) fails — pre-existing; #18 fixed.
- t6423: only #23 fails — PRE-EXISTING, worktree conflict-marker labels (`HEAD:y/d`
  vs `HEAD`) from the untouched 2to1 pre-pass; my changes are inert for #23 (no
  source deletion, no D/F at y/d).
- t6430 #25 (symlink-ancestor cherry-pick), t6437 #16/#22 (submodule): pre-existing,
  separate code paths (cherry-pick worktree checkout / submodule), unrelated.
- t6429, t3030, t6436 fully pass.
