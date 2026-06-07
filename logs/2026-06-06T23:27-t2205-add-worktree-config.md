# t2205-add-worktree-config — ticket e973fb

Date: 2026-06-06T23:27 (UTC)

## Starting state
12/13 passing. Only failing subtest: `12: 3b: ignored`.

## Root cause
NOT a grit bug. The ported test file
`tests/t2205-add-worktree-config.sh` had a transcription error in the
`3a: setup--add repo dir` setup block: the heredoc that writes
`expect-ignored-unsorted` was missing the line `actual-ignored-unsorted`
that is present in the upstream reference `git/t/t2205-add-worktree-config.sh`.

Subtest 3b runs:
```
git --git-dir=repo/.git ls-files -io --directory --exclude-standard >actual-ignored-unsorted
```
The shell redirect creates the empty file `actual-ignored-unsorted` before
the command runs, so it exists at scan time and is correctly reported as
ignored (matches `actual-*` in .gitignore). grit's output therefore
correctly includes `actual-ignored-unsorted`, but the (mis-ported) expected
file did not, causing the diff.

## Verification that grit is correct
Reproduced the exact 3b command with both real `git` and grit on an
identical fixture; output is byte-identical (both include
`actual-ignored-unsorted`). No Rust change was needed or made.

## Fix
Restored the dropped `actual-ignored-unsorted` line in the 3a heredoc in
`tests/t2205-add-worktree-config.sh` to match the upstream reference, so the
expected output now matches correct Git/grit behavior.

## Result
13/13 passing. `data/tests/t2/t2205-add-worktree-config.toml` updated
(fully_passing = true). No Rust changes; only the 2 known pre-existing
`ignore::gitignore_glob_tests` unit failures remain (unrelated to this ticket).
