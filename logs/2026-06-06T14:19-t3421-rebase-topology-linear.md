# t3421-rebase-topology-linear — bb1722

Date: 2026-06-06T14:19 UTC
Ticket: bb1722
File: tests/t3421-rebase-topology-linear.sh
Result: fully passing (63/63 counted; test 35 is a known `test_expect_failure`).

## Starting state
Fresh run at start: 51/64 (12 genuine failures). Failing subtests:
20, 24, 27, 30, 36, 39, 40, 42, 58, 59, 60, 61 (plus 35 = known breakage).

## Root causes and fixes (all in grit/src/commands/rebase.rs — shared sequencer machinery)

1. **Interactive rebase did not drop clean cherry-picks of upstream** (tests 24, 27, 30, 58).
   `filter_cherry_equivalents` was forced false for plain `-i`. Git's `sequencer_make_script`
   always drops clean cherry-picks (unless `--reapply-cherry-picks`/`--keep-base`) for every
   backend including interactive. Changed `filter_cherry_equivalents = !reapply_cherry_picks`.

2. **Begin-empty commits were wrongly dropped by cherry detection** (tests 36, 39, 40, 42).
   `collect_rebase_todo_commits` used rev-list `--cherry-pick`, which drops empty commits (all
   empty commits share the same empty patch-id, so they match each other). Git only drops
   `!is_empty && PATCHSAME` (sequencer.c make_script). Rewrote it to use `--cherry-mark` + a manual
   retain that keeps a cherry-equivalent commit when it started empty. Same fix applied to
   `commits_for_rebase_merge_walk` (was using both cherry_mark+cherry_pick) and
   `filter_redundant_patch_commits` (now skips begin-empty commits). Also extended the
   `--no-keep-empty` preliminary drop to cover interactive (test 39).

3. **--rebase-merges emitted `reset [new root]` instead of `reset onto`** (test 42).
   In `generate_rebase_merge_script`, when the first-parent walk stopped at a commit below the
   rebased set (an ancestor of onto collapsed by Git's limited symmetric walk), grit emitted
   `[new root]`, creating a spurious empty root commit. Git resets to `onto` there. Changed the
   uninteresting-boundary branch to emit `reset onto`. `[new root]` now only comes from an actual
   root-reaching walk.

4. **Interactive empty pick list did not move HEAD to onto** (tests 20, 24, 27, 30, 58; also
   t3404 test 48). When `-i`'s pick list is empty because the branch is behind upstream OR every
   commit was dropped as a clean cherry-pick (divergent), Git resets the branch to `onto` and
   prints "Successfully rebased and updated <ref>". grit printed "up to date" and left HEAD put.
   Added `rebase_reset_to_onto_noop` (reset HEAD/branch to onto, checkout tree, reflog, post-checkout
   hook, "Successfully rebased" message) and call it from the interactive empty path when
   `head != onto`.

5. **`rebase --root` (no `--onto`) rewrote linear history** (tests 59, 60, 61). The root commit has
   no parent, so it never matched the existing parent==HEAD fast-forward; it was rewritten with a
   fresh committer date, cascading to all descendants. Git fast-forwards a root pick when HEAD is
   unborn (`!parent && unborn` in do_pick_commit), reusing the original commits. Added a
   root-commit fast-forward in `cherry_pick_for_rebase`: pick of a parentless commit with unborn
   HEAD, fast-forward allowed (not `-f`/signoff/trailers/ws-fix) → reuse the original commit OID.
   Subsequent picks then fast-forward via the existing parent==HEAD path.

## Regression verification
Apples-to-apples (HEAD rebase.rs vs my rebase.rs, same other-agent files) comparison of genuine
(non-TODO) failure sets:
- t3424-rebase-empty: 0 regressions, +1 fix (test 11 --empty=stop default).
- t3430-rebase-merges: 0 regressions, +1 fix (test 21 refuse to merge ancestors).
- t3436-rebase-more-options: 0 regressions.
- t3404, t3400, t3418: failure sets byte-identical between HEAD and mine (the committed-baseline
  count drift on those files is from OTHER agents' in-flight changes, not this ticket).
- t3412, t3422, t3425, t3431, t3432, t3406, t3419, t3402, t3403, t3407, t3429: still fully passing.
grit-lib unit tests: only the 2 known ignore::gitignore_glob_tests failures (unrelated).
