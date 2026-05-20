# t2400-worktree-add: hooks, HEAD, missing worktrees (2026-05-19)

- Resolve HEAD from per-worktree `git_dir` (fixes `cd wt` bad-HEAD warnings).
- Run `post-checkout` after `worktree add` with `GIT_DIR`/`GIT_WORK_TREE` in the new tree.
- Load hooks from `commondir` for linked worktrees.
- Skip creating empty `.git/hooks` on init so harness `mkdir .git/hooks` + `test_hook` works.
- Reclaim missing registered worktrees with `-f` / `-f -f`; `AddArgs.force` is now a count.
- t2400: 226/232 passing; remaining: submodules (225, 227), relative paths (229–232).
