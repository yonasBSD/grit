# t6434-merge-with-no-common-ancestor — test-authoring cwd-leak fix

Date: 2026-06-08 (UTC)
Agent: schacon+claude-opus-t5
Ticket: 1e3f8b
Result: 3/3 passing (was 1/3). Classification: test-bug-fixed.

## Root cause (cwd-persistence harness trap, NOT a grit bug)

Each of the three subtests does `cd ancestor-test` directly (no subshell). The setup
block (test 1) does `git init ancestor-test && cd ancestor-test && ...`, leaving the
harness shell inside `ancestor-test/`. The next block's bare `cd ancestor-test` then
fails (`test-lib.sh:1417: cd: ancestor-test: No such file or directory`) and aborts the
subtest BEFORE any grit command runs. Removing only test 1's leak exposed the same leak
in test 2 (which then ran but left the shell inside ancestor-test/), breaking test 3.

## Differential verification (grit vs /opt/homebrew/bin/git 2.52.0)

Reproduced the exact command sequence in /tmp from the correct directory with BOTH the
harness `git` (target/release/grit) and real git 2.52.0:

- `git checkout left && git merge right -m "merge right"` -> exit 0, ort strategy,
  base + left-file + right-file all present (satisfies test 2's three test_path_is_file).
- `git merge-base left-tip right-tip` == `git rev-parse base` byte-identical within each
  binary (satisfies test 3's test_cmp). OIDs differ between grit and real git only due to
  commit-timestamp/tick differences; the *assertion* merge-base==base holds in both.

grit MATCHES real git => TEST-AUTHORING bug. This file is a synthetic port (no upstream
original); it was authored with the cd-leak bug. Four prior agents diagnosed this same
root cause but were blocked by a no-test-edit hard rule; this task explicitly authorizes
the test-file edit.

## Fix (sanctioned cwd-persistence pattern, hand-applied — NOT _wrap_cd_subshell.py)

Wrapped the body of all three `cd ancestor-test` blocks in a subshell `( ... )` so cwd
never leaks between subtests. No expected values changed; only the comparison/cd mechanism.

## Verification

`./scripts/run-tests.sh t6434-merge-with-no-common-ancestor.sh` -> 3/3 (fully_passing=true).
