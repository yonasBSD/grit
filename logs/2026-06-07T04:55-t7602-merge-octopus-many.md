# t7602-merge-octopus-many — mop-up (ticket f9b67b)

## Result
5/5 passing (was 3/5).

## Failing subtests at start
- 3: merge output uses pretty names
- 5: merge fast-forward output uses pretty names

## Root cause
`do_octopus_merge` (grit/src/commands/merge.rs) printed the per-head
"Trying simple merge with <name>" / "Fast-forwarding to: <name>" lines
**twice**:

1. In the real merge loop (lines ~4848 / ~4857) as each merge head is
   processed — this matches git-merge-octopus, which emits these lines
   during the merge.
2. Again in the finalize block (the old `if head_is_ancestor_of_all { ... }`
   branch around line ~5036) just before "Merge made by the 'octopus'
   strategy."

So `git merge c2 c3 c4` produced the three "Trying simple merge" lines
twice, and `git merge c1 c2` produced "Fast-forwarding to: c1 / Trying
simple merge with c2" twice. test_cmp against the expected (single)
output failed.

## Fix
Removed the redundant per-head printing in the finalize block, keeping
only `println!("Merge made by the '{strategy_name}' strategy.")` there.
The real merge loop remains the single source of the per-head lines,
which is correct: those lines should be emitted as each head is merged
(matching git-merge-octopus), and the summary line once at the end.

`head_is_ancestor_of_all` is still used for choosing the merge commit's
parents (no-ff vs ff), so it was left in place.

## Verification
- Direct repro in /tmp: `grit merge c2 c3 c4` and `grit merge c1 c2` now
  emit each per-head line exactly once.
- `./scripts/run-tests.sh t7602-merge-octopus-many.sh` → 5/5.
- `cargo test -p grit-lib --lib` → only the 2 known pre-existing
  ignore::gitignore_glob_tests failures (unrelated to this ticket).

## Coexistence note
merge.rs was being edited concurrently by other agents (rename/rename
2to1 conflict labels around line ~7160; modify/delete `-X ours/theirs`
handling around line ~9661). My octopus hunk (line ~5036) was swept into
another agent's commit 81b62bad2 when they ran `but commit` while my hunk
sat in the unassigned area. The code change and the t7602 TOML
(fully_passing = true) are committed on grit-t5-progress.
