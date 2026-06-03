# t7407-submodule-foreach

## 2026-06-02 19:49

- Claimed `t7407-submodule-foreach.sh` after `t7506-status-submodule.sh` reached 40/40.
- Starting baseline from `data/test-files.csv`: 4/23 passing, 19 failing.
- Direct run `cd tests && sh t7407-submodule-foreach.sh -v` initially failed only test 5.
  Plain `git submodule update --init` was initializing nested submodules before the foreach
  command, so the subsequent `test_must_fail rev-parse nested1/nested2/.git` check failed.
- Removed the CLI-only `implicit_recursive` override for `submodule update`; explicit
  `--recursive` and internal recursive callers still use the existing recursive flags.
- Verification:
  - `cd tests && sh t7407-submodule-foreach.sh -v` passed 23/23.
  - `./scripts/run-tests.sh t7407-submodule-foreach.sh --verbose` passed 23/23 and refreshed
    `data/test-files.csv` plus dashboards.
  - Regression `./scripts/run-tests.sh t7406-submodule-update.sh --verbose` remained 70/70.
