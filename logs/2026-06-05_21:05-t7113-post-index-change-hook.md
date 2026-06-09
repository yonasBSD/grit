# t7113-post-index-change-hook

Ticket: 69aef2

## Start

- Claimed ticket for `tests/t7113-post-index-change-hook.sh`.
- Reproduced `1/4`; only setup passed.
- Upstream uses `index_state.updated_workdir` and `updated_skipworktree` when running `post-index-change`.

## Changes

- Added flag-aware index write helpers in `Repository`.
- Wired whole-tree checkout writes to run `post-index-change 1 0`.
- Wired reset final writes to run `1 0` for worktree reset modes and `0 1` for mixed reset.
- Wired CLI `update-index` writes to run `0 1`.
- Avoided rewriting the index for bare `git update-index`, which should not fire the hook.
- Added no-op checkout index writes for already-at-target branch creation/switching paths, including `checkout -B`.

## Results

- `./scripts/run-tests.sh t7113-post-index-change-hook.sh`: 4/4 passing.
- `cargo check -p grit-cli`: passed with the pre-existing `ext_total` warning in `diff.rs`.
- `cargo clippy --fix --allow-dirty`: completed with the known auto-fix failure/warning backlog in unrelated files.
- `cargo test -p grit-lib --lib`: 252 passed, 2 pre-existing ignore glob tests failed.
