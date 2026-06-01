# t8940-for-each-ref-points-at

## Goal

Make `tests/t8940-for-each-ref-points-at.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic for-each-ref points-at test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8940-for-each-ref-points-at.sh` passes 29/29.
