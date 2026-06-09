# t7301 clean interactive

Ticket: 8ac9c9

## Reproduction

- Baseline ticket run: `./scripts/run-tests.sh t7301-clean-interactive.sh` was 8/23.

## Findings

- `grit clean -i` only handled the top-level clean/quit prompts.
- Failing subtests exercise the scripted interactive submenus:
  `filter by pattern`, `select by numbers`, and `ask each`.
- Numbered selections use path-order display, while final removal still needs safe
  depth-first ordering for directories.
- The remaining prefix/path failures were from `git clean -id ..` inside `build/`;
  the explicit root pathspec normalized to an empty string and was treated like no
  pathspec, so only the current directory was scanned.

## Validation

- `./scripts/run-tests.sh t7301-clean-interactive.sh`: 23/23.
- `./scripts/run-tests.sh t7300-clean.sh`: 55/55.
- `cargo fmt`: completed.
- `cargo check -p grit-cli`: completed with the known `diff.rs` `ext_total` warning.
- `cargo clippy --fix --allow-dirty`: completed; printed the existing clippy warning backlog
  and failed-autofix diagnostics but exited 0 and left only this ticket's files dirty.
- `cargo test -p grit-lib --lib`: 252/254, failing only the two pre-existing gitignore glob
  tests (`dir_star_extension_matches_nested_path`, `nested_dir_star_extension`).
