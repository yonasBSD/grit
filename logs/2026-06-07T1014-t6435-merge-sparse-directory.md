# t6435-merge-sparse-directory — mop-up round 2 (2026-06-07T10:14)

Ticket: 857608

## Result
1/2 passing (unchanged). NOT a grit bug. No grit Rust change can make subtest 2 pass.

## Fresh re-run
`./scripts/run-tests.sh t6435-merge-sparse-directory.sh` -> `✗ t6435-merge-sparse-directory (1/2)`.
Other agents' fixes did not cascade to this file.

## Root cause (third independent confirmation)
Documented cwd-leak harness pitfall (TESTING.md "Harness pitfall: cwd persists across tests").

Verbose run shows subtest 2 dies BEFORE any grit command:
```
./test-lib.sh: line 1413: cd: merge-dirs: No such file or directory
not ok 2 - merge brings in both directories
```
- Setup (subtest 1) does `cd merge-dirs` and never cd's back out, leaving the shell inside `merge-dirs/`.
- test-lib.sh:1405-1411 intentionally PERSISTS cwd across top-level `test_expect_success` blocks (matches upstream git/t; only nested lib-subtest.sh scripts reset cwd). This is correct and must not be changed.
- Subtest 2's bare `cd merge-dirs` then runs from inside `merge-dirs/`, looking for `merge-dirs/merge-dirs` -> fails, aborting the block before `git merge` runs.

## grit merge verified correct
Reproduced the exact subtest-2 sequence with a fresh release binary from a clean cwd:
- `git merge sideB` -> "Merge made by the 'ort' strategy", exit 0
- `dirA/file` PRESENT, `dirB/file` PRESENT

So the merge logic grit implements is fully correct; the failure is purely the harness `cd`.

## Why it cannot be fixed under current rules
- Sanctioned fix is `scripts/_wrap_cd_subshell.py` (wrap subtest body in a subshell so the `cd` cannot leak). That edits the test file body.
- Hard rule forbids ALL test-file edits except flipping `test_expect_failure` -> `test_expect_success`. Subtest 2 is already `test_expect_success`, so no permitted flip applies.
- test-lib.sh edits are forbidden and its cwd-persistence behavior is correct/upstream-matching.
- No grit-side change can make this pass because the failure precedes any grit invocation.

The grit's t6435-merge-sparse-directory.sh is a CUSTOM port (upstream t6435 is about sparse-checkout modify/delete and never `cd repo`); the cwd-leak bug was introduced by the port authoring its setup with a bare `cd merge-dirs`.

## Disposition
Leaving open. Only path to 2/2 is an owner-sanctioned subshell wrap of the test body (out of scope for this agent's rules).
