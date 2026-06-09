# t7008-filter-branch-null-sha1.sh — ticket 259eba

## Status
6/6 passing (was 5/6).

## Root cause
`grit filter-branch` delegates to the system `git-filter-branch` shell script.
`resolve_filter_branch_script()` in `grit/src/commands/filter_branch.rs` only
probed `$GIT_EXEC_PATH/git-filter-branch` plus a hard-coded Linux candidate list
(`/usr/lib/git-core`, `/usr/libexec/git-core`, `/usr/local/...`). On macOS the
real script lives under the Xcode toolchain
(`/Applications/Xcode.app/Contents/Developer/usr/libexec/git-core`) or Homebrew,
so none of the candidates matched and grit bailed with
`cannot find git-filter-branch`.

That made subtest 6 ("removing the broken entry works") fail outright. Subtests 4
and 5 (`test_must_fail`) "passed" only by accident — the command failed because
the script was missing, not because of the intended null-sha1 error.

Extra wrinkle: the test harness exports `GIT_EXEC_PATH` pointing at a synthetic
helper dir (`$BIN_DIRECTORY/git-exec`) that only contains `git-p4`. So:
- `git --exec-path` echoes that synthetic value back instead of git's real
  `git-core` dir.
- The sourced `git-sh-i18n` helper can't be found when the script runs.

## Fix (grit/src/commands/filter_branch.rs)
1. Added `system_git_binary()` (probes `/usr/bin/git`, `/bin/git`) and
   `system_git_exec_path()` which runs `<git> --exec-path` with `GIT_EXEC_PATH`,
   `GIT_DIR`, `GIT_WORK_TREE` removed so the real built-in exec-path is reported.
2. `resolve_filter_branch_script()` now adds that discovered exec dir to the
   candidate list (after `$GIT_EXEC_PATH`, before the hard-coded Linux paths), so
   discovery is portable on macOS (Xcode/Homebrew) without breaking Linux.
3. In `run()`, export `GIT_EXEC_PATH` = the resolved real exec dir when invoking
   the script, so `git-sh-setup`/`git-sh-i18n` resolve and the previous stderr
   noise (`git-sh-i18n: No such file or directory`, `eval_gettext: command not
   found`) disappears. Subtests 4/5 now fail for the correct reason (genuine
   null-sha1 write-tree error), matching upstream.

## Verification
- `./scripts/run-tests.sh t7008-filter-branch-null-sha1.sh` -> 6/6.
- `cargo test -p grit-lib --lib` -> 276 passed; only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures remain (unrelated to this ticket).
- rustfmt + clippy clean on the changed file.
