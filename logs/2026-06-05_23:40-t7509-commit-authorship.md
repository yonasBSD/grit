# t7509-commit-authorship

Ticket: fe49b3

## Start

- Claimed ticket for `tests/t7509-commit-authorship.sh`.
- Reproducing current failures before editing code.
- `./scripts/run-tests.sh t7509-commit-authorship.sh`: 3/12 passing baseline.
- First failure was `git commit -a -c Initial` ignoring `EDITOR=:` / `VISUAL=:` and reporting `Terminal is dumb, but EDITOR unset`.
- Adjusted editor resolution to preserve `:` as Git's no-op editor and to fall back to `vi` when `TERM` is not dumb, matching `t7005` expectations.
- Fixed `git commit -c <rev>` message preparation to seed `COMMIT_EDITMSG` from the reused commit message before launching the editor.
- `./scripts/run-tests.sh t7509-commit-authorship.sh`: 12/12 passing.
- `cargo check -p grit-cli`: passed with the existing `diff.rs` unused-assignment warning.
- `./scripts/run-tests.sh t7005-editor.sh`: 12/12 passing.
- `cargo clippy --fix --allow-dirty`: completed with the repository's existing warning backlog and temporary auto-fix diagnostics.
- `cargo test -p grit-lib --lib`: failed only the two known ignore glob tests.
