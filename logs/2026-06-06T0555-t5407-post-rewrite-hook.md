# t5407-post-rewrite-hook — mop-up round 1 (ticket 66f44b)

Date: 2026-06-06T05:55 UTC
Agent: schacon+claude-opus@gmail.com
Branch: grit-t5-progress

## Starting state

Prior agent reached 16/17 (commit adbef8c4...). Remaining failure: subtest 11
`git rebase with failed pick`. Their note pointed at the merge step not aborting
on an untracked-overwrite; re-running fresh showed the `merge -C merge-E E` step
(which shells out to `grit merge`) ALREADY aborts correctly with
"would be overwritten" — another agent's merge work had cascaded.

## Root cause of the remaining failure

The test's todo interleaves `exec >FILE` (creating untracked files) with
`pick`/`merge`/`fixup`. Each step that would materialize the just-created
untracked file must abort:

- `merge -C merge-E E` over untracked `bar` → handled (shells to `grit merge`).
- `pick G` over untracked `G` → NOT handled.
- `pick H` over untracked `H` → NOT handled.
- `fixup I` over untracked `I` → NOT handled.

The internal pick path `cherry_pick_for_rebase` (grit/src/commands/rebase.rs)
only ran `preflight_cherry_pick_cwd_obstruction`, which covers cwd-removal and
submodule obstructions — NOT the general "newly-added entry would overwrite an
untracked working-tree file" case (Git's unpack-trees `verify_absent`). The
standalone cherry-pick command already guards this via
`super::reset::check_untracked_cherry_pick_obstruction`; the rebase path was
missing it, so `pick G` silently overwrote the untracked file and the rebase ran
straight to the end.

## Fix

grit/src/commands/rebase.rs, in `cherry_pick_for_rebase` full-merge pick path
(just before `preflight_cherry_pick_cwd_obstruction` / `write_index`):

```rust
super::reset::check_untracked_cherry_pick_obstruction(wt, &old_index, &merged_index)?;
```

`find_untracked_obstruction` (the underlying helper) only inspects stage-0
entries whose path is absent from the old index and present untracked on disk,
so it is conflict-safe and matches the standalone cherry-pick behavior. The bail
happens before any index/worktree mutation, so `--continue` can be retried
cleanly (state left untouched), which the test relies on (four retries).

## Result

- t5407-post-rewrite-hook: 16/17 -> 17/17 (full pass).
- Manual repro: all four obstruction steps abort with "would be overwritten",
  final `--continue` succeeds.
- Regression checks (all still fully passing): t3403 (20/20), t3407 (17/17),
  t3417 (4/4), t3418 (30/30), t3429 (7/7), t3510 (55/55). t3404 unchanged-to-
  improved (77/132 vs committed baseline 42/132; not regressed by this change).
- grit-lib unit tests: 269 pass; 2 pre-existing failures in
  `ignore::gitignore_glob_tests` are unrelated (gitignore globbing; my edit is
  CLI-only in grit/src and touches no grit-lib code).
- No new clippy warnings in the edited region.
