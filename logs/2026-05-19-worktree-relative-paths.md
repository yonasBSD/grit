# t2400 worktree relative paths (2026-05-19)

## Goal
Finish `tests/t2400-worktree-add.sh` tests 229–232 (relative/absolute linking paths and `extensions.relativeWorktrees`).

## Root causes
1. **`extensions.relativeWorktrees` rejected** — `enable_relative_worktrees_extension` bumped repo format to v1, but `validate_repository_format` in `grit-lib` did not whitelist `relativeworktrees`. `test_config` / `test_unconfig` cleanup then failed; harness `test_run_` returns the clobbered `eval_ret` from cleanup, so tests 229/225/227 appeared to fail even when `worktree add` succeeded.
2. **Absolute gitdir paths** — `--no-relative-paths` wrote `./absolute` in paths; test 230 expects canonical paths without `.` components.
3. **Relative linking** — implemented via `write_worktree_linking_files` + `make_relative_path` in `grit/src/commands/worktree.rs`.

## Changes
- `grit-lib/src/repo.rs`: allow `relativeworktrees` extension for format v1.
- `grit/src/commands/worktree.rs`: relative/absolute worktree linking on add; canonicalize absolute paths.

## Result
`./scripts/run-tests.sh` equivalent: all **232/232** pass in `tests/t2400-worktree-add.sh` (clean env).

## Commit
`df05f4698` on `refactor-worktree-resolve-linked-head` via GitButler.
