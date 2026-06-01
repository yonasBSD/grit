# t8820-branch-tracking-display

## Goal

Make `tests/t8820-branch-tracking-display.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic branch tracking display test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8820-branch-tracking-display.sh` passes 27/27.
