# t8590-for-each-ref-filter

## Goal

Make `tests/t8590-for-each-ref-filter.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic for-each-ref filter test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8590-for-each-ref-filter.sh` passes 30/30.
