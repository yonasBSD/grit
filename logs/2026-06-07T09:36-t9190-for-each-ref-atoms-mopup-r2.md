# t9190-for-each-ref-atoms — mop-up round 2 (2026-06-07T09:36)

Ticket: c62a5a. Status: 26/27 passing, unchanged from prior runs. No cascade from other agents.

## Re-run result

`./scripts/run-tests.sh t9190-for-each-ref-atoms.sh` → `✗ t9190-for-each-ref-atoms (26/27)`.

Only failing subtest: #20 `--count larger than ref count returns all`.

## Root cause (re-confirmed end-to-end, not a grit bug)

The test (tests/t9190-for-each-ref-atoms.sh:227-235) does:

```sh
grit for-each-ref --format="%(refname)" >all &&
total=$(wc -l <all) &&
grit for-each-ref --count=100 --format="%(refname)" >actual &&
test_line_count = "$total" actual
```

On macOS/BSD, `wc -l <all` emits a leading-whitespace-padded count, so
`total` = `"       5"` (verified: `total=[       5]`, len 8).

The active harness helper `test_line_count` (tests/test-lib.sh:1547) trims
whitespace only from the FILE's own wc output (`actual` → `5`), NOT from the
passed `count` argument. The comparison therefore becomes:

```
test "5" "=" "       5"   ->   FAIL: expected        5 lines (=), got 5
```

This reproduces the exact harness error message byte-for-byte.

## grit output is byte-for-byte correct

In a fresh trash repo replicating the test setup (5 refs:
feature, master, v1.0, v2.0, v3.0):

- `for-each-ref --format` emits exactly 5 newline-terminated refs, no trailing blank.
- `for-each-ref --count=100` emits the identical 5 refs.
- `od -c actual` tail shows clean `... v 3 . 0 \n` ending (no stray bytes).

So `--count` larger than the ref count correctly returns all refs. The grit
code (grit/src/commands/for_each_ref.rs, `max = count.unwrap_or(usize::MAX)`,
break when `printed >= max`) is correct.

## Why no grit change can fix this

The failure is entirely platform/harness-side: macOS BSD `wc -l` whitespace +
`test_line_count` trimming only one side of the comparison. No change to grit
Rust output can affect how `wc -l <all` pads its count or how the harness
compares it.

The only two viable fixes are both OFF-LIMITS per coexistence rules:
1. Trim the count arg in `test_line_count` (tests/test-lib.sh) — forbidden to modify test-lib.sh.
2. `total=$(wc -l <all | tr -d ' ')` in the test file — only allowed test edit is
   flipping test_expect_failure -> test_expect_success.

Upstream `git/t/test-lib-functions.sh` test_line_count has the same BSD-wc
vulnerability; the test only passes upstream on Linux/GNU wc (no leading space).

## Conclusion

Not a grit bug. Identical conclusion to the three prior agent investigations on
this ticket. Leaving the ticket blocked for the harness owner; no grit Rust
change committed because none is warranted.
