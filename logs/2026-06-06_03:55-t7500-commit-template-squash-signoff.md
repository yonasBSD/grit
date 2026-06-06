# t7500 commit template squash signoff

Ticket: 17ae1e

## Reproduction

- `./scripts/run-tests.sh t7500-commit-template-squash-signoff.sh`: 42/57.
- Direct verbose run confirms failing subtests: 10-13, 30-33, 36-38, 49, 51, 52, 54.

## Findings

- Initial focus is the template-edit group, because subtests 10-13 share the same
  editor/template path and fail before the later autosquash cases.
- The harness exports `VISUAL=:` globally, while `test_set_editor` and inline editor
  cases set `EDITOR`. Commit editor launch resolved the no-op `VISUAL` and never fell
  through to a non-noop `EDITOR`, so template/fixup/squash editor edits were skipped.
- The remaining status-template failure was byte-for-byte output: blank commented
  lines should render as `#`, not `# `.

## Fixes

- Let commit editor launch fall through from a resolved no-op `:` editor to a
  non-empty, non-noop `EDITOR`.
- Render empty commented template lines without a trailing space.

## Validation

- `./scripts/run-tests.sh t7500-commit-template-squash-signoff.sh`: 57/57.
- `./scripts/run-tests.sh t7505-prepare-commit-msg-hook.sh`: 23/23.
- `./scripts/run-tests.sh t7600-merge.sh`: 83/83.
- `cargo fmt`: completed.
- `cargo check -p grit-cli`: completed with the known `diff.rs` `ext_total` warning.
- `cargo clippy --fix --allow-dirty`: completed with the existing warning backlog.
- `cargo test -p grit-lib --lib`: 252/254, failing only the two pre-existing gitignore glob
  tests (`dir_star_extension_matches_nested_path`, `nested_dir_star_extension`).
