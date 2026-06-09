# t6432-merge-recursive-rename-options — MOP-UP ROUND 2 (2026-06-07T08:17)

Ticket: d23035. Commit at run: 7f82886a9 (HEAD).

## Result: still 1/3 passing, UNCHANGED. Remains BLOCKED. No grit Rust fix exists.

## Fresh re-run
- Built `cargo build --release -p grit-cli -j 4` (clean).
- `./scripts/run-tests.sh t6432-merge-recursive-rename-options.sh` -> 1/3.
- Verbose (`sh ./t6432-... -v -i`): test 1 (setup) OK; tests 2 & 3 abort at
  `./test-lib.sh: line 1413: cd: merge-opts: No such file or directory`
  BEFORE any grit command runs.

## Root cause (re-confirmed, identical to prior 2 agents)
Documented cwd-persistence porting trap (TESTING.md "Harness pitfall"):
- The setup block runs `git init merge-opts && cd merge-opts && ...` and never
  cd's back to the trash root.
- test-lib.sh `test_eval_inner_` (lines 1401-1422) intentionally does NOT reset
  cwd between top-level `test_expect_success` blocks — this matches upstream
  git/t behavior (the comment at line 1403 says so explicitly).
- So after setup, cwd is `.../trash.../merge-opts`. Tests 2 & 3 each begin with
  `cd merge-opts`, which fails because there is no nested `merge-opts/`.
- Traced directly: after the setup chain, `pwd` =
  `.../trash.t6432.../merge-opts`; a subsequent `cd merge-opts` errors
  "no such file or directory". Identical to what upstream Git would do.

## grit is correct (verified by emulating both bodies with proper cwd)
Ran the full setup + test-2 + test-3 command sequences with `tests/grit` in a
scratch trash dir, supplying the correct cwd:
- Test 2: `git merge modify-B -m "merge B"` -> "Merge made by the 'ort' strategy",
  file1 contains "A change", file2 contains "B change". PASS.
- Test 3: `git reset --hard modify-A` + `git merge --no-commit modify-B` ->
  HEAD == modify-A (rev-parse equal), file2 contains "B change". PASS.
All grit behaviors (ort merge, --no-commit, reset --hard, rev-parse) are right.

## Why this is not a faithful upstream port
Header says "Adapted from git/t/t6434-merge-recursive-rename-options.sh", but
upstream t6434 is an entirely different and far more complex rename-detection /
threshold test. This t6432 is a custom hand-written simplification introduced in
batch port commit d6e470dc5, and the hand adaptation carries the cwd bug.

## Why BLOCKED (no action taken)
The only possible fix is a test-content edit (subshell-wrap the 3 blocks, or drop
the redundant `cd merge-opts` in tests 2 & 3). That is forbidden by this run's
hard rule — the ONLY allowed test edit is flipping test_expect_failure ->
test_expect_success for a fixed bug, which does not apply here (all three are
already test_expect_success and there is no Rust bug to fix). `cd` is a shell
builtin, so there is no grit Rust change that can make tests 2 & 3 succeed.

Left ticket open + blocked. No commit made (no files changed by me).
