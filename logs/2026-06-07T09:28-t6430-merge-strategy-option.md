# t6430-merge-strategy-option.sh — mop-up round 2 re-verification

Ticket: 214b65. Date: 2026-06-07T09:28Z.

## Fresh run result

`./scripts/run-tests.sh t6430-merge-strategy-option.sh` → **0/6** (unchanged).

Re-ran fresh in case other agents' fixes cascaded; no change. Prior commit
14dcfdadb (remove MergeFavor short-circuits in `git merge`'s `merge_trees`
modify/delete arms) is present in workspace history and remains correct.

## Re-verified diagnosis — two independent blockers, both TEST bugs

This is a grit-AUTHORED synthetic test (upstream t6430 is merge-recursive). Both
blockers were re-verified byte-for-byte against system git **2.52.0** this run.

### axis1 — T1/T4 setup: `git checkout master` cannot succeed

The harness (`scripts/run-tests.sh:378` + `tests/test-lib.sh:322-324`) forces
`GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=main` → `git config --global
init.defaultBranch main`. So `git init repo` creates default branch **main**,
not master. The test's final setup command `git checkout master` therefore fails:

```
error: pathspec 'master' did not match any file(s) known to git   (exit 1)
```

Verified identical in **grit** and **git 2.52.0**. Because T1 ends on this failing
command, T1 fails AND the worktree is left on `feature`, so every later test
starts from the wrong HEAD and cascades to failure.

Proof that the content-conflict logic is correct: re-ran the T1-T3 scenario
substituting `main` for `master` with the grit binary →
- T2 `merge -X ours feature`  → exit 0, file.txt == "ours line"  ✓
- T3 `reset --hard HEAD~1; merge -X theirs feature` → exit 0, file.txt == "theirs line"  ✓

So grit's `-X ours/-X theirs` content-conflict resolution is already correct; the
only reason T2/T3 fail in the harness is the `master` branch never existing.

### axis2 — T5/T6 expect `-X` to auto-resolve modify/delete (factually wrong)

T5 expects `! test -f file.txt` after `merge -X ours feature` on a modify/delete
conflict. Real Git does NOT consult the recursive variant (ours/theirs) for
modify/delete — merge-ort `process_entry` (filemask 3/5, merge-ort.c
~L4368-4415) always leaves the modified file in tree and reports
CONFLICT(modify/delete), exit 1. Re-verified byte-for-byte this run:

grit and git 2.52.0 BOTH emit:
```
CONFLICT (modify/delete): file.txt deleted in HEAD and modified in feature.  Version feature of file.txt left in tree.
Automatic merge failed; fix conflicts and then commit the result.
exit=1   file.txt present
```

So T5's `! test -f file.txt` and T6's "file == modified after exit-0 merge" are
both impossible under real Git semantics.

## Conclusion

grit matches git 2.52.0 byte-for-byte on every behavior this test exercises. The
0/6 is caused entirely by two bugs in the synthetic test file:
1. hardcoded `git checkout master` vs harness-forced `init.defaultBranch=main`;
2. T5/T6 asserting modify/delete auto-resolution that real Git never performs.

Fixing either requires editing the forbidden test (none are `expect_failure`).
No grit Rust change is warranted; making grit pass would require diverging from
Git, which is also forbidden. Maintainer action needed: make T1/T4 use
`--initial-branch=master` (or `git checkout -b master`/`main`), and rewrite or
drop T5/T6 to match real modify/delete semantics.

## Regression check (this run, clean release binary)

- grit-lib --lib: only the 2 known `ignore::gitignore_glob_tests` failures.
- clippy: 0 warnings in merge.rs.
- t6417-merge-ours-theirs 7/7, t6402-merge-rename 46/46, t6436-merge-overwrite 18/18 — all green.

Leaving ticket open/blocked: needs test-owner fix, not a grit fix.
