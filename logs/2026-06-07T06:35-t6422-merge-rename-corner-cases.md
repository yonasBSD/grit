# t6422-merge-rename-corner-cases — mop-up round 1

Ticket: 65325c. Group: merge-ort (thread t5 mop-up).

Start state on fresh re-run: 19/26 passing. Only non-known-breakage failure was
#26 (`submodule/directory preliminary conflict`). Tests 2, 4, 5, 6, 14, 15 are
`test_expect_failure` known breakage.

## Fix for #26 — submodule/directory preliminary conflict

The merge `A2 (A^0)` x `B2 (B^0)` has two merge bases:
- A1 (`folder` = submodule / gitlink)
- B1 (`folder/` = directory tree A..E)

These must be folded into a virtual merge base. git merge-ort renames the submodule
to `folder~Temporary merge branch 2` in the virtual base and keeps the directory
intact; the outer merge then sees a `folder~Temporary merge branch 2` -> `folder`
gitlink rename that becomes a rename/delete (folder-as-submodule was deleted on the
A2 side). Expected: exit 1 with 2 gitlink stages (1 and 2) at `folder`.

Two changes, both in `grit/src/commands/merge.rs`:

### 1. Virtual-base submodule-vs-directory relocation
`merge_trees`, second-pass `(None, None, Some(te))` branch (theirs added a submodule
at `path` while ours holds a directory `path/...`). Previously this called
`conflict_submodule_vs_non_gitlink(relocate_file=false)`, which staged a gitlink and
a single directory file *both at the bare `path`* — collapsing the virtual tree to one
`folder` entry and dropping `folder/B..E`.

Now, when `criss_cross_outer_merge` (virtual-base / criss-cross build), it relocates the
gitlink to `path~<their_name>` (stage 3) and leaves the directory files in `ours_entries`
to be emitted by their own per-path iterations. In the virtual-base tree builder a
stage-3-only gitlink at `folder~Temporary merge branch 2` becomes a clean gitlink there,
and `folder/A..E` are emitted as clean adds — so the virtual tree is complete.

### 2. Virtual-base 2-base fold order (equal-date tiebreak)
`create_virtual_merge_base`. git folds `reverse(get_merge_bases())` with
branch1=`prev`, branch2=`next`. For equal-date bases the order is the reverse of git's
`merge-base --all` output (which grit emits OID-ascending), so the submodule base (A1)
must end up as `next` = "Temporary merge branch 2". The 2-base equal-date tiebreak was
OID-ascending (`a.cmp(b)`), which put A1 first (branch1) and produced `folder~HEAD`.
Flipped to OID-descending (`b.cmp(a)`) so A1 lands as branch2. Date-differing bases are
unaffected (date comparison dominates); confirmed no regression in t6416/t6404/t6437.

## Result
t6422: 20/26, `fully_passing = true` (6 expect_failure known breakage remain).
No regressions: t6402 46/46, t6404 6/6, t6406 13/13, t6409 12/12, t6416 37/40 (3 known
breakage, failing=0), t6418 11/11, t6423 80/82 (pre-existing), t6424 19/19, t6425 1/1,
t6437 22/22.

t3030 #21 "read-tree -m fails with 4 trees" is failing but is UNRELATED to this change
(read-tree never reaches `create_virtual_merge_base`; the working tree also has other
agents' in-flight edits to read-tree-adjacent files). Left alone.
