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
- Fixed symlink add/add virtual-base materialization so unresolved symlink pairs are omitted from
  the synthetic ancestor instead of creating a stage-1 symlink entry in the final criss-cross
  conflict.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; improved to
  32/40 passing, 5 failing, with 3 expected failures. Newly passing: symlink add/add. Remaining
  ordinary failures: 30, 32, 34, 38, and 40.
- Fixed submodule modify/modify, submodule add/add, and submodule-vs-symlink add/add criss-cross
  handling by preserving/omitting gitlink virtual-base entries appropriately, staging gitlink
  add/add conflicts without a fake stage 1, suppressing unmerged gitlink directories from
  `ls-files -o`, and avoiding extra symlink side files for submodule-vs-symlink conflicts.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; improved to
  35/40 passing, 2 failing, with 3 expected failures. Newly passing: submodule modify/modify,
  submodule add/add, and conflicting entry types (submodule vs symlink). Remaining ordinary
  failures: 38 and 40.
- Fixed recursive virtual-base nested content by carrying the real sub-base marker label into
  internal temporary-branch merges, lengthening pre-existing conflict marker lines when they are
  merged again, and using the bare `merged common ancestors` label for same-path outer criss-cross
  content conflicts.
- Ran `./scripts/run-tests.sh t6416-recursive-corner-cases.sh --verbose`; improved to
  36/40 passing, 1 failing, with 3 expected failures. Newly passing: virtual merge base with
  nested conflicts. Remaining ordinary failure: 38.
