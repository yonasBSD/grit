# t7615 diff-algorithm cherry-pick

Ticket: 7205d6

Initial state: `./scripts/run-tests.sh t7615-diff-algo-with-mergy-operations.sh`
reported 5/7 passing. The merge subtests already pass, but the two
cherry-pick histogram cases fail.

Findings:
- `grit merge` parses `-Xdiff-algorithm=...` and `diff.algorithm` config.
- `grit cherry-pick` only parses favor and whitespace `-X` options.
- `grit_lib::merge_trees::merge_trees_three_way` always calls the line merge
  engine with `diff_algorithm: None`, so cherry-pick cannot influence the
  merge algorithm even after parsing it.

Implemented:
- Threaded an optional diff algorithm through `merge_trees_three_way`.
- Taught `cherry-pick` to read `-Xdiff-algorithm=...`, legacy `-Xhistogram`
  / `-Xpatience`, and `diff.algorithm` when no command-line algorithm is set.
- Left checkout/revert callers at the previous default algorithm.

Result: `./scripts/run-tests.sh t7615-diff-algo-with-mergy-operations.sh`
now reports 7/7 passing.
