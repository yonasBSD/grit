# t6424 merge unrelated index changes

## Goal

Finish `t6424-merge-unrelated-index-changes.sh`, which currently has one remaining failure.

## Notes

- Claimed after completing `t6404-recursive-merge.sh`.
- The remaining failure was `ff update`: Grit rejected a fast-forward from `A` to `E` when the
  index had an unrelated staged `random_file`.
- The fast-forward path already composes a target index that preserves unrelated staged additions
  and runs a per-path overwrite check. Removed the broad dirty-index abort so the per-path check
  decides whether the merge touches staged paths.
- Direct run now passes `19/19`.
- Official `./scripts/run-tests.sh t6424-merge-unrelated-index-changes.sh --quiet` records
  `19/19`, `0` failing. Re-ran the traced `t6416-recursive-corner-cases.sh` refresh afterward to
  restore its expected-failure total to `40/37/0`.
