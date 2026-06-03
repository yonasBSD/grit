# t6438-submodule-directory-file-conflicts

- Claimed `t6438-submodule-directory-file-conflicts.sh` after completing `t6423`.
- Current CSV baseline before refresh: 56 total, 23 passing, 33 failing.
- Official refresh: `./scripts/run-tests.sh t6438-submodule-directory-file-conflicts.sh --verbose`
  reports 39/56 passing and regenerated `data/test-files.csv` plus dashboards.
- Added a merge preflight that aborts before replacing a checked-out gitlink with regular files or
  directories, while preserving relocated gitlink conflict handling for t6437.
- Focused debug run for replacement scenarios passed all targeted cases; full debug run reached
  55/56 with only `merge --no-ff` "replace directory with submodule" still failing.
- Official refresh after `cargo build --release -p grit-cli`:
  `./scripts/run-tests.sh t6438-submodule-directory-file-conflicts.sh --verbose` reports 55/56
  passing and regenerated `data/test-files.csv` plus dashboards.
