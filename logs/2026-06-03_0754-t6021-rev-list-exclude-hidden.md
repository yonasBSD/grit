# t6021-rev-list-exclude-hidden

## Context

- Started from the t6 family objective: finish all t6 tests, grouped by dependency/topic and
  choosing high-failure files first.
- `t6-plan.md` was missing, so it was created from current `data/test-files.csv` and this task was
  claimed as the highest-failure current t6 row.
- Current CSV baseline: `t6021-rev-list-exclude-hidden` has 62 total tests, 1 passing, 61 failing.

## Work Log

- Claimed `t6021-rev-list-exclude-hidden.sh`.
- Initial harness run was blocked because `target/release/grit` did not exist; release build first
  failed on a stale `merge --abort` caller to `checkout_merge_reset_worktree`.
- Fixed the stale merge caller by passing explicit non-recursive submodule behavior.
- Direct `sh t6021-rev-list-exclude-hidden.sh -v` showed `rev-list` rejected
  `--exclude-hidden` and `--exclude` in CLI parsing. Added CLI handling for hidden-ref sections,
  duplicate/unsupported-section errors, pseudo-ref incompatibility with branches/tags/remotes, and
  exclusion-aware expansion for `--all` and `--glob`.
- Second direct run improved to 50/62. Remaining failures were empty pseudo-ref expansions
  returning `no revisions specified` and `GIT_NAMESPACE` listing only namespaced refs.
- Switched this expansion path to physical ref listing and allowed empty pseudo-ref expansion to
  succeed with empty output.
- Direct verification: `cd tests && sh t6021-rev-list-exclude-hidden.sh -v` passed 62/62.
- Harness verification: `./scripts/run-tests.sh t6021-rev-list-exclude-hidden.sh --verbose`
  passed 62/62 and refreshed `data/test-files.csv` plus dashboards.
