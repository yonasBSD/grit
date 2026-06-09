# t0610-reftable-basics.sh — fully passing (91/91)

Ticket: 8bb469 (group reftable-refstore, thread A)

## Starting state
Prior agent (ticket bc27f1, t0614) fixed the GIT_TEST_DEFAULT_REF_FORMAT env
precedence bug in init.rs, which carried t0610 from 42/91 to 89/91. Re-running
fresh confirmed 89/91 with two remaining failures:

- 81 worktree: pack-refs in main repo packs main refs
- 82 worktree: pack-refs in worktree packs worktree refs

## Root cause
Both subtests do `test_commit -C repo A` (which also creates a tag), then
`worktree add ../worktree` with autocompaction disabled, then assert the main
reftable stack has exactly 3 tables before pack-refs. grit's stack had 4.

`git worktree add` with a new branch was writing the new `refs/heads/<wt>` ref
and its reflog ("branch: Created from HEAD") as TWO separate reftable
transactions in the main stack:
  - grit/src/commands/worktree.rs called refs::write_ref (1 table) and then
    refs::append_reflog (a 2nd table).
Upstream git writes the branch ref + reflog in a single transaction => 1 table.
So grit's main stack grew by 2 instead of 1 during `worktree add`, making it 4
instead of 3 and breaking the post-pack-refs line-count assertions.

## Fix
grit/src/commands/worktree.rs, branch-creation block in cmd_add:
For reftable repos, create the branch ref and its reflog in ONE reftable
transaction via grit_lib::reftable::reftable_write_ref(common, branch_ref,
commit_oid, Some(identity), Some("branch: Created from HEAD")) (which writes the
ref + log record into a single table). The loose-ref backend path is unchanged
(still write_ref + append_reflog). Also switched the "branch already exists"
probe to refs::resolve_ref for reftable repos since there is no loose ref file.

## Verification
- t0610-reftable-basics: 91/91 (was 89/91)
- No regressions: t2400-worktree-add 232/232, t2401-worktree-prune 13/13,
  t0613-reftable-write-options 11/11, t0614-reftable-fsck 7/7,
  t1405-main-ref-store 16/16, t1406-submodule-ref-store 15/15.
- cargo test -p grit-lib --lib: 276 pass; only the 2 known pre-existing
  ignore::gitignore_glob_tests failures remain (not mine).
