# t6430-merge-strategy-option.sh — work log

Ticket: 214b65. Subsystem group merge-ort (thread C). File: tests/t6430-merge-strategy-option.sh.

## TL;DR
This is a **grit-authored synthetic test** (upstream t6430 is `merge-recursive`, unrelated).
It is **factually wrong** and cannot pass without (a) editing the test file, which is forbidden,
or (b) making grit diverge from real Git, which violates the core mandate. Grit's actual
`merge -X ours/theirs` behavior is **byte-for-byte identical to upstream Git 2.52.0**.

## Two independent defects in the test file (not in grit)

### 1. Branch name: `master` vs `main` (breaks T1 setup and T4 setup)
- Setup does `git init repo` then `git checkout master`.
- The harness (scripts/run-tests.sh:376) forces `GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=main`,
  and test-lib.sh:322-325 writes `init.defaultBranch=main` into the global config.
- grit correctly honors `init.defaultBranch`, so `git init repo` makes branch `main`, and
  `git checkout master` fails with "pathspec 'master' did not match". Setup aborts.
- This is the classic test-porting bug; passing grit tests that need `master` use
  `git init --initial-branch=master repo` (see t6200). This file does not.
- Verified: with `init.defaultBranch=master` set manually, setup, T2, T3 all pass in grit.

### 2. T5 / T6 expect behavior NEITHER grit NOR real Git has (modify/delete conflict)
- T5: `git merge -X ours feature && ! test -f file.txt` — expects `-X ours` to keep the
  deletion (our side) in a delete/modify conflict.
- T6: `git merge -X theirs feature && test_cmp expect file.txt` — expects `-X theirs` to
  keep the modification, with merge exiting 0 (note the `&&`).
- **Upstream merge-ort.c (process_entry, filemask 3/5, lines ~4368-4416) does NOT consult
  `recursive_variant` (ours/theirs) for modify/delete.** It unconditionally keeps the
  modified version in the tree, marks the merge unclean, and prints
  "CONFLICT (modify/delete): ... Version <branch> of <path> left in tree." Merge exits 1.
- Verified against the system `/opt/homebrew/bin/git` (Git 2.52.0):
  - `-X ours` modify/delete  → CONFLICT, exit 1, file PRESENT with modified content.
  - `-X theirs` modify/delete → CONFLICT, exit 1, file PRESENT with modified content.
- grit produces the identical result (same conflict message, same exit 1, same index stages 1+3).
- Therefore T5's `! test -f file.txt` and T6's reliance on merge exit 0 are both impossible
  under correct Git semantics. The test is wrong, not grit.

## What grit does correctly (verified equal to real Git)
- T2 `merge -X ours feature`  → "ours line",   exit 0.  (matches git 2.52.0)
- T3 `merge -X theirs feature` → "theirs line", exit 0.  (matches git 2.52.0)
- modify/delete -X ours/theirs → CONFLICT, file left in tree. (matches git 2.52.0)

## Conclusion / recommendation
No grit code change is warranted — grit already replicates Git exactly. To make this file
pass it must be corrected at the test level:
  1. `git init --initial-branch=master repo` (or use `main` throughout), AND
  2. Rewrite/remove T5 and T6 so they match real Git's modify/delete behavior (the deletion
     is NOT auto-resolved by -X ours/theirs; the merge conflicts and the modified file stays).
Both are test-file edits outside this ticket's allowed scope (only expect_failure→expect_success
flips permitted, and none of these are expect_failure). Ticket left open with findings for a
human / test-owner to correct the synthetic test.

No Rust files changed.
