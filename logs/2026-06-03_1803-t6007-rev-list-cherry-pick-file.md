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
