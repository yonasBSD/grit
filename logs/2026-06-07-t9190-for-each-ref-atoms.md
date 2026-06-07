# t9190-for-each-ref-atoms — work log (2026-06-07)

Ticket: c62a5a. Status at start: 26/27 passing; only subtest 20 failing.

## Subtest 20: "--count larger than ref count returns all"

```sh
(
cd repo &&
grit for-each-ref --format="%(refname)" >all &&
total=$(wc -l <all) &&
grit for-each-ref --count=100 --format="%(refname)" >actual &&
test_line_count = "$total" actual
)
```

## Root cause: BSD `wc` leading whitespace + harness comparison, NOT a grit bug

Grit output is provably correct. Manual repro in the trash repo:

- `grit for-each-ref --format="%(refname)"` -> 5 refs
- `grit for-each-ref --count=100 --format="%(refname)"` -> 5 refs (correctly clamps to all)

Both 18/19 (`--count=1`, `--count=2`) and 25/26 (count combined with sort/pattern)
pass. Those use **literal** count arguments. Subtest 20 is the only one that
captures `wc -l` into a shell variable and feeds it back as the expected count.

On macOS/BSD, `total=$(wc -l <all)` yields `"       5"` (leading whitespace).
`test_line_count` (tests/test-lib.sh:1547) trims only the *file's* wc output
(`actual`), not the passed-in `count` argument:

```sh
actual=$(wc -l <"$file"); actual=$(echo "$actual" | tr -d ' ')
test "$actual" "$op" "$count"     # count = "       5", actual = "5" -> NOMATCH
```

So `test "5" = "       5"` fails. Confirmed via direct shell repro:

```
total=[       5]   actual_lines=[       5]   (both grit calls identical)
test 5 = "       5"  ->  NOMATCH
```

The real upstream `test_line_count` (git/t/test-lib-functions.sh:1055) has the
**same** vulnerability on BSD wc — it works upstream only because CI runs on
Linux/GNU coreutils where `wc -l` emits no leading whitespace.

## Why no grit fix is possible

`total` is produced by `wc`, never by grit. Grit's `--count` clamping and refname
formatting are already correct. The mismatch is entirely between BSD `wc`'s output
format and the harness's one-sided whitespace trimming.

The only real fixes are off-limits per the coexistence rules:
- Modify `tests/test-lib.sh` `test_line_count` to also trim `count` (forbidden:
  "Do NOT modify tests/test-lib.sh").
- Modify the test file's `total=$(wc -l <all)` to strip whitespace, e.g.
  `total=$(wc -l <all | tr -d ' ')` (forbidden: only test edit allowed is
  test_expect_failure -> test_expect_success).

## Conclusion

No grit code change applies. Leaving ticket open with this diagnosis for the
mop-up agent / harness owner. The fix belongs in the harness (trim `count` in
test_line_count) or the ported test (strip wc whitespace into `total`).
