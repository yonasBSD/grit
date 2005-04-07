# t7 — pathspec outside repo, blame abs paths, reset --merge, submodule helper

## t7010-setup (16/16)

- `resolve_pathspec_in_worktree` rejects paths outside the work tree (Git fatal message).
- `resolved_pathspecs_for_add` uses the same check for `git add`.
- `normalize_worktree_file_path` for `git blame` absolute paths under the work tree.

## t7426-submodule-get-default-remote (15/15)

- `get-default_remote_for_path_in_super` resolves `../subpath` from cwd and canonicalizes.

## t7111-reset-table (42/42)

- `check_merge_reset_worktree` no longer skips verification when HEAD OID equals target (t7111 row `A B C C merge XXXXX`).
