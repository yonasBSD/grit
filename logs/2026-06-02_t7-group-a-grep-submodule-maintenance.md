# t7 Group A — grep, submodule checkout, maintenance batch

## t7818-grep-extended (11/11)

- Removed erroneous `-P` / USE_LIBPCRE bail; use regex crate for perl mode.
- `--all-match` with multiple `-e`: require all atoms on the same line.
- Leading `--and` before first `-e`: normalize to `Atom, And, Atom` chain.

## t7450-bad-git-dotfiles test 49 (50/50)

- `checkout_gitlink_worktree_entry`: call `submodule_gitdir_outer_conflict` before writing submodule gitfile so failed recurse checkout does not leave `thing2/.git`.

## t7900-maintenance test 24 (72/72)

- Root cause: `test-lib-commit-bulk.sh` fast-import path left loose objects; test expects `pack-*.pack` after bulk setup.
- Fix: `git repack -a -d -q` after fast-import checkout in `test_commit_bulk`.
