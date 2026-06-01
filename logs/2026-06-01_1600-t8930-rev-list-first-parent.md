# t8930-rev-list-first-parent

## Goal

Make `tests/t8930-rev-list-first-parent.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic rev-list first-parent test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8930-rev-list-first-parent.sh` passes 32/32.
