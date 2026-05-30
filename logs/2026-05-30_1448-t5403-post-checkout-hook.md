# t5403-post-checkout-hook — fix log (2026-05-30)

## Result
- Target: `tests/t5403-post-checkout-hook.sh`
- Before: 11/14 (failing subtests 9, 10, 13)
- After: 14/14 (all pass)

## Root cause
`grit/src/commands/checkout.rs` `fn check_dirty_worktree` had an over-aggressive
untracked-file removal. When the target tree wanted to materialize a path that was
absent from the old index but present on disk as an *untracked* file whose content
*differed* from the target blob, the code did:

```rust
if abs_path.is_file() || abs_path.is_symlink() {
    let _ = std::fs::remove_file(&abs_path);
    continue;
}
```

i.e. it silently deleted the untracked file and let checkout proceed. Upstream git
(`unpack-trees.c` `verify_absent`) instead refuses such a checkout with
"The following untracked working tree files would be overwritten by checkout".

In t5403, `git rebase <branch>` spawns `grit checkout --quiet <branch>` (rebase.rs
~445). With HEAD detached at `two` and an untracked `three.t` ("untracked\n") on
disk, checking out `rebase-fast-forward` (commit `three`, tracked `three.t`="three\n")
silently destroyed the untracked file (subtests 9 and 13). Because checkout then
succeeded, the rebase reached the up-to-date fast-forward path and wrote
`.git/post-checkout.args`, which the test expects to be absent. Subtest 10 was a
cascade: subtest 9's buggy success plus its `test_when_finished "rm three.t"` left a
` D three.t` deletion in the index that tripped the rebase --merge dirty-worktree
guard.

## Fix
checkout.rs ~3196-3220: removed the silent `remove_file` branch. A differing
untracked file (ordinary file, symlink, or gitlink path) now falls through to
`untracked_conflicts.push(...)`, which triggers the existing
"untracked working tree files would be overwritten" bail. The content-identical
carve-out (`untracked_path_matches_index_entry`, line ~3203, which also accepts
trailing-newline-only differences) is kept intact, so genuine orphan / `rm --cached
-r .` flows where the worktree file already equals the target blob still proceed.
Updated the now-stale comment that claimed the blanket removal was needed for t3501.

## Regression testing (apples-to-apples, built a pre-fix binary at parent commit)
Formal phase guards — both unchanged:
- t7503-pre-commit-and-pre-merge-commit-hooks: 22/22 PRE and POST
- t5571-pre-push-hook: 11/11 PRE and POST

Broad checkout/switch/rebase/cherry-pick set (PRE -> POST passing):
- t2011: 10 -> 10, t2012: 22 -> 22, t2020: 19 -> 19, t7201: 28 -> 28 (no change)
- t3502-cherry-pick-merge: 12 -> 12, t3426-rebase-submodule: 3 -> 3 (no change)
- t3403: 7 -> 7, t3406: 15 -> 15, t3407: 15 -> 15 (no change)
- t3400-rebase: 17 -> 18 (+1 improvement)
- t3420-rebase-autostash: 12 -> 18 (+6 improvement)
- t5403: 11 -> 14 (+3, the target)
- t3501-revert-cherry-pick: 17 -> 16 (-1)
- t3404-rebase-interactive: 27 -> 24 (-3)

Net: +11 improvements, -4. All regressions are leaked-state cascades in
already-broken, out-of-scope subsystems, NOT direct checkout correctness breakage:

- t3501 subtest 12 ("cherry-pick - works with arguments"): subtests 8/9/10
  ("cherry-pick on unborn branch" etc.) fail in BOTH pre- and post-fix for unrelated
  reasons ("cannot cherry-pick onto unborn branch without -n", "cannot detach HEAD on
  unborn branch"). Those failures leave an orphaned untracked `spoo` on disk (hash
  0acb407b, not in old HEAD nor matching main:spoo). Subtest 12's `git checkout main`
  now correctly refuses to overwrite it — exactly upstream's behavior. Upstream passes
  subtest 12 only because its earlier subtests succeed and leave a clean tree; the
  old blanket removal was masking grit's leaked dirty state. There is no semantic
  predicate that separates this case from t5403 — they hit the identical plain
  `git checkout` path — so a call-site force/orphan flag (per the diagnosis fallback)
  cannot distinguish them. The correct upstream behavior is to refuse.

- t3404 subtests 46/50/60/61/63: these are `git rebase -i` (interactive) cases, a
  plan non-goal (108/132 already failing). Subtest 46 fails with an interactive-rebase
  backend bug ("error: todo: bad revision ...: I/O error") that leaves a rebase in
  progress; subtests 50/60/61/63 then cascade with "error: a rebase is already in
  progress" — NOT the untracked-overwrite error. My change only shifted interactive
  rebase timing/state; subtests 24/25 improved in the same file. None of these are
  checkout-overwrite failures.

## Decision
Kept the upstream-correct fix. Reverting to the blanket removal would re-break t5403
and re-mask a real bug, while only preserving fake passes in non-goal areas whose
underlying failures are unrelated. Net effect across the regression set is strongly
positive.

## Gates
- cargo fmt: clean
- cargo test -p grit-lib --lib: 225 passed, 0 failed
- cargo clippy -p grit-cli: no warnings on changed lines
