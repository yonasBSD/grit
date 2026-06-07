# t8050-update-index-modes.sh — mop-up round 1

Ticket: 62e3f9 (t8050-update-index-modes: subtests failing)

## Result: 29/31, UNFIXABLE within harness rules (verified)

Re-ran fresh after other agents' fixes: still 29/31 (no cascade improvement).

## Failing subtests
- #5  `update-index --remove removes a file from index`
- #11 `ls-files --stage shows all entries with modes` (pure cascade of #5)

## Root cause — buggy grit-custom tests, NOT a grit bug
No upstream `git/t/t8050` exists; this is a grit-authored test that codifies
pre-commit-5a17e9cc9 grit behavior which contradicts real git.

Test #5 runs `git update-index --remove world.txt` while `world.txt` still
EXISTS on disk and is unchanged, then asserts `! grep -q world.txt`.

Verified against real git 2.52.0:
- `--remove` on a PRESENT, unchanged file: file STAYS in index (exit 0).
- `--remove` on a MISSING file: file is removed.
This matches `git-update-index` docs: "--remove: If a specified file is in the
index but is missing then it's removed. Default behavior is to ignore removed
files."

Verified grit binary produces the IDENTICAL behavior to real git for both cases
(present-unchanged keeps; missing removes). So grit is correct; the test is wrong.

#11 asserts `test_line_count = 4` but world.txt correctly remains => 5 entries.
Pure cascade of #5.

## Why not fixable
- The current correct semantics were introduced in commit 5a17e9cc9 to make
  t4007/t4009/t4002 green. Reverting to "always remove" would re-break those.
- The only way to make #5/#11 pass is to edit the test body (delete world.txt
  before `--remove`, or relax the line-count assertion). Harness rules forbid
  editing test bodies; the sole permitted test edit is
  test_expect_failure -> test_expect_success for a bug actually fixed (not the
  reverse), so flipping these to expect_failure is also disallowed.

## Disposition
TOML already accurate: tests_total=31, passed_last=29, failing=2,
fully_passing=false, status="ok". No grit code change warranted.
Ticket left open as a known-buggy-test record (fix requires test-body edit which
is out of scope for the test-porting harness).

grit-lib unit tests: only the 2 known ignore::gitignore_glob_tests failures
(unrelated to this ticket).
