# t6430-merge-strategy-option.sh — MOP-UP ROUND 1 (ticket 214b65)

Date: 2026-06-07
Agent: schacon+claude-opus@gmail.com (grit-t5-progress)

## Status
Fresh run: 0/6. Investigated and found a REAL grit divergence from Git in
modify/delete + `-X ours/theirs` handling, fixed it. The test FILE itself
still cannot reach 6/6 (it is a grit-authored synthetic test that is
factually wrong against real Git on two independent axes).

## Findings

### Test is grit-authored & synthetic
Upstream `git/t/t6430` is `t6430-merge-recursive.sh`, NOT
`t6430-merge-strategy-option.sh`. The file in `tests/` was written by a
previous grit effort.

### Axis 1 — T1/T4 setup use `git checkout master` but harness forces `main`
`scripts/run-tests.sh:378` exports `GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=main`
and `tests/test-lib.sh:322-324` sets `init.defaultBranch=main` globally.
So `git init repo` creates branch `main`; the setup's final
`git checkout master` fails with `pathspec 'master' did not match`. Verified
this aborts T1 (and T4) and leaves cwd wrong → cascades all 6 subtests.
Real Git behaves identically under the same harness env. Cannot fix without
editing the test (forbidden).

### Axis 2 — T5/T6 expect `-X ours/theirs` to auto-resolve modify/delete (WRONG)
Real Git's merge-ort `process_entry` (merge-ort.c ~L4368-4415, filemask 3/5)
does NOT consult `opt->recursive_variant` for modify/delete. It always keeps
the modified version in tree and reports `CONFLICT (modify/delete)`, exit 1.
Verified byte-for-byte against system git 2.52.0 for both `-X ours` and
`-X theirs`. So T5 (expects file deleted, exit 0) and T6 (expects exit 0)
are both factually wrong.

## REAL grit bug found & fixed
Before this fix, grit's `git merge` `merge_trees` (grit/src/commands/merge.rs)
DID apply `MergeFavor::Ours/Theirs` to modify/delete conflicts:
- `-X ours` on delete/modify → removed file, exit 0  (WRONG)
- `-X theirs` on delete/modify → kept modified file, exit 0  (WRONG)

This was added in commit a1899cea4 specifically to satisfy this synthetic
test, but it diverges from real Git. Removed the favor short-circuits in both
modify/delete arms (`(Some,None,Some)` and `(Some,Some,None)`), so grit now
always reports CONFLICT(modify/delete) and exits 1 — matching git 2.52.0
exactly.

After fix, grit output == git 2.52.0 for both `-X ours` and `-X theirs`
modify/delete: `CONFLICT (modify/delete): file.txt ... left in tree`, exit 1,
file present with modified content.

## Regression check
- grit-lib unit tests: only the 2 known ignore failures.
- t6417-merge-ours-theirs 7/7, t4301 44/44, t6402 46/46, t6406 13/13,
  t7607 1/1, t3510 55/55, t6436 18/18, t6400 7/7 — all pass.
- t3507 / t6437 showed failures during the run; isolating whether these are
  from another agent's concurrent edit (shared binary) vs my change.

## Conclusion on ticket
The merge-fidelity bug is fixed (worth committing). The t6430 test file
remains red because it is a broken synthetic test on two axes the rules
forbid me to edit. Leaving ticket open/blocked with this finding.
