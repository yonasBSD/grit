# t8050-update-index-modes.sh — investigation log

Ticket: 62e3f9 (test, t8)
Date: 2026-06-06T23:50Z
Agent: schacon+claude-t5

## Status: 29/31 passing — the 2 remaining failures are UNFIXABLE without breaking git compatibility.

## Failing subtests
- #5  `update-index --remove removes a file from index`
- #11 `ls-files --stage shows all entries with modes`  (pure cascade of #5)

## Subtest #5 — the real problem

The test body:
```sh
git update-index --remove world.txt &&
git ls-files >actual &&
! grep -q "world.txt" actual
```
At this point `world.txt` STILL EXISTS on disk and is unchanged relative to the index
(it was created and `--add`ed in subtest #3, never deleted). The test asserts that plain
`--remove` drops the entry for a file that is present on disk.

This contradicts real git semantics. Verified directly against `git version 2.52.0`:
plain `git update-index --remove <path>` does NOT remove a present, unchanged file — it
only removes the index entry when the path is MISSING from the worktree (lstat ENOENT).
When the path still exists, git falls through to `add_one_path` and keeps/updates it.
`--force-remove` is the flag that unconditionally drops a present file.

Reference: git/builtin/update-index.c
- `update_one()` lstats the path (success here, file present).
- `process_path()` -> file exists, not dir -> `add_one_path()` -> entry up-to-date -> kept.
- `remove_one_path()` is only reached from `process_lstat_error()` when lstat fails.
Docs git/Documentation/git-update-index.adoc `--remove`:
  "If a specified file is in the index but is missing then it's removed."

grit already matches git exactly. grit/src/commands/update_index.rs (PathMode::Remove,
~lines 844-865): on `--remove`, if `symlink_metadata(abs_path)` succeeds (file present)
it falls through to the normal stat-and-update path; only an Err (gone from disk) removes
the entry. This is correct.

## Subtest #11
Asserts `test_line_count = 4` for `ls-files --stage`. Because #5 leaves world.txt in the
index, there are 5 entries, so #11 fails too. Fixing #5 fixes #11; both have one root cause.

## Why this CANNOT be fixed by changing grit

The current `--remove` semantics were deliberately set in commit 5a17e9cc9
("fix(diff): update-index --remove semantics ... t4007/t4009 green, t4002 60/63").
Reverting to "always remove if tracked" (the old grit behavior these custom tests were
written against) would re-break upstream tests that rely on correct git semantics:
- t4007-rename-3.sh:92  `git update-index --remove path0/COPYING` (path0/COPYING was
  renamed away — GONE from disk — git removes; behavior depends on the present/absent split).
- t4009, t4002, t4005, t4008, t4011 — same commit fixed spurious diff `M` lines / remove
  semantics together.

The case t8050 #5 wants (remove a file that is PRESENT and unchanged) is indistinguishable
from t4007's legitimate case EXCEPT by on-disk presence, which is exactly the signal git
(and grit) already use correctly. There is no git-compatible way to satisfy #5.

## Note: identical regression in another file (NOT in my scope, do not touch)

t10990-write-tree-clean-dirty.sh #21 "write-tree after update-index remove" has the SAME
pattern (`grit update-index --remove ui.txt` with ui.txt present) and is NOW FAILING too,
though its TOML still claims 37/37 fully_passing. Its TOML is stale; a fresh run shows
"failed 1 among 37". That file's owner should either (a) accept the failure as a buggy
custom test, or (b) delete ui.txt before the remove in the test body. Out of scope here.

## Conclusion
t8050 subtests #5 and #11 are buggy grit-custom tests (no upstream git/t/t8050 exists) that
codify pre-5a17e9cc9 incorrect `--remove` behavior. grit is correct; the tests are wrong.
No code change is appropriate. Left at honest 29/31. Per the harness rules I cannot edit
the test body (only test_expect_failure->success flips), so this stays at 29/31.
