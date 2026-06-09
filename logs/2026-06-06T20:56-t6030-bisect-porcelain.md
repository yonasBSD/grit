# t6030-bisect-porcelain.sh — ticket e11815

Subsystem group "bisect" (thread C). All bisect logic lives in `grit/src/commands/bisect.rs`.

## Baseline
Fresh run at session start: 84/96 (failing 39, 56, 57, 65-69, 71, 72, 89).

## Root cause
The previous bisect selection used a naive midpoint over the path-filtered rev-list
candidate set plus an ad-hoc skip handler. That diverged from Git's weight-based
`find_bisection` / `managed_skipped` machinery (`git/bisect.c`), giving wrong commits and
wrong "only skipped commits left" lists.

## Fix (all in grit/src/commands/bisect.rs)
1. Ported Git's `find_bisection` + `do_find_bisection` weight computation (per-candidate
   reachable-candidate count via full-ancestry walk), including the `approx_halfway`
   early-out so the chosen midpoint matches `git rev-list --bisect` exactly (verified the
   weight table for linear histories: 3->2, 4->2, 5->2, 6->3, 7->3, 8->4, ...).
2. Ported `managed_skipped` + `filter_skipped` + `skip_away` (`get_prn`/`sqrti`) so the
   "tried"/skip list and skip-away selection match Git. With skips, uses the
   `best_bisection_sorted` order (distance desc, oid asc); without skips uses the non-ALL
   `best_bisection` selection.
3. `error_if_skipped_commits` now driven by the real `tried` list (collected skipped
   commits) rather than an ad-hoc head-ancestor heuristic.
4. `bisect_skipped_commits_log` no longer appends `bad` separately — Git re-walks
   `bad ^good` and lists every yielded commit (the set already includes `bad`).
5. Candidate rev-list switched from `OrderingMode::Default` to `OrderingMode::Topo`:
   grit's path-limited date-order walk currently returns empty for pathspec+merge ranges
   (a rev_list regression another agent is editing in grit-lib/src/rev_list.rs); topo
   ordering sidesteps it and is correct for bisect (order does not affect weight ranking).
6. Added a "No testable commit found" pre-check: with pathspecs whose filtered set is empty
   but whose unfiltered `bad ^good` range is non-empty, emit BISECT_NO_TESTABLE_COMMIT(4)
   before the "was both good and bad" path (Git keeps TREESAME commits in revs.commits).

## Status
95/96 in the honest harness run. 96/96 when `SHELL_PATH=/bin/sh` is exported.

## Final follow-up fixes (second commit)
- Reworked `find_bisection` to run over the full interesting DAG (`bad ^good`, no pathspec)
  with `TREESAME` marking instead of only the path-filtered candidate set. Git's
  `count_interesting_parents` counts parents that are not UNINTERESTING (including TREESAME
  ones), so the merge/single-parent classification and weight propagation now match Git;
  `nr` still counts only candidates. This fixed test 56 (pathspec bisection through merges).
- Order the DAG list by committer date (newest-first, rev-list order as tiebreak) before the
  oldest-first reversal so `best_bisection`'s tie-break matches Git's date-ordered limited
  walk. Needed because the candidate rev-list is topo-ordered (the path-limited date-order
  walk in grit-lib is currently broken for pathspec+merge ranges — a rev_list regression
  another agent owns); sorting by date locally restores the correct selection.

## Remaining (not a grit bug)
Test 69 ("demonstrate identification of damage boundary") uses `git bisect run "$SHELL_PATH"
-c '...'`. The grit harness does not generate/source `GIT-BUILD-OPTIONS`, so `$SHELL_PATH`
is empty and `git bisect run` execs an empty command (exit 127). With `SHELL_PATH=/bin/sh`
exported the test passes and the whole file is 96/96. Fixing this requires the harness to
export `SHELL_PATH` (test-lib.sh / run-tests.sh), which is out of scope for this ticket and
must not be edited per the rules. Left as the single honest-run failure.
