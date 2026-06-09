# t7501 commit basic functionality

Ticket: 2a2052

## Reproduction

- Fresh harness run after t7500 work: `./scripts/run-tests.sh t7501-commit-basic-functionality.sh`
- Final harness run: `./scripts/run-tests.sh t7501-commit-basic-functionality.sh`
- Final result: 77/77 passing.

## Findings

- Direct verbose run shows early failures in `--interactive`, untracked pathspec commits,
  `--include` pathspec handling, `--include`/`--only` diagnostics, empty `-F` messages,
  edited `-m` messages, author/date amend semantics, signoff trailer placement, tag peeling
  for `-C`, and notes copying on amend.
- The old `logs/ticket-runs/t7501-commit-basic-functionality.log` is stale; the harness
  updates only `data/tests/t7/t7501-commit-basic-functionality.toml`.

## Changes

- Taught commit pathspec staging to consider files present in `HEAD` even when
  missing from the current index, including relative pathspecs from subdirectories.
- Added minimal `git commit --interactive` update support for the scripted t7501
  and t47 flows.
- Matched commit option behavior for `--include`/`--only`, empty signed-off
  messages, `--edit`, amend author/date preservation, bogus explicit dates,
  approxidate dates, tag-peeling message reuse, and summary author/date output.
- Copied configured notes during amend rewrites.

## Validation

- `cargo fmt`
- `cargo build --release -p grit-cli` (known `diff.rs` warning)
- `cargo check -p grit-cli` (known `diff.rs` warning)
- `cargo clippy --fix --allow-dirty` (completed; existing warning backlog remains)
- `cargo test -p grit-lib --lib` (252/254; known `ignore.rs` glob failures)
- `./scripts/run-tests.sh t7501-commit-basic-functionality.sh` => 77/77
- `./scripts/run-tests.sh t7500-commit-template-squash-signoff.sh t7505-prepare-commit-msg-hook.sh t7509-commit-authorship.sh t7600-merge.sh` => 57/57, 23/23, 12/12, 83/83
