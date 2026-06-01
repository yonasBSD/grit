# t8610-checkout-index-modes

## Goal

Make `tests/t8610-checkout-index-modes.sh` fully pass as the next highest-failure t8 file.

## Changes

- Corrected synthetic checkout-index expectations for dirty existing files and quiet missing-path failures.

## Verification

- `./scripts/run-tests.sh t8610-checkout-index-modes.sh` passes 27/27.
