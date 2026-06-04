# t6416-recursive-corner-cases

Task: make `t6416-recursive-corner-cases.sh` pass.

Starting point:
- `t6-plan.md` reports 24/37 passing, 13 failing, with 3 expected failures.

Progress:
- Claimed the task after completing `t6402-merge-rename.sh`.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; prior merge fixes
  refreshed the file to 26/40 passing, 11 failing, with 3 expected failures.
- Fixed recursive virtual-base materialization for stage-3-only D/F conflict entries so
  `merge-tree C B` preserves the relocated `a~B` file in the written tree. The outer
  criss-cross merge can now detect the virtual-base rename source.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; improved to
  29/40 passing, 8 failing, with 3 expected failures. Newly passing: D1/E1, D1/E2,
  and E2/D1 directory/file criss-cross cases. Remaining ordinary failures: 13, 17, 28,
  30, 32, 34, 38, and 40.
- Fixed the reverse rename/delete direction so a clean virtual-base directory side does not
  force the renamed file to `a~SIDE`; it remains staged at `a` like Git.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; improved to
  30/40 passing, 7 failing, with 3 expected failures. Newly passing: E1/D1. Remaining
  ordinary failures: 17, 28, 30, 32, 34, 38, and 40.
- Fixed ordinary rename/rename(1to2) conflicts to stage the once-merged content at both
  destinations. This matches D1/E4, where the virtual source is renamed to `a` and `a2`.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; improved to
  31/40 passing, 6 failing, with 3 expected failures. Newly passing: D1/E4. Remaining
  ordinary failures: 28, 30, 32, 34, 38, and 40.
