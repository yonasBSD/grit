# log.decorate=full decorations

## Goal

Make `git log --format="%s%d"` honor `log.decorate=full` from the merged config, including config passed through `GIT_CONFIG_PARAMETERS` by `for-each-repo`.

## Notes

- Reproduced `t0068-for-each-repo.sh` failing because `%d` printed `HEAD -> one` instead of `HEAD -> refs/heads/one`.
- Kept the fix in `grit/src/commands/log.rs`; this does not overlap the open PR files.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `cargo check -p grit-cli`
- `cargo clippy --fix --allow-dirty -p grit-cli` (completed with pre-existing warnings)
- `cargo test -p grit-lib --lib`
- `./scripts/run-tests.sh --output-csv /tmp/grit-t0068-fixed.csv --no-catalog t0068-for-each-repo.sh` -> `5/5`
