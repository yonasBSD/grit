# t6007-rev-list-cherry-pick-file

- Claimed `t6007-rev-list-cherry-pick-file.sh` after completing `t6006`.
- Baseline from `t6-plan.md` / `data/test-files.csv`: 6/23 passing, 17 failing.
- Read the upstream/local test setup and `git-rev-list` docs for `--cherry-pick`: patch-equivalent
  commits on opposite sides of a symmetric difference should be omitted, including with path
  limiting.
- First increment: accepted `name-rev --no-refs` by clearing accumulated ref filters, and made
  plain `rev-list --count --left-right` print Git's two count columns instead of a third zero.
- Direct verbose run and official harness `./scripts/run-tests.sh t6007-rev-list-cherry-pick-file.sh`
  improve from 6/23 to 8/23 and refresh `data/test-files.csv` plus dashboards.
- Second increment: added path-limited patch-id computation for cherry equivalence, made
  `--cherry-mark --left-right` preserve `<` / `>` for non-equivalent commits, made `--cherry`
  behave as `--right-only --cherry-mark --no-merges`, and fixed cherry-mark count columns.
- Direct verbose run reaches test 22 before the duplicate patch-id case fails; official harness
  `./scripts/run-tests.sh t6007-rev-list-cherry-pick-file.sh` improves from 8/23 to 21/23 and
  refreshes `data/test-files.csv` plus dashboards.
- Third increment: changed cherry equivalence to retain all commits for a patch-id on the indexed
  side so duplicate add/revert/add sequences on both sides are all omitted by `--cherry-pick`.
- Direct verbose run reaches the final `...shy-diff` parser case; official harness
  `./scripts/run-tests.sh t6007-rev-list-cherry-pick-file.sh` improves from 21/23 to 22/23 and
  refreshes `data/test-files.csv` plus dashboards.
- Final increment: normalized omitted symmetric range endpoints in `rev-list` CLI handling to
  `HEAD`, so `...shy-diff` no longer tries to resolve an empty revision.
- Direct verbose run passes all 23 tests; official harness
  `./scripts/run-tests.sh t6007-rev-list-cherry-pick-file.sh` records 23/23 and refreshes
  `data/test-files.csv` plus dashboards.
