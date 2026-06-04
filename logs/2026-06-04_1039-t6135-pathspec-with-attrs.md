# t6135 pathspec with attrs

## Goal

Finish `t6135-pathspec-with-attrs.sh`, currently the largest failing t6 row at 7/37.

## Notes

- Claimed after completing `t6424-merge-unrelated-index-changes.sh`.
- Test scope covers `:(attr:...)` pathspec magic for `ls-files`, `grep`, `stash push`, `add`,
  validation errors, subdirectory `.gitattributes`, and `builtin_objectmode`.
- First increment teaches pathspec attr magic to parse set/unset/unspecified/value requirements,
  validates malformed attr magic, preserves escaped commas in attr values, rejects attr magic from
  `check-ignore`, and makes `ls-files` load nested `.gitattributes` for attr pathspec matching.
- Direct run improves from 7/37 to 25/37. Official
  `./scripts/run-tests.sh t6135-pathspec-with-attrs.sh --quiet` records 25/37 with 12 failing.
  Remaining failures are tree-ish `grep`, `stash`/`add`, and status exclusion integration.
- Merged current `main` into `grit-t6` before committing this increment, then reapplied the work and
  rebuilt `target/release/grit`.
- Validation before commit: `cargo fmt`, `cargo check -p grit-cli`,
  `cargo clippy --fix --allow-dirty` (existing warning backlog and failed auto-fix attempts remain;
  restored the unrelated `filter_process.rs` auto-fix), `cargo test -p grit-lib --lib`,
  `cargo build --release -p grit-cli`, `./scripts/run-tests.sh t6135-pathspec-with-attrs.sh
  --quiet`, traced `t6416-recursive-corner-cases.sh` refresh to restore expected-failure accounting,
  and `git diff --check`.
- Second increment teaches tree-ish `grep` to evaluate attr pathspecs with attributes loaded from
  the searched tree, and to keep descending through trees when attr magic can match descendants.
  Direct and official t6135 runs improve from 25/37 to 30/37. Remaining failures are `stash push`,
  `add` variants, and status `builtin_objectmode` exclusion.
- Validation for the second increment: `cargo fmt`, `cargo check -p grit-cli`,
  `cargo clippy --fix --allow-dirty` (existing warning backlog and failed auto-fix attempts remain;
  restored the unrelated `filter_process.rs` auto-fix), `cargo test -p grit-lib --lib`,
  `cargo build --release -p grit-cli`, direct and official
  `t6135-pathspec-with-attrs.sh`, traced `t6416-recursive-corner-cases.sh` refresh, and
  `git diff --check`.
