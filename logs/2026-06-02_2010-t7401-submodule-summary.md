# t7401-submodule-summary

## 2026-06-02 20:10

- Claimed `t7401-submodule-summary.sh` after `t7403-submodule-sync.sh` reached 18/18.
- Starting baseline from `data/test-files.csv`: 10/25 passing, 15 failing.
- Direct run initially failed 15/25. Failures covered cwd-relative display/pathspecs, divergent
  range ordering and `--summary-limit`, gitlink/blob typechanges, deleted submodules, path-vs-rev
  disambiguation for a gitlink path, and missing-commit warnings.
- Fixed `submodule summary` to normalize pathspecs relative to the invocation directory and render
  submodule paths relative to that same cwd.
- Fixed summary argument parsing to prefer known gitlink paths over revision parsing, so
  `git submodule summary sm2` is treated as a path filter.
- Fixed divergent summaries to print destination-only commits before source-only commits and apply
  `--summary-limit` across that combined order.
- Fixed typechange/deletion/worktree cases by rendering gitlink/blob labels only for non-null
  typechanges, detecting a checked-out submodule when the index currently holds a blob, and
  reporting deleted submodule paths without requiring the submodule worktree to exist.
- Fixed missing-commit output to print `Warn: <path> doesn't contain commit <oid>` when the
  checked-out submodule lacks the source commit.
- Verification:
  - `cd tests && sh t7401-submodule-summary.sh -v` passed 25/25.
  - `./scripts/run-tests.sh t7401-submodule-summary.sh --verbose` passed 25/25 and refreshed
    `data/test-files.csv` plus dashboards.
  - Regression `./scripts/run-tests.sh t7403-submodule-sync.sh t7407-submodule-foreach.sh
    --verbose` remained green at 18/18 and 23/23.
