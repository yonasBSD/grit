# t8950-show-ref-patterns — mop-up round 1 (ticket c3d261)

Date: 2026-06-07T04:58Z
Status: 27/29 passing. Remaining 2 are NOT grit defects — unfixable without forbidden test-body edits.

## Fresh run
`./scripts/run-tests.sh t8950-show-ref-patterns.sh` → `✗ t8950-show-ref-patterns (27/29)`.
No cascade from other agents changed the count.

## Failing subtests
- 19: `show-ref --hash shows only SHAs`
- 21: `show-ref --tags --hash lists only SHAs for tags`

## Root cause (independently re-verified)
Both failures are a macOS/BSD `wc -c` portability bug in the **synthetic** test body, not a grit defect.

Subtest 19 body:
```
sha=$(tr -d "\n" <actual) &&
len=$(printf "%s" "$sha" | wc -c) &&
test "$len" = 40
```
On macOS, BSD `wc -c` emits leading whitespace, so `len="      40"` (verified: `len=[      40]`).
The QUOTED compare `test "      40" = 40` therefore fails. On Linux, GNU `wc -c` outputs `40`, so it passes.
Subtest 21 has the identical `test "$len" = 40` pattern inside its while-loop.

Contrast: subtests that pass on BSD (e.g. #4 "each line has SHA", #28 "--abbrev") use either the
unquoted `test $(...| wc -c) = 40` form (word-splitting drops the leading whitespace) or a numeric
context — both tolerate BSD's leading whitespace.

## grit output is byte-perfect
`grit show-ref --hash refs/heads/master` → `xxd` shows exactly 40 hex chars + a single `0a` (`\n`).
`/usr/bin/git show-ref --hash refs/heads/master` produces the byte-for-byte identical output, so
canonical git would FAIL this same synthetic test on macOS too. No upstream `git/t/t8950*` exists
(fully synthetic test, no C ground truth to diverge from).

## Why no fix was applied
- grit Rust output is already identical to canonical git — there is nothing to fix in grit-lib/grit-cli.
- The only mechanical fix is editing the test body (`test "$len" = 40` → `test "$len" -eq 40`,
  numeric `-eq` tolerates BSD leading whitespace). Test-body edits are forbidden by the ticket rules
  (only `test_expect_failure` → `test_expect_success` flips are allowed, and these are already
  `test_expect_success`).

## Precedent (same BSD `wc` class, also left unfixed grit-side)
- 3b988355b — t9190 #20: BSD wc leading-whitespace; grit output correct; log-only commit.
- e39408ae8 — t8070: macOS wc/test_line_count harness bug not grit; log-only commit.

## Outcome
Cannot reach 29/29 on macOS without a forbidden test-body edit. Ticket left open/blocked with
this verification. grit behavior is correct.
