# t6432-merge-recursive-rename-options — work log

Ticket: d23035
Date: 2026-06-06T14:32Z
Final harness status: 1/3 (unchanged — see below)

## Summary

The 2 failing subtests (`merge with non-overlapping changes`, `merge --no-commit
stages but does not commit`) are NOT a grit bug. grit's merge behavior is fully
correct for both. The failures are caused entirely by the documented harness
cwd-persistence porting trap (TESTING.md "Harness pitfall: cwd persists across
tests").

## Root cause

`tests/t6432-merge-recursive-rename-options.sh` was ported from upstream
`git/t/t6434-merge-recursive-rename-options.sh` but uses the broken pattern:

- `setup` does `git init merge-opts && cd merge-opts && ...` and never returns to
  the trash root, so the shell is left inside `merge-opts/`.
- test 2 and test 3 each begin with a bare `cd merge-opts`. Because `test-lib.sh`
  persists cwd across top-level `test_expect_success` blocks (intentional, matches
  upstream git/t — see `test_eval_inner_` at tests/test-lib.sh:1401), that
  `cd merge-opts` runs from *inside* `merge-opts/`, fails ("No such file or
  directory"), and the whole block aborts before any grit command runs.

## Proof grit is correct

Ran the full sequence manually (and again emulating the harness with each body
wrapped in a subshell `( ... )` rooted at the trash dir):

- TEST 1 setup: PASS
- TEST 2 (checkout modify-A; merge modify-B -m; grep A change/B change): PASS
- TEST 3 (reset --hard modify-A; merge --no-commit modify-B; HEAD==modify-A;
  grep B change in file2): PASS — "Already up to date." is correct here because
  test 2 already merged modify-B's content into modify-A, and test 3 runs on the
  modify-A branch after test 2.

All three pass once the cwd leak is contained. grit's ort merge, --no-commit
staging, reset --hard, and rev-parse all behave correctly.

## Fix (blocked)

The TESTING.md-sanctioned cure is to subshell-wrap each test body via
`scripts/_wrap_cd_subshell.py tests/t6432-merge-recursive-rename-options.sh`
(wraps 3 blocks; verified the wrapped file passes 3/3 by emulating the harness).

This is a TEST FILE edit. The auto-mode classifier and the orchestrator's hard
rule ("Do NOT modify test files — the ONLY allowed test edit is flipping
test_expect_failure -> test_expect_success") block it. There is NO Rust change to
make — grit is already correct. Left for a human/mop-up agent authorized to apply
the subshell wrap.

## Verification commands used

```
# manual sequential run (no cd leak): all assertions pass
# harness emulation with ( cd TRASH && ( <body> ) ) per block: T1/T2/T3 all PASS
python3 scripts/_wrap_cd_subshell.py <copy of test>   # -> wrapped 3 blocks
```
