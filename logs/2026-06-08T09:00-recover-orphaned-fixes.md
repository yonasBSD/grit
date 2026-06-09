# Recover orphaned grit fixes + test-portability finish-up — 2026-06-08

After the `test-portability-fixes` workflow completed, the working tree held ~600
lines of **uncommitted grit source changes** across 16 files (grit-lib/src/rev_list.rs,
diffstat.rs; grit/src/commands/{checkout,log,rebase,commit,status,...}.rs). These are
orphaned, coherent fixes from the earlier final-push / regression-sweep runs whose
commits never landed (GitButler shared-stage races, and the final batch was blocked
when the session entered plan mode for the libification task).

**Validation before committing (the tree is exactly what produced the recorded
pass-counts):**
- `cargo build --release -p grit-cli -j 4` → exit 0 (3 pre-existing warnings).
- `cargo test -p grit-lib --lib` → 276 passed; only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures remain.

Committed as recovered work (isolated + revertible) so the branch source matches the
committed test results. Also committed the two differential-verified test-body fixes
the workflow applied (t13180-log-patch-stat, t5505-remote) plus refreshed data/tests
and regenerated dashboards.

Left untouched: ~820 mode-only (644→755) flips the harness applies to test files at
run time (persistent benign churn); stray build artifacts (build_err.txt,
git/GIT-BUILD-OPTIONS, tests/git); grit-lib/src/progress.rs (separate libification
branch).
