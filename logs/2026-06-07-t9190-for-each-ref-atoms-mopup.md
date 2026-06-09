# t9190-for-each-ref-atoms mop-up (ticket c62a5a)

Date: 2026-06-07T06:04

## State at start
- TOML: 26/27 passing, subtest 20 failing.
- Prior agents (comments on c62a5a) concluded subtest 20 is a BSD-`wc` harness issue, not a grit bug.

## Fresh re-run
- `./scripts/run-tests.sh t9190-for-each-ref-atoms.sh` -> 26/27 (unchanged). No cascade from other agents.

## Subtest 20: "--count larger than ref count returns all" (lines 227-235)
```sh
grit for-each-ref --format="%(refname)" >all &&
total=$(wc -l <all) &&
grit for-each-ref --count=100 --format="%(refname)" >actual &&
test_line_count = "$total" actual
```

Verbose failure:
```
test_line_count: expected        5 lines (=), got 5 in 'actual'
```

## Root cause (confirmed, re-verified)
- grit output is byte-for-byte correct: 5 refs, each newline-terminated, no trailing blank line (verified with `od -c`).
- grit `--count=100` correctly emits all 5 refs (grit/src/commands/for_each_ref.rs:128 `max = opts.count.unwrap_or(usize::MAX)`, then breaks when `printed >= max`). This matches upstream.
- The failure is purely shell arithmetic: on macOS/BSD, `total=$(wc -l <all)` captures `       5` (leading whitespace). It is passed as the `count` arg to `test_line_count`.
- The active `test_line_count` (tests/test-lib.sh:1547) trims whitespace from the *file's* `actual` count (`tr -d ' '` -> `5`) but NOT from the passed-in `count` (`       5`). So `test "5" = "       5"` fails.
- Upstream git/t/test-lib-functions.sh has the same BSD-wc vulnerability; it only passes on GNU/Linux wc (no leading space).

## Why no grit fix is possible
No grit Rust change affects macOS `wc` whitespace formatting of `$(wc -l <all)`. grit's bytes are already correct and identical to upstream.

## Only valid fixes are off-limits per coexistence rules
1. Trim `count` in `test_line_count` (tests/test-lib.sh) — OFF LIMITS (cannot modify test-lib.sh).
2. Change test to `total=$(wc -l <all | tr -d ' ')` (test file) — OFF LIMITS (test edits limited to expect_failure->expect_success flips).

## Conclusion
Confirmed not a grit bug. grit implementation is correct. Blocked on harness owner. Setting ticket state to blocked.
