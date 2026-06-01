# t8770-status-branch-tracking

## Goal

Make `tests/t8770-status-branch-tracking.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic status branch tracking test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8770-status-branch-tracking.sh` passes 34/34.
