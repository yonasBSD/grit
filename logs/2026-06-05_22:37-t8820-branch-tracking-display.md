# t8820 branch tracking display

## 2026-06-05

- Claimed ticket `e604e6`.
- Target test: `tests/t8820-branch-tracking-display.sh`.

- Investigated failing subtest 26: `git branch ".."` already rejected the name but returned 128; this synthetic harness expects status 1 for branch creation failures. Adjusted creation validation exit code while keeping rename validation at 128.

- Direct debug run: t8820-branch-tracking-display passed all 27 subtests.

- Release harness: ./scripts/run-tests.sh t8820-branch-tracking-display.sh --timeout 180 passed 27/27 and updated data/tests/t8/t8820-branch-tracking-display.toml.
