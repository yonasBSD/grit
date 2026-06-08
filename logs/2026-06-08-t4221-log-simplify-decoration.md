# t4221-log-simplify-decoration — work log (2026-06-08)

Ticket: 0cf4c1 (# t4221-log-simplify-decoration)
File: tests/t4221-log-simplify-decoration.sh — 2/4 passing.

## Conclusion: BLOCKED — test file has incorrect expectations (not a grit bug)

This is a grit-authored test (no upstream `git/t/t4221-*.sh` exists; the real
upstream port for this feature is t6012-rev-list-simplify.sh, which is fully
passing at 42/42). Two subtests assert behavior that real Git does NOT exhibit.

### Proof: grit matches Git 2.52.0 exactly

Repo: commits first→second→third(tag v1.0)→fourth→fifth(HEAD), all touching file.txt.

`git log --simplify-by-decoration --oneline` (Git 2.52.0):
    fifth
    third
    first        <-- ROOT commit IS shown by Git

`grit log --simplify-by-decoration --oneline`:
    fifth
    third
    first        <-- identical

Same for `--format=tformat:%s` (3 lines) and `--all --oneline` (fifth/third/first)
— grit and Git agree on every case.

### Why the test is wrong

Git's `--simplify-by-decoration` marks undecorated commits TREESAME and applies
normal history simplification. Per Documentation/rev-list-options.adoc the
simplification rules KEEP root commits (a rewritten commit that is a root or
merge is kept; rev-list-options.adoc ~line 590-593, 813-819). So the unreferenced
ROOT commit `first` is always shown.

Failing subtests with wrong assertions:
1. '--simplify-by-decoration shows only decorated commits' asserts `! grep "first"`.
   Git shows `first` → assertion is wrong.
2. '--simplify-by-decoration shows HEAD and tagged commits' asserts `wc -l == 2`.
   Git emits 3 lines → assertion is wrong.

grit's keep-set logic (grit-lib/src/rev_list.rs compute_simplify_by_decoration_keep_set,
~line 5202) correctly keeps roots (is_root branch, line 5246-5247).

### Action
Cannot make these pass without breaking grit to diverge from Git, or editing the
test assertions (forbidden by working contract: only allowed test edit is
test_expect_failure->test_expect_success). Marking ticket blocked per FIX
PHILOSOPHY (proven test bug, not grit bug). No grit code changed.
