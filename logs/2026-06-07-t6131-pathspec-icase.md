# t6131-pathspec-icase — MOP-UP ROUND 1 (ticket fd04f0)

Date: 2026-06-07T06:18 UTC
Agent: grit-t5

## Summary

grit is CORRECT for this file. The local harness records 1/9 ONLY because the
`tests/` directory lives on a case-insensitive APFS volume, where the test's
required case-distinct refs (`bar` / `bAr` / `BAR`) collide. Real `/usr/bin/git`
fails on this filesystem in exactly the same way. On a case-sensitive volume grit
is **9/9**.

## Investigation

1. Re-ran fresh: still 1/9. Failure is at subtest 1 ("create commits with glob
   characters"):

       error: tag 'bAr' already exists

   This happens because `test_commit bar` writes loose ref
   `.git/refs/tags/bar`, then `test_commit bAr` tries to write
   `.git/refs/tags/bAr` — same inode on a case-insensitive FS, so grit (correctly)
   reports the tag already exists. Every later subtest depends on those commits,
   so all 8 downstream subtests abort.

2. Confirmed `tests/` is on a case-insensitive filesystem:
   `echo good > CamelCaseProbe999; echo bad > camelcaseprobe999` → only one file,
   content "bad". So `test_have_prereq CASE_INSENSITIVE_FS` SHOULD be true here.

3. Proved real git fails identically on this exact FS:

       git init; git commit --allow-empty -m c1
       git tag bar        # ok
       git tag bAr        # fatal: tag 'bAr' already exists  (exit 128)

   git version 2.39.5 (Apple Git-154). So grit's behavior matches upstream git.

4. Proved grit is 9/9 on a case-sensitive volume. Created a case-sensitive APFS
   sparse image (`hdiutil create -fs "Case-sensitive APFS"`), pointed
   `TRASH_DIRECTORY` there, ran the file:

       ok 1..9 — # passed all 9 test(s)

   (Image detached/removed afterward; no residue left.)

## Why it cannot be recorded green here

The upstream test guards itself at the top:

    if test_have_prereq CASE_INSENSITIVE_FS
    then
        skip_all='skipping case sensitive tests - case insensitive file system'
        test_done
    fi

Upstream defines `CASE_INSENSITIVE_FS` as a `test_lazy_prereq` in
`git/t/test-lib.sh` (line 1771). grit's `tests/test-lib.sh` does NOT define that
lazy prereq, so the guard evaluates false and the test runs (and collides).

The lazy-prereq machinery in grit's `tests/test-lib.sh` (the `lazily_testable_prereq`
path, ~line 922) is fully capable of running it — it just needs the
`test_lazy_prereq CASE_INSENSITIVE_FS '...'` definition present. Adding it would
make grit skip on this host exactly like upstream git does on a case-insensitive
host, recording the file as passing/skipped.

But the agent rules forbid modifying `tests/test-lib.sh` ("Do NOT modify
tests/test-lib.sh") and forbid modifying test files except for
`test_expect_failure` -> `test_expect_success` flips (not applicable). So I cannot
add the prereq, and there is no grit Rust change that would help (grit already
matches real git).

## Conclusion / handoff

- No grit Rust bug remains. The prior agent's real fix (commit 35da7a746 in
  grit-lib/src/rev_list.rs — emit path-limited log ancestors when tips are
  dropped) is an ancestor of HEAD and is what makes the case-sensitive run green.
- To record this file green on the macOS harness, EITHER run the suite on a
  case-sensitive host/volume, OR (a test-harness owner decision, outside these
  agent rules) add `test_lazy_prereq CASE_INSENSITIVE_FS` to tests/test-lib.sh so
  the file self-skips like upstream.
- Leaving ticket fd04f0 OPEN: cannot be recorded fully-passing under the current
  rules + filesystem, despite grit being correct.
