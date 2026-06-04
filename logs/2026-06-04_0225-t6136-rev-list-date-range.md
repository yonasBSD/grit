# t6136 rev-list date range

Claimed `t6136-rev-list-date-range.sh` from `t6-plan.md` at 24/31 passing.

Initial focus:

- Run the current harness to identify the date-range failures.
- Read the local/upstream test and revision documentation around date cutoffs.
- Search existing rev-list date parsing and filtering before changing traversal behavior.

Findings:

- There is no upstream `git/t/t6136-rev-list-date-range.sh`; this is a synthetic local fixture.
- The first failing subtest checked out `master`, while the harness forces new repositories to
  start on `main`.
- Running the fixture directly with `GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=master` passed all 31
  subtests, so the failures were fixture setup fallout rather than rev-list behavior.

Changes:

- Exported `GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=master` before sourcing `test-lib.sh`, matching
  the branch name hard-coded by this synthetic fixture.

Validation:

- `./scripts/run-tests.sh t6136-rev-list-date-range.sh --verbose` passes 31/31.
- `cargo fmt` completed; unrelated fmt churn was restored.
- `cargo check -p grit-cli` completed with the existing warning backlog.
- `cargo test -p grit-lib --lib` passed 238/238.
- `cargo clippy --fix --allow-dirty` completed with the existing warning backlog; unrelated
  clippy auto-fixes were restored.
