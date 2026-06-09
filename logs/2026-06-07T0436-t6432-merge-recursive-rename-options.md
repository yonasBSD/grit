# t6432-merge-recursive-rename-options.sh — MOP-UP ROUND 1 (ticket d23035)

Date: 2026-06-07T04:36Z
Agent branch: grit-t5-progress

## Status: 1/3 passing (unchanged). BLOCKED — no grit Rust bug exists.

## Fresh re-run
`./scripts/run-tests.sh t6432-merge-recursive-rename-options.sh` → 1/3 (setup only).
Verbose TAP shows tests 2 & 3 abort with:
`./test-lib.sh: line 1413: cd: merge-opts: No such file or directory`
before any grit command runs.

## Root cause (confirmed, identical to prior agent's finding)
Documented cwd-persistence porting trap (TESTING.md §"Harness pitfall: cwd
persists across tests"). `test-lib.sh` `test_eval_inner_` (line ~1401) persists
cwd across top-level `test_expect_success` blocks by design — matching upstream
git/t. The adapted test file does `cd merge-opts` in ALL THREE blocks:
- setup (test 1): `git init merge-opts && cd merge-opts && …` — leaves shell INSIDE merge-opts/
- test 2: `cd merge-opts && …` — runs from inside merge-opts/, no merge-opts/merge-opts → cd fails → block aborts
- test 3: same as test 2.

This file is a hand-adaptation (header: "Adapted from
git/t/t6434-merge-recursive-rename-options.sh"); upstream t6434 has an entirely
different structure and does not exhibit this. Ported in batch commit d6e470dc5.

## Grit behavior is correct (verified manually)
Ran the exact test-2 and test-3 command sequences in a scratch dir with the
correct cwd using tests/grit:
- TEST 2 (merge non-overlapping): ort merge succeeds, file1 has "A change",
  file2 has "B change" → PASS.
- TEST 3 (merge --no-commit): HEAD == modify-A, file2 has "B change" → PASS.
grit's merge (ort), reset --hard, rev-parse all behave correctly.

## Why not fixed
The ONLY remediation is wrapping each test body in a subshell `( cd merge-opts && … )`
— TESTING.md sanctions this via `scripts/_wrap_cd_subshell.py` and the script
exists. BUT this is a test-file content edit, forbidden by the orchestrator hard
rule for this run ("Do NOT modify test files — the ONLY allowed test edit is
flipping test_expect_failure -> test_expect_success"). There is NO grit Rust
change that can make a bare `cd merge-opts` succeed from inside merge-opts/ —
`cd` is a POSIX shell builtin. Prior agent (comment 1 on d23035) reached the
same conclusion; auto-mode classifier denied the test overwrite.

## Recommendation
Needs a human / an agent authorized to apply the subshell wrap:
`python3 scripts/_wrap_cd_subshell.py tests/t6432-merge-recursive-rename-options.sh`
(prior agent reports the wrapped file passes 3/3). Then re-run and flip nothing
else. No grit source change required.
