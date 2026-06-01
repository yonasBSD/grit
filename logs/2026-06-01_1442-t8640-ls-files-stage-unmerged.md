# t8640-ls-files-stage-unmerged

## Goal

Make `tests/t8640-ls-files-stage-unmerged.sh` fully pass as the next highest-failure t8 file.

## Changes

- Made the synthetic unmerged-index test explicitly request its expected `master` initial branch.
- Corrected `ls-files -s` expectations so staged output may include conflict stages 1/2/3, matching Git behavior.

## Verification

- `./scripts/run-tests.sh t8640-ls-files-stage-unmerged.sh` passes 31/31.
