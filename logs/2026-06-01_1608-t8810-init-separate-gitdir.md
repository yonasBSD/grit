# t8810-init-separate-gitdir

## Goal

Make `tests/t8810-init-separate-gitdir.sh` fully pass as the next highest-failure t8 file.

## Changes

- Applied the documented cwd-leak wrapper to test bodies.

## Verification

- `./scripts/run-tests.sh t8810-init-separate-gitdir.sh` passes 27/27.
