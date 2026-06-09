# t8950-show-ref-patterns — work log (ticket c3d261)

Date: 2026-06-07T00:32 UTC
Agent: schacon+claude-t5

## Status
27/29 passing. Failing: subtest 19 (`show-ref --hash shows only SHAs`) and
subtest 21 (`show-ref --tags --hash lists only SHAs for tags`).

## Root cause — NOT a grit bug (test-file portability defect)

grit's `--hash` output is byte-for-byte identical to `/usr/bin/git`:
a 40-char hex SHA followed by a single `\n` (41 bytes total). Verified with
`xxd` against both grit and real git. The `show_ref.rs` implementation
(`grit/src/commands/show_ref.rs`, `print_one` / `hash_only` path) is correct.

The two failing subtests fail solely because of a BSD (macOS) `wc -c`
portability bug in the *synthetic test file*:

```sh
len=$(printf "%s" "$sha" | wc -c) &&
test "$len" = 40
```

On macOS, BSD `wc -c` emits leading whitespace: `len` becomes the string
`"      40"` (with leading spaces). The comparison is *quoted*
(`test "$len" = 40`), so `"      40" != "40"` and the test fails.

On Linux, GNU `wc -c` emits `40` with no leading whitespace, so the test
passes there. Contrast with subtests 4 and 27 in the same file, which use the
unquoted command-substitution form `test $(... | wc -c) = 40` — word-splitting
collapses the spaces, so those pass even on BSD.

This test is fully synthetic (no upstream `git/t/t8950-...sh` exists; it calls
`grit` and `$REAL_GIT` directly). The bug is in the test's shell logic, not in
grit.

## Why no Rust fix was committed
grit already produces the exact correct output (confirmed identical to
`/usr/bin/git`). No change to grit-lib or grit CLI can make a quoted
`test "      40" = 40` succeed. The only fix is to the test body — e.g. change
`test "$len" = 40` to `test "$len" -eq 40` (numeric comparison tolerates the
leading whitespace) or strip whitespace from the `wc` output — but editing test
bodies is prohibited by the ticket rules (only `test_expect_failure` ->
`test_expect_success` flips are allowed, which does not apply here).

## Recommendation for mop-up
If the rules permit a portability fix to this synthetic test, change the two
`test "$len" = 40` comparisons to `test "$len" -eq 40` (numeric `-eq` ignores
the BSD `wc` leading whitespace). That makes both subtests pass on macOS while
remaining correct on Linux. Otherwise this file is effectively
environment-blocked on BSD `wc` and grit itself needs no change.
