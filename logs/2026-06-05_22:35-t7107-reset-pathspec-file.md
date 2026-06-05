# t7107-reset-pathspec-file

Ticket: e03718

## Start

- Claimed ticket for `tests/t7107-reset-pathspec-file.sh`.
- Reproducing current failures before editing code.

- Added reset argument support for `--pathspec-from-file` and `--pathspec-file-nul`; building before rerunning the harness.

- First t7107 rerun reached 10/11; remaining failure was Git-compatible fatal output for reset modes with pathspecs.

- Implemented reset pathspec file parsing. `./scripts/run-tests.sh t7107-reset-pathspec-file.sh` now reports 11/11 passing.

## Validation

- `cargo fmt`: passed.
- `cargo check -p grit-cli`: passed with existing `diff.rs` unused assignment warning.
- `cargo clippy --fix --allow-dirty`: exit 0; existing warning backlog and auto-fix failure messages remain.
- `cargo test -p grit-lib --lib`: 252 passed, 2 existing ignore glob unit tests failed.
- `./scripts/run-tests.sh t7107-reset-pathspec-file.sh`: 11/11 passing.
