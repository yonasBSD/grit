# t7602-merge-octopus-many

Ticket: f9b67b

## Start

- Claimed ticket for `tests/t7602-merge-octopus-many.sh`.
- Noted existing unassigned `merge.rs` / t7600 changes before editing; inspecting whether this can be safely worked without mixing tickets.

- Current failures are merge output only: missing octopus pretty-name progress lines and extra `[branch sha]` commit summaries. Patched octopus and ordinary merge success output.

- `./scripts/run-tests.sh t7602-merge-octopus-many.sh` now reports 5/5 passing.

## Validation

- `cargo fmt`: passed.
- `cargo check -p grit-cli`: passed with existing `diff.rs` unused assignment warning.
- `cargo clippy --fix --allow-dirty`: exit 0; existing warning backlog and auto-fix failure messages remain.
- `cargo test -p grit-lib --lib`: 252 passed, 2 existing ignore glob unit tests failed.
- `./scripts/run-tests.sh t7602-merge-octopus-many.sh`: 5/5 passing.
