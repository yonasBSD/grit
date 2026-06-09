# t12590-log-format-tformat — port fix (BSD wc padding)

Date: 2026-06-08T07:04Z
Ticket: aabc5c (# t12590-log-format-tformat: 32/33 — BLOCKED (BSD wc padding))
Agent: grit-t5 (concurrent, scoped to tests/t12590-log-format-tformat.sh only)

## Starting state
- 32/33 passing. One failing subtest:
  - `not ok 22 - format with -n 2 shows two commits`

## Root cause (differential vs git 2.52.0 at /opt/homebrew/bin/git)
Subtest body was:
```
(cd repo && grit log -n 2 --format="%s" >../actual) &&
wc -l <actual >count &&
echo 2 >expect_count &&
test_cmp expect_count count
```
On macOS/BSD, `wc -l <actual` left-pads the count with spaces ("       2"),
while `expect_count` holds "2", so `test_cmp` fails. On GNU coreutils there is no
padding, so it passes there. This is the sanctioned BSD-wc-padding test-bug pattern.

### Differential proof
Reproduced in /tmp with identical inputs (3 commits) using BOTH the harness grit
(target/release/grit) and /opt/homebrew/bin/git:
- `grit log -n 2 --format="%s"` output == real git 2.52.0 output, byte-for-byte
  (`diff actual gactual` clean).
- grit emits exactly 2 lines; the only divergence is the BSD `wc` padding in the
  comparison mechanism, not grit's behavior.

Verdict: TEST bug (comparison mechanism / BSD wc padding). Not a grit bug.

## Fix (test-body only, no expected-value change)
Replaced the padded `wc -l >count` + `test_cmp` with a numeric command-substitution
comparison (command substitution + numeric `-eq` ignores the BSD padding):
```
test "$(wc -l <actual)" -eq 2
```

## Result
- `./scripts/run-tests.sh t12590-log-format-tformat.sh` -> 33/33, fully passing.
- TOML data/tests/t1/t12590-log-format-tformat.toml: passed_last=33, failing=0,
  fully_passing=true.

Classification: test-bug-fixed.
