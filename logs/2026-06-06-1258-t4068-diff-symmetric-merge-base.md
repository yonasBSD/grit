# t4068-diff-symmetric-merge-base

Ticket: 22eb99
Date: 2026-06-06T12:58

## Failing subtest
- 8: "diff with ranges and extra arg"
  ```sh
  test_must_fail git diff main br1..main commit-D 2>err &&
  test_grep "usage" err
  ```

## Root cause
`grit/src/commands/diff.rs` rev-argument validation (the analogue of
builtin/diff.c `symdiff_prepare`) rejected:
- multiple `...` symmetric tokens, or one `...` mixed with other revs
- more than one `..` range token

but it did NOT reject a single `..` range token combined with extra revs.
So `git diff main br1..main commit-D` (3 revs incl. one `..` range) was
classified as an auto-combined diff (`revs.len() >= 3`). The combined-diff
path then tried to resolve `br1..main` literally as one revision, which
failed with an "ambiguous argument" message (exit 128) instead of the
expected "usage" error (the C code does `if (lpos >= 0 && othercount > 0)
usage(...)` — a range's left side sets lpos, any extra rev sets othercount).

## Fix
Added `|| (two_dot_range_tokens == 1 && revs.len() != 1)` to the usage-bail
condition, mirroring the existing `...` rule and the C `lpos >= 0 &&
othercount > 0` check. A range argument may not be combined with any other
revision.

## Verification
- `git diff main br1..main commit-D` -> usage error, exit 1 (was exit 128)
- `git diff commit-C main commit-D` (3 plain commits) -> combined diff, exit 0 (unchanged)
- `git diff br1..main` (plain range) -> exit 0 (unchanged)
- Full file: 36/36 passing.
