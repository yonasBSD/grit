# t6423-merge-rename-directories

## Context

- Claimed after `t6002-rev-list-bisect.sh` reached 53/53.
- Current CSV baseline: `t6423-merge-rename-directories` has 80 total tests, 29 passing, 51
  failing, and 2 expected failures.
- This is now the highest-failing remaining in-scope t6 row overall.

## Work Log

- Claimed `t6423-merge-rename-directories.sh`.
- Reproduced the current release-harness baseline at 29/82 passing, 51 failing, with 2 expected
  failures.
- Fixed rename/rename(2to1) worktree conflict marker labels for directory-rename-induced
  add/add conflicts by preserving the pre-directory-rename source path for each side. This flipped
  testcase `1d`.
- Changed directory rename map construction from "drop any split source" to Git's majority rule:
  when one destination has the most renamed paths it wins; exact ties are left unmapped. This
  flipped testcase `1f`.
- Added tied split detection for cases where the opposite side adds new files under the split
  source directory. These now leave the paths unmapped but mark the merge conflicted with a
  `directory rename split` message. This flipped testcase `2a` while preserving `2b`.
- Disabled directory rename application for a source directory that still has paths on both sides
  of the merge. This matches Git's "directory still exists on both sides" rule and flipped the
  section 4 cases reached by the current harness.
- Reported implicit directory rename conflicts when a same-side path collision blocks remapping,
  while avoiding the dual-directory fallback case where the colliding target is also under a
  same-side directory rename source. This flipped testcase `5a`.
- Left blocked transitive rename targets unmapped when directory renames would collide with an
  existing file or descendant directory, and staged the associated rename/add side for
  rename/rename/add-add cases. This flipped testcase `5c`.
- Treated descendants as "in the way" for implicit directory rename application, allowing the
  later file/directory conflict pass to stage `y/d~HEAD` and leave `z/d` unmapped. This flipped
  testcase `5d`.
- Avoided duplicate same-target transitive rename staging and preserved the pre-directory-rename
  target label for conflict markers. This flipped testcase `7b`.
- Suppressed doubly-transitive directory rename remaps when the intermediate target is itself a
  source directory renamed by the side being rewritten, relocated rename/delete D/F stages to
  `path~side`, and fixed explicit index-colon parsing for paths containing `^0`. This flipped
  testcase `7e`.
- Materialized the surviving side's blob for plain modify/delete conflicts so the conflicted file
  remains in the worktree. This flipped testcase `8c`.
- Current harness refresh: `./scripts/run-tests.sh t6423-merge-rename-directories.sh --verbose`
  reports 42/82 passing and regenerated `data/test-files.csv` plus dashboards.
- Added warnings for suppressed doubly-transitive directory renames and printed those notices on
  clean merges. This flipped `9c` and `9d`.
- Blocked N-to-1 implicit directory rename remaps as a single conflict while leaving all colliding
  source paths unmapped. This flipped `9e`.
- Extended merge preflight checks to protect untracked files that would be overwritten by
  conflict-file materialization, and aligned dirty/untracked error headers with Git. This flipped
  section 10 and section 11 (`10a`-`10d`, `11a`-`11f`).
- Refined transitive directory-rename suppression to allow nested mutual renames only when pure
  additions exist under the rewritten source. This preserved `12b1`/`12c1` and flipped
  `12b2`/`12c2`.
- Added root directory-rename destination support and root subtree fingerprint inference. This
  flipped `12d`, `12e`, and helped later rename-to-self coverage.
- Added merge-ort trace2 region markers used by recursive replay tests. This flipped `12f`.
- Implemented conflict-mode "in the way" handling for opposite-side files and rename sources,
  preserving collisions that should remain reported in `merge.directoryRenames=conflict`. This
  flipped `12i`, `12j`, `12k`, `12o`, and `12q`.
- Added directory-rename file-location notices for pure additions and rename targets, with
  conflict-mode `CONFLICT (file location)` output and explicit-true `Path updated:` output. This
  flipped `13a`, `13b(info)`, `13c`, and `13d`.
- Current release harness refresh: after `cargo build --release -p grit-cli`,
  `./scripts/run-tests.sh t6423-merge-rename-directories.sh --verbose` reports 75/82 passing and
  regenerated `data/test-files.csv` plus dashboards.
- Remaining real failures: `12i2`, `12l` in both directions, `12n`, and `13e`. `9g` and `12h`
  remain expected failures.
- Refined pure-add protection for nested mutual directory renames so additions under `sub1/sub2`
  are not remapped by the opposite `sub1 -> sub3` rename, while exact destination/source matches
  like `z/e -> y/e` still apply. Focused debug checks passed `5c` and both `12l` directions.
- Current release harness refresh: after `cargo build --release -p grit-cli`,
  `./scripts/run-tests.sh t6423-merge-rename-directories.sh --verbose` reports 77/82 passing and
  regenerated `data/test-files.csv` plus dashboards.
- Remaining real failures: `12i2`, `12n`, and `13e`. `9g` and `12h` remain expected failures.
