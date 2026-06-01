# t8170-init-reinitialize

## Goal

Make `tests/t8170-init-reinitialize.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic init reinitialize test explicitly request its expected `master` initial branch.
- Applied the documented cwd-leak wrapper to test bodies.

## Verification

- `./scripts/run-tests.sh t8170-init-reinitialize.sh` passes 35/35.
