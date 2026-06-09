# t6434-merge-with-no-common-ancestor — work log (MOP-UP ROUND 1)

Date: 2026-06-07T06:41 UTC
Ticket: 1e3f8b
File: tests/t6434-merge-with-no-common-ancestor.sh
Result: 1/3 passing (no grit change possible). NOT a grit bug.

## Fresh re-run (after other agents' fixes)

`./scripts/run-tests.sh t6434-merge-with-no-common-ancestor.sh` -> 1/3 (still 2 failing).
No cascade from sibling fixes changed this file.

Failing subtests:
- 2: merge diverged branches succeeds without conflicts
- 3: merge-base of diverged branches is base

Both abort identically BEFORE any grit command runs:
    ./test-lib.sh: line 1413: cd: ancestor-test: No such file or directory

## Root cause: cwd-persistence harness pitfall (TESTING.md §"Harness pitfall")

This file does NOT exist in upstream git/t/ — it is a ported/synthetic test
(introduced by commit d6e470dc5) authored with the cd-leak bug.

test-lib.sh persists cwd across top-level test blocks (matching upstream git/t).
Test 1 (setup) runs `git init ancestor-test && cd ancestor-test && ...` WITHOUT a
subshell, leaving the shell INSIDE ancestor-test/ after setup. Tests 2 and 3 each
begin with a bare `cd ancestor-test`, which runs while still inside ancestor-test/,
so the cd fails and the block aborts before any grit command runs.

## grit behavior is fully correct (re-verified manually in the trash dir)

In tests/trash.t6434-.../ancestor-test, using target/release/grit:
- `grit checkout left`                 -> "Switched to branch 'left'"
- `grit merge right -m "merge right"`   -> succeeds; base, left-file, right-file
  all present; exit 0.
- `grit merge-base left-tip right-tip`  -> 8d55173aaa417d42562d0d66b6a3b35da44f6136
- `grit rev-parse base`                 -> 8d55173aaa417d42562d0d66b6a3b35da44f6136
  (merge-base == base — exactly what test 3 asserts)

If tests 2 and 3 reached their grit commands they would PASS.

## Disposition: docs-only, consistent with sibling cwd-trap tickets

The only fix is wrapping the cd-using test bodies in a subshell via
`scripts/_wrap_cd_subshell.py` — a TEST-FILE edit. My ticket hard-rule forbids
test-file edits (only allowed: test_expect_failure -> test_expect_success), and the
auto-mode classifier denied the wrap for the prior two agents.

Established precedent in THIS effort is docs-only for the identical pattern:
- 98a5997f5  docs: t6434 cwd-trap root cause (this file, prior agent)
- fd8afd7d9  docs: t6432 cwd-trap root cause
- c9c4dcb6a / fc7254e8b  docs: t6435 cwd-trap, grit correct

The wrap is also independently risky: commits fe250ebf7 / 0994a90a8 record that a
spurious subshell wrap broke t5526 by trapping test_when_finished. So wrapping is
not a safe blanket fix.

Conclusion: no grit Rust change can make tests 2 & 3 pass; grit is already correct.
Ticket left OPEN for a human or a mop-up agent explicitly permitted to apply the
test-file subshell wrapper.
