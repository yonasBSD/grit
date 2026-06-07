# t6435-merge-sparse-directory — mop-up round 1 (2026-06-07)

Ticket: 857608

## Result: 1/2 passing (unchanged). NOT a grit bug.

## Diagnosis (confirms prior agent + adds upstream comparison)

Fresh build (`cargo build --release -p grit-cli -j4`) + fresh run: still 1/2.

Verbose run of subtest 2 shows the failure is in the harness, before any grit
command executes:

```
expecting success of 6435.2 'merge brings in both directories':
	cd merge-dirs &&
	...
./test-lib.sh: line 1413: cd: merge-dirs: No such file or directory
not ok 2 - merge brings in both directories
```

This is the **documented cwd-leak harness pitfall** (TESTING.md "Harness pitfall:
cwd persists across tests"):

- Setup (test 1) does `git init merge-dirs && cd merge-dirs && ...` and never
  cd's back out, so the shell is left **inside** `merge-dirs/`.
- Test 2 begins with a bare `cd merge-dirs`, which runs *before* test-lib resets
  cwd (top-level test_eval_inner_ deliberately persists cwd across blocks to match
  upstream git/t — see test-lib.sh:1400-1419). So `cd merge-dirs` from inside
  `merge-dirs/` fails and the block aborts before `git merge` runs.

## grit merge is correct (re-verified manually)

Reproduced the exact test-2 sequence in /tmp with a fresh release binary:
`git merge sideB -m "merge sideB"` -> "Merge made by the 'ort' strategy",
and both `dirA/file` and `dirB/file` exist afterward. Merge works perfectly.

## Upstream comparison

`git/t/t6435-merge-sparse.sh` is an ENTIRELY DIFFERENT test (sparse-checkout
modify/delete conflicts). The grit port `tests/t6435-merge-sparse-directory.sh`
is a custom-written, simplified test that introduced the `cd merge-dirs` pattern.
Upstream never uses `cd repo` here, so upstream never hits this leak. The bug is
purely in how the grit port was authored.

## Why this cannot be fixed under the current rules

- Sanctioned fix per TESTING.md is `scripts/_wrap_cd_subshell.py` (wrap each body
  in a subshell). But the ticket/commit-contract hard rule forbids any test-file
  edit except `test_expect_failure -> test_expect_success`.
- The `test_expect_failure -> success` flip does not apply: subtest 2 is already
  `test_expect_success`. The test genuinely fails on the harness pitfall, not on
  grit capability.
- `test-lib.sh` modification is also forbidden, and its cwd-persist behavior is
  intentional/correct (matches upstream).

No grit Rust change exists that can make this pass. Leaving 1/2; ticket stays open.
The only path to 2/2 is the sanctioned subshell wrap of the test file by an
owner permitted to edit test bodies.
