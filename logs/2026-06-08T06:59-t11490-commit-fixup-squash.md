# t11490-commit-fixup-squash — port log

Date: 2026-06-08T06:59Z
Ticket: 53efd6

## Initial state
- `./scripts/run-tests.sh t11490-commit-fixup-squash.sh` => 32/33.
- Failing subtest: #31 "log -n limits output" (lines 356-362).

## Subtest 31 analysis (differential rule)
Body:
```
git log --oneline -n 3 >out &&
test "$(wc -l <out)" = "3"
```

Differential vs /opt/homebrew/bin/git (git 2.52.0), in /tmp scratch repo:
- grit `log --oneline -n 3` => exactly 3 lines.
- real git `log --oneline -n 3` => exactly 3 lines.
- Both produce BSD `wc -l` output `"       3"` (left-padded).

Root cause: BSD wc padding. The quoted command substitution `"$(wc -l <out)"`
preserves the left-padding, so the string comparison `"       3" = "3"` is
false on macOS. This is a pure comparison-mechanism portability bug — grit and
real git agree byte-for-byte.

Verified: `test "$(wc -l <out)" -eq 3` passes (numeric `-eq` parses the padded
value as 3). Note the sibling subtests using `-gt 5` already worked because
`-gt` is numeric.

## Fix
Changed the string comparison to numeric:
`test "$(wc -l <out)" = "3"` -> `test "$(wc -l <out)" -eq 3`.

Classification: test-bug-fixed (BSD wc padding pattern).

## Result
33/33 passing after fix.
