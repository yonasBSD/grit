# t6422-merge-rename-corner-cases

Ticket: 65325c. Group: merge-ort (thread C).

Start state: 14/26 passing. Failing (non-known-breakage): 9, 16, 18, 19, 25, 26.
(Tests 2, 4, 5, 6, 14, 15 are `test_expect_failure` known breakage.)

## Fixes

### #9 disappearing dir in rename/directory conflict handled
`grit/src/commands/merge.rs`, Case 1 rename pass. When ours renames `sub/file` -> `sub`
and theirs only modified `sub/file` (the rename source), the directory `sub/` disappears
once the rename is consumed; there is no real file/directory conflict. Added
`only_tree_descendant_is()` helper and a `theirs_dir_is_only_rename_source` guard so the
rename handler content-merges the two versions instead of bailing to the D/F pass.

### #16 rename/rename/add-dest merge still knows about conflicting file versions
`grit/src/commands/merge.rs`, Case 2 rename/rename(1to2) staging block. ours renamed
`a`->`c` + added `b`; theirs renamed `a`->`b` + added `c`. The 1to2 logic staged the
add at `ours_target` (theirs's added `c` at stage 3) but missed the symmetric add at
`theirs_new_path` (ours's added `b` at stage 2). Added the symmetric staging plus proper
two-way conflict-marked working-tree content via new `two_way_conflict_blob()` helper
(labels `HEAD` / `their_name`).

### #18 rrdd: rename/rename(2to1)/delete/delete
`grit/src/commands/merge.rs` Case 1 add/add-at-rename-dest. When theirs' file at our
rename destination is itself a rename *target* (both sides renamed distinct sources to the
same path, each deleting the other's source), emit `CONFLICT (rename/rename)` instead of
`rename/add`. The existing `rename/delete` line satisfies the test's `(rename.*delete)`
requirement; the new line satisfies the `(X/X)` requirement.

### #25 binary rename/rename(1to2)
`grit/src/commands/merge.rs` Case 2 1to2 staging. Binary files cannot be merged; on a
`BinaryConflict` from the source content merge, keep ours' blob at `ours_target` and
theirs' blob at `theirs_new_path` separately (instead of a single merged blob at both).
Working-tree content now reads each destination's own staged blob.

### #19 mod6: chains of rename/rename(1to2) and rename/rename(2to1)
`grit/src/commands/merge.rs` colliding-cycle prepass. The once-merged source blob placed at
two colliding destinations must use widened conflict markers (size 8) and an empty base
label, matching git merge-ort's `handle_content_merge(..., 1 + 2*call_depth)`. Added
`try_content_merge_ext()` with an `extra_marker_size` param (added to the resolved marker
size); the prepass calls it with `extra_marker_size = 1` and base label `""`.

## Remaining: 26
- 26 submodule/directory preliminary conflict: deep recursive merge with a virtual merge
  base. The virtual base merges A1 (folder=submodule/gitlink) with B1 (folder=directory
  tree); merge-ort renames the submodule to `folder~Temporary merge branch 2` in the
  virtual base, producing a rename/delete in the outer merge. grit currently clean-merges
  to a single gitlink at `folder` (exit 0); expected is exit 1 with 2 gitlink stages
  (1 and 2) at `folder`. NOTE: real git 2.52 also diverges from the ported expectation
  here (1 entry, different submodule oid), so this needs virtual-base submodule/directory
  rename handling in the recursive merge machinery. Left for mop-up.
