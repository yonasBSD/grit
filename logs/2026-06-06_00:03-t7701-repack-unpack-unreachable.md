# t7701-repack-unpack-unreachable

Ticket: 6d1170

## Start

- Claimed ticket for `tests/t7701-repack-unpack-unreachable.sh`.
- Starting with baseline reproduction and upstream test/doc review.
- Baseline `./scripts/run-tests.sh t7701-repack-unpack-unreachable.sh`: 1/9 passing.
- Found `repack -A -d -l` omitted reflog-only commits from the first pack. Narrowed `pack-objects --all --reflog --unpack-unreachable` to include reflog tips while leaving ordinary full repacks unchanged.
- After that fix, `./scripts/run-tests.sh t7701-repack-unpack-unreachable.sh`: 3/9 passing.
- Fixed `--unpack-unreachable` materialization to write loose copies even when the object exists in a local pack, stopped pruning newly loosened objects when no expiry cutoff is supplied, removed old packs for `repack -A -d`, and made `-A` without `-d` behave like `-a`.
- `./scripts/run-tests.sh t7701-repack-unpack-unreachable.sh`: 4/9 passing.
- Applied `--unpack-unreachable=<date>` at the source-pack level so old packs are not loosened before deletion.
- `./scripts/run-tests.sh t7701-repack-unpack-unreachable.sh`: 5/9 passing.
- Preserved source pack mtimes on loosened loose objects and honored `gc.recentObjectsHook` during cutoff pruning.
- `./scripts/run-tests.sh t7701-repack-unpack-unreachable.sh`: 6/9 passing.
- Fixed `--keep-unreachable` to include the full local object database, not just the reachable closure.
- `./scripts/run-tests.sh t7701-repack-unpack-unreachable.sh`: 9/9 passing.
- `cargo check -p grit-cli`: passed with the existing `diff.rs` unused-assignment warning.
- `./scripts/run-tests.sh t7700-repack.sh`: 36/47, unchanged baseline for that open ticket.
- `cargo clippy --fix --allow-dirty`: completed with the repository's existing warning backlog and temporary auto-fix diagnostics.
- `cargo test -p grit-lib --lib`: failed only the two known ignore glob tests.
