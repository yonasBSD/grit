# t6131-pathspec-icase — MOP-UP ROUND 2 re-verify (2026-06-07T10:12, agent t5)

Ticket: fd04f0. Re-ran fresh after rebuild: still **1/9**, dies at subtest 1.

## Conclusion: grit is CORRECT. Blocker is the test harness, not grit code.

The upstream test is **designed to self-skip** on case-insensitive filesystems
(git/t/t6131-pathspec-icase.sh:7-11, `test_have_prereq CASE_INSENSITIVE_FS`).
grit's `tests/test-lib.sh` does not define the `CASE_INSENSITIVE_FS` lazy prereq
(it lives in git/t/test-lib.sh:1771). Agent rules forbid modifying
`tests/test-lib.sh` and test files, so the file cannot self-skip here and instead
fails at subtest 1.

## Evidence gathered this round

1. **FS is case-insensitive**: `touch CamelCaseProbe999` then `[ -e camelcaseprobe999 ]`
   returns true in both `/tmp` and the repo `tests/` dir.

2. **Subtest 1 failure cause**: `test_commit bAr` runs `git tag bAr` after
   `git tag bar` already exists. On a case-insensitive FS loose refs
   `.git/refs/tags/{bar,bAr}` are the same file →
   grit: `error: tag 'bAr' already exists` (exit 1).

3. **Real /usr/bin/git 2.39.5 fails IDENTICALLY** on this same FS:
   `git tag bar` ok (exit 0), `git tag bAr` → `fatal: tag 'bAr' already exists`
   (exit 128). The working tree even shows `modified: bar` after writing `bAr`,
   proving `bar`/`bAr` collapse to one inode. grit matches upstream behavior.

4. **The `:(icase)` feature itself works in grit** (tested in a single-case dir to
   avoid the FS collision): `git ls-files ":(icase)d/bar"` → `d/Bar`;
   plain `git ls-files "d/bar"` → nothing. Feature under test is implemented.

5. **Prior grit fix is present in history**: 35da7a746 (rev_list.rs
   `date_order_walk_through_dropped`, the dense path-limiting ancestor-emit fix)
   and 41a9786ec are both ancestors of HEAD. grit-lib/src/rev_list.rs:1097-1108
   confirms the through-dropped walk is wired in. This is what makes the file 9/9
   on a case-sensitive host (proven by prior agent via a case-sensitive APFS image).

## What I did NOT do
No grit Rust change — grit is already correct and there is no code change that
helps on a case-insensitive host. Did not modify tests/test-lib.sh or the test
file (forbidden). Did not touch other agents' dirty files.

## Recommendation
Leave ticket OPEN. To record green, either (a) run on a case-sensitive host, or
(b) a harness owner adds the `CASE_INSENSITIVE_FS` lazy prereq to
`tests/test-lib.sh` (mirroring git/t/test-lib.sh:1771) so this file self-skips.
