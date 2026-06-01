# t8970-symbolic-ref-chains

## Goal

Make `tests/t8970-symbolic-ref-chains.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic symbolic-ref chains test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8970-symbolic-ref-chains.sh` passes 30/30.
