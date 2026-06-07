# t10990-write-tree-clean-dirty.sh — investigation

Ticket: 1b50d6
Date: 2026-06-07
Agent: schacon+claude-opus@gmail.com

## State

36/37 passing. The single failing subtest is #21
`write-tree after update-index remove`.

## Ticket premise (INCORRECT)

The ticket claims: "a plain `--remove` (without `--add`) always drops the index
entry regardless of whether the file is present on disk; only `--add --remove`
is conditional on file existence."

This is wrong. It contradicts both the documentation and real git's behavior.

## Ground truth

`git/Documentation/git-update-index.adoc`:

    --remove::
        If a specified file is in the index but is missing then it's
        removed.
        Default behavior is to ignore removed files.

"is missing" = missing from the worktree (lstat fails). When the file is
present on disk, `--remove` does NOT remove the entry.

C source `git/builtin/update-index.c`:
- `process_path()` (line 380): if the entry exists and is NOT skip-worktree,
  and `lstat` succeeds (file present), it falls through to `add_one_path(ce, ...)`
  (line 413), which UPDATES (re-stats/re-hashes) the existing entry — it does
  NOT remove it.
- `remove_one_path()` is only reached via `process_lstat_error()` (line 408),
  i.e. only when the file is MISSING from disk.
- `--remove` only sets `allow_remove=1`; it does not force removal of present
  files. `--force-remove` is what unconditionally removes.

grit's implementation in `grit/src/commands/update_index.rs` (lines 844-865)
already matches this exactly:
- path missing on disk  -> remove the entry
- path present on disk  -> fall through to re-stat/update (keep) the entry

## Proof that real git fails this exact subtest

Reproduced the exact subtest-21 preconditions (file `ui.txt` present on disk and
tracked in the index) with **real git 2.52.0**:

    git update-index --add ui.txt   # tracked
    git add ui.txt                   # staged, present on disk
    git update-index --remove ui.txt # exit 0
    git ls-files | grep ui           # ui.txt STILL PRESENT
    git write-tree; git ls-tree $T   # ui.txt STILL IN TREE

Real git keeps `ui.txt`, so `! grep "ui.txt" actual` (the subtest's assertion)
FAILS under real git too. grit produces byte-identical behavior.

## Conclusion

This is a TEST BUG, not a grit bug. `t10990-*` is a grit-authored test (it
mixes `grit` and `$REAL_GIT`; it does not exist in upstream `git/t/`). Subtest
21 encodes an assertion that contradicts documented and actual git semantics.

Per the working rules I may NOT edit the test assertion (only
`test_expect_failure` -> `test_expect_success` flips are permitted, and this
subtest is already `test_expect_success`). grit is already correct. Marking the
ticket blocked with this proof; no Rust change is warranted.
