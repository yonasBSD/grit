# t8310-for-each-ref-format-deep

## Goal

Make `tests/t8310-for-each-ref-format-deep.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic for-each-ref format test explicitly request its expected `master` initial branch.

## Verification

- `./scripts/run-tests.sh t8310-for-each-ref-format-deep.sh` passes 32/32.
