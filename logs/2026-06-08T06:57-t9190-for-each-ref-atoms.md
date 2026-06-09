# t9190-for-each-ref-atoms — test-body fix

Date: 2026-06-08T06:57Z
Ticket: c62a5a
Result: 27/27 passing (was 26/27).

## Failing subtest

- 20: `--count larger than ref count returns all`

## Differential verdict: TEST BUG (BSD wc padding)

Reproduced in `/tmp/t9190scratch/repo` with both grit (harness `git` =
`target/release/grit`) and real git 2.52.0 (`/opt/homebrew/bin/git`) on
identical inputs.

grit output is byte-for-byte identical to real git for both branches:

- `for-each-ref --format="%(refname)"` -> 5 refs, ending `refs/tags/v3.0\n`,
  no trailing blank line (od -c clean for both grit and real git).
- `for-each-ref --count=100 --format="%(refname)"` -> same 5 refs for both
  grit and real git.

So grit MATCHES real git -> not a grit bug. (This confirms 4 prior agent
investigations on the ticket; the only new fact is that this task explicitly
authorizes editing the test file, which the prior agents were not permitted
to do.)

Root cause of the harness failure: the test did

    total=$(wc -l <all)

On macOS/BSD `wc` left-pads its count (`       5`). Command substitution
strips trailing newlines but NOT leading spaces, so `total` became the
8-char string `       5`. The test then passed `"$total"` (quoted) to
`test_line_count = "$total" actual`, which compares it against the file's
own (trimmed) line count `5`, so `test '5' = '       5'` -> FAIL:
"expected        5 lines (=), got 5".

Verified directly:

    total=$(wc -l <all)            # -> [       5]
    total=$(wc -l <all | tr -d ' ') # -> [5]

## Fix (sanctioned BSD-wc padding pattern, test-body only)

`tests/t9190-for-each-ref-atoms.sh`, subtest 20:

    -	total=$(wc -l <all) &&
    +	total=$(wc -l <all | tr -d " ") &&

This strips the BSD padding so the captured count matches the file's
trimmed count. Comparison MECHANISM only — no expected VALUE changed, no
differential proof of a value needed.

## Re-run

`./scripts/run-tests.sh t9190-for-each-ref-atoms.sh` -> `27/27` (✓).
TOML updated: `fully_passing = true`, `passed_last = 27`, `tests_total = 27`.
