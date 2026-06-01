# t8060-symbolic-ref-extra

## Goal

Make `tests/t8060-symbolic-ref-extra.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic symbolic-ref test explicitly request its expected `master` initial branch.
- Fixed `update-ref --no-deref HEAD <oid>` so it still writes a direct `HEAD` when the resolved target already has that OID.

## Verification

- `./scripts/run-tests.sh t8060-symbolic-ref-extra.sh` passes 33/33.
- Neighbor check `./scripts/run-tests.sh t8600-update-ref-symref.sh` remains 24/28.
- `cargo check` passes with existing warnings.
- `cargo test -p grit-lib --lib` passes.
- `cargo clippy --fix --allow-dirty` completes with the existing workspace clippy warning backlog.
