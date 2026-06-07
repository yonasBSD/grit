# t6007-rev-list-cherry-pick-status.sh — mop-up investigation

Date: 2026-06-07
Agent: schacon+claude-t5
Ticket: 104846

## Starting state
28/29 passing. One failing subtest.

## Failing subtest (test 7)
`'rev-list --left-right count with --count'`
```
git rev-list --left-right --count left...right >actual &&
echo "2\t2\t0" >expect &&
test_cmp expect actual
```
The test expects THREE tab-separated columns: `2\t2\t0`.

grit produces TWO columns: `2\t2`.

## Diagnosis: incorrect test expectation, NOT a grit bug

Upstream `git/builtin/rev-list.c` (lines 956-963) defines the `--count` output format:
```c
if (revs.left_right && revs.cherry_mark)
    printf("%d\t%d\t%d\n", count_left, count_right, count_same);   // 3 cols
else if (revs.left_right)
    printf("%d\t%d\n", count_left, count_right);                   // 2 cols
else if (revs.cherry_mark)
    printf("%d\t%d\n", count_left + count_right, count_same);
else
    printf("%d\n", count_left + count_right);
```
The third column (`count_same`, the equivalent/cherry count) only appears when
`--cherry-mark` is ALSO given. The failing test passes only `--left-right --count`
(no `--cherry-mark`), so the correct output is two columns.

### Verification against real git 2.39.5
```
$ git rev-list --left-right --count left...right
2	2          # two columns — matches grit
```

### grit code is already correct
`grit/src/commands/rev_list.rs:1160-1200` mirrors upstream exactly:
- `left_right && cherry_mark` -> three columns
- `left_right` only -> two columns
- `cherry_mark` only -> different\tequivalent
- else -> single count

### Other tests constrain the correct (2-column) behavior
- `tests/t6007-rev-list-cherry-pick-file.sh` (the upstream-faithful sibling port) test 21
  `'--count --left-right'` expects `1\t2` (two columns). File is FULLY PASSING 23/23.
  Its test 19 `'--cherry-mark --left-right --count'` expects three columns and passes.
- `tests/t5310-pack-bitmaps.sh` compares plain `--left-right --count` output between
  bitmap and non-bitmap index — also assumes two columns.

Changing grit to emit three columns for plain `--left-right --count` would regress
t6007-rev-list-cherry-pick-file test 21 (and possibly t5310), breaking grit's
upstream-correct behavior to satisfy one synthetic test.

## Conclusion
This is a test-authoring discrepancy in the synthetic `*-status.sh` variant, not a grit
bug. The only ways to make it pass are (a) edit the test expectation (disallowed by the
contract) or (b) make grit diverge from upstream git and regress other passing tests
(unacceptable). Marking the ticket BLOCKED with this rationale. No grit code change made.

## Round 2 re-verification (2026-06-07T08:36)
Re-ran fresh after potential cascades from other agents: still 28/29 (only test 7 fails).
Re-verified against the real git binary on this machine (git version 2.52.0), not just the
2.39.5 cited above:
```
$ git rev-list --left-right --count left...right          => 2<TAB>2      (2 cols)
$ git rev-list --left-right --count --cherry-mark left...right => 2<TAB>2<TAB>0 (3 cols)
```
grit produces byte-identical output for both invocations:
```
$ grit rev-list --left-right --count left...right          => 2<TAB>2
$ grit rev-list --left-right --count --cherry-mark left...right => 2<TAB>2<TAB>0
```
Confirms the prior diagnosis under a newer real-git version. Test 7 demands `2\t2\t0` from
`--left-right --count` WITHOUT `--cherry-mark`, which neither real git 2.52.0 nor grit emit.
Re-read C source `git/builtin/rev-list.c:955-963` (3-col branch is `left_right && cherry_mark`).
No grit change possible without regressing the upstream-faithful sibling
`t6007-rev-list-cherry-pick-file.sh` test 21 (`1\t2`, 2 cols) and t5310. Remains BLOCKED.
