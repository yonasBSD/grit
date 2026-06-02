# t0 worktree merge/orchestration — 2026-06-02

## Worktree inventory

- `wf/t0/path-utils`: code changes for `git_path` and `rev-parse --git-path`.
- `wf/t0/cache-tree`: cache-tree helper/index/status/commit changes; branch log said remaining
  failures were due to `ls-tree -d`.
- `wf/t0/reftable`: reftable log-block, reflog, compaction, and locking changes; remaining failures
  are cross-command transaction/sort/httpd issues.
- `wf/t0/repo-setup`: investigation log only; no owned-module defect fix.
- `wf/t0/*-2`: branch heads already equal `main` (`831730a95`).

`but status -fv` is not available in this checkout (`main` is not a `gitbutler/*` branch), so the
inventory used read-only `git worktree`, `git branch`, `git show`, and `git diff` commands.

## Merged/finished work

- Confirmed the staged main tree already contained the path-utils, cache-tree, and reftable code
  changes from the wf branches.
- Verified `t0060-path-utils` now passes 219/219.
- Fixed the final `t0090-cache-tree` failures by making `ls-tree -d` treat trailing-slash directory
  pathspecs (`dir/`, `./dir/`) as a request to descend and list child directories, while preserving
  `dir` as an exact tree entry match.
- Verified `t0090-cache-tree` now passes 22/22.
- Ran `t3105-ls-tree-output` as an ls-tree guard; it passes 60/60.

## Reftable status

- `t0610-reftable-basics`: 89/91.
- `t0613-reftable-write-options`: 10/11.
- `t0611-reftable-httpd`: 0/1; lane notes indicate the current environment serves with Apple Git,
  which cannot read reftable repos.

## Current t0 state

After focused reruns, `data/test-files.csv` reports 72 in-scope t0 rows:

- 59 fully green.
- 13 non-green.
- 32 failing subtests total.

Remaining non-green rows:

- `t0001-init` 94/102
- `t0002-gitfile` 13/14
- `t0007-git-var` 26/27
- `t0027-auto-crlf` 0/0 timeout
- `t0028-working-tree-encoding` 20/22
- `t0033-safe-directory` 20/22
- `t0034-root-safe-directory` 0/0
- `t0050-filesystem` 10/11
- `t0110-environment` 26/31
- `t0600-reffiles-backend` 25/33
- `t0610-reftable-basics` 89/91
- `t0611-reftable-httpd` 0/1
- `t0613-reftable-write-options` 10/11

## Verification run

- `cargo fmt`
- `cargo build --release -p grit-cli` passed with existing warnings.
- `cargo clippy --fix --allow-dirty --allow-staged` completed after sandbox escalation. It left the
  existing warning backlog and reported failed auto-fixes in unrelated binary files
  (`bundle_uri_test_tool.rs`, `mergetool.rs`). Unrelated auto-fixes in `config.rs` and
  `filter_process.rs` were backed out; two local `reftable.rs` style fixes were kept.
- `cargo test -p grit-lib --lib` -> 233/233.
- Final `cargo build --release -p grit-cli` passed with existing warnings.
- `./scripts/run-tests.sh t0060-path-utils.sh` -> 219/219
- `./scripts/run-tests.sh t0090-cache-tree.sh` -> 22/22
- `./scripts/run-tests.sh t3105-ls-tree-output.sh` -> 60/60
- `./scripts/run-tests.sh t0610-reftable-basics.sh` -> 89/91
- `./scripts/run-tests.sh t0613-reftable-write-options.sh` -> 10/11
- `./scripts/run-tests.sh t0611-reftable-httpd.sh` -> 0/1
