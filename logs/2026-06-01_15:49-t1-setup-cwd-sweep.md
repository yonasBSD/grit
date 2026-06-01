# t1 setup cwd sweep

## Claim
- Claimed the t1 files with exactly one passing test that matched the `cd repo` setup leak seen in `t13190-log-format-body`.

## Findings
- There were 46 t1 rows with exactly one passing test after `t13190-log-format-body` was fixed.
- 41 matched the setup-cwd pattern: setup enters `repo`, then later assertions use `(cd repo && ...)`.
- 5 did not match that pattern: `t1022-read-tree-partial-clone`, `t1407-worktree-ref-store`, `t1419-exclude-refs`, `t1422-show-ref-exists`, and `t1462-refs-exists`.

## Work
- Wrapped the setup body in a subshell for the 41 matching files so later tests run from the trash root.

## Verification
- Ran `./scripts/run-tests.sh` across all 41 changed files.
- 23 files became fully passing.
- The other 18 improved beyond one passing test and now expose command-specific failures.
- `cargo test --workspace`: skipped; no Rust code changed.
- `./tests/harness/run.sh`: skipped; project uses `./scripts/run-tests.sh` for CSV/dashboard updates.
