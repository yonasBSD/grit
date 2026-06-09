# t8270-log-author-search

Ticket: 6c0a2e6f-424a-49ae-b791-61ad3b134924
Date: 2026-06-06

## Status

First run: 28/29 passing. Final run: 28/29 (unchanged).
Only failing subtest: 13 - "log --author is case-insensitive by default".

## Investigation

Subtest 13 asserts:

    git log --author=alice  --format="%an" >actual_lower
    git log --author=ALICE  --format="%an" >actual_upper
    test_cmp actual_lower actual_upper

i.e. it expects `--author` to be case-INsensitive by default.

### This contradicts upstream Git.

- git/Documentation/rev-list-options.adoc documents `-i`/`--regexp-ignore-case`
  as the way to match `--author`/`--committer`/`--grep` without regard to case;
  the default is case-sensitive.
- Empirically verified with the system's real `git version 2.52.0`:
  `git log --author=alice` matches (via email `alice@...`), but
  `git log --author=ALICE` matches nothing. grit produces identical output.
- grit's behavior already matches real Git exactly (case-sensitive default).

### Canonical upstream test depends on the case-sensitive default.

tests/t4209-log-pickaxe.sh (48/48 passing) explicitly asserts:

    test_log       expect_second  --author Person   # matches
    test_log       expect_nomatch --author person   # NO match without -i
    test_log_icase expect_second  --author person   # matches WITH -i

So the correct default is case-sensitive, which grit implements.

### History of the conflict.

- fb0b75e1b ("fix: make t8270 t8280 log search pass") forced `--author`/`--committer`
  regexes to `case_insensitive(true)` unconditionally, just to satisfy this
  synthetic t8270 assertion. That made t8270/13 pass but diverged from Git.
- ed5832390 ("fix(log): case-sensitive --author/--committer by default") correctly
  reverted that to honor `args.regexp_ignore_case`, restoring Git conformance and
  keeping t4209 green — at the cost of re-breaking t8270/13.

## Conclusion

t8270 subtest 13 is an INVALID test assertion (contradicts upstream Git and the
canonical t4209 conformance test). It cannot be fixed in grit without reverting
ed5832390, which would regress the real upstream test t4209-log-pickaxe (48/48).

Decision: keep grit conformant with upstream Git (case-sensitive `--author` by
default). No grit code change. The test logic may not be modified (only
test_expect_failure -> test_expect_success flips are allowed, and this is already
test_expect_success). Left the ticket open with this finding for the mop-up agent.

The 1 remaining failure is a bad test, not a grit bug.
