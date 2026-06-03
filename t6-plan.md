# T6 Plan

Generated from `data/test-files.csv` on 2026-06-03. Work highest failing counts first inside
dependency/topic groups. Mark an item `[~]` when claimed and `[x]` only after the harness row has
`failing=0`.

## Rev-List And Revision Traversal

- [x] `t6021-rev-list-exclude-hidden.sh` — 62/62 passing. Highest current t6 blocker;
  depends on rev-list option parsing, pseudo-ref globbing, and hidden-ref filtering.
- [x] `t6018-rev-list-glob.sh` — 95/95 passing after extending pseudo-ref glob and exclude
  handling across `rev-list`, `rev-parse`, and `shortlog`.
- [x] `t6002-rev-list-bisect.sh` — 53/53 passing after adding rev-list bisection
  midpoint selection, `--bisect-vars`, `--bisect-all`, and bisect-ref defaults.
- [x] `t6111-rev-list-treesame.sh` — 65/65 passing after dense TREESAME traversal,
  path-limited parent rewriting, ancestry-bottom pruning, simplify-merges path rewriting, and
  adjacent merge-parent topo ordering.
- [~] `t6600-test-reach.sh` — 43/47 passing, 4 failing after adding the
  upstream `test-tool reach` helper operations and first-parent `%(is-base)` selection.
- [ ] `t6022-rev-list-missing.sh` — 16/40 passing, 24 failing.
- [ ] `t6006-rev-list-format.sh` — 58/80 passing, 22 failing.
- [ ] `t6007-rev-list-cherry-pick-file.sh` — 6/23 passing, 17 failing.
- [ ] `t6012-rev-list-simplify.sh` — 26/42 passing, 16 failing.
- [ ] `t6000-rev-list-misc.sh` — 9/23 passing, 14 failing.
- [ ] `t6003-rev-list-topo-order.sh` — 23/36 passing, 13 failing.
- [ ] `t6019-rev-list-ancestry-path.sh` — 5/18 passing, 13 failing.
- [ ] `t6137-rev-parse-misc.sh` — 23/34 passing, 11 failing.
- [ ] `t6016-rev-list-graph-simplify-history.sh` — 2/12 passing, 10 failing.
- [ ] `t6136-rev-list-date-range.sh` — 24/31 passing, 7 failing.
- [ ] `t6015-rev-list-show-all-parents.sh` — 32/38 passing, 6 failing.
- [ ] `t6138-rev-list-boundary.sh` — 24/29 passing, 5 failing.
- [ ] `t6001-rev-list-graft.sh` — 10/14 passing, 4 failing.
- [ ] `t6101-rev-parse-parents.sh` — 35/38 passing, 3 failing.
- [ ] `t6010-merge-base.sh` — 10/12 passing, 2 failing.
- [ ] `t6011-rev-list-with-bad-commit.sh` — 5/6 passing, 1 failing.
- [ ] `t6013-rev-list-reverse-parents.sh` — 2/3 passing, 1 failing.

Completed rev-list/revision files: `t6004`, `t6005`, `t6007-rev-list-cherry-pick-status`,
`t6008`, `t6009`, `t6011-rev-list-with-hierarchies`, `t6014`, `t6017`, `t6100`, `t6102`,
`t6110`, `t6112`, `t6113`, `t6114`, `t6115`, `t6135-rev-list-merge-order`, `t6601`, `t6700`.

## Merge Machinery

- [x] `t6423-merge-rename-directories.sh` — 80/82 passing, 0 failing, with 2 expected
  failures.
- [x] `t6438-submodule-directory-file-conflicts.sh` — 56/56 passing.
- [ ] `t6430-merge-recursive.sh` — 11/36 passing, 25 failing.
- [ ] `t6402-merge-rename.sh` — 24/46 passing, 22 failing.
- [ ] `t6416-recursive-corner-cases.sh` — 24/37 passing, 13 failing, with 3 expected failures.
- [ ] `t6415-merge-dir-to-symlink.sh` — 13/24 passing, 11 failing.
- [ ] `t6422-merge-rename-corner-cases.sh` — 11/20 passing, 9 failing, with 6 expected failures.
- [ ] `t6430-merge-strategy-option.sh` — 0/6 passing, 6 failing.
- [ ] `t6436-merge-overwrite.sh` — 12/18 passing, 6 failing.
- [ ] `t6418-merge-text-auto.sh` — 7/11 passing, 4 failing.
- [ ] `t6421-merge-partial-clone.sh` — 0/3 passing, 3 failing.
- [ ] `t6400-merge-df.sh` — 5/7 passing, 2 failing.
- [ ] `t6411-merge-filemode.sh` — 17/19 passing, 2 failing.
- [ ] `t6427-diff3-conflict-markers.sh` — 7/9 passing, 2 failing.
- [ ] `t6432-merge-recursive-rename-options.sh` — 1/3 passing, 2 failing.
- [ ] `t6434-merge-with-no-common-ancestor.sh` — 1/3 passing, 2 failing.
- [ ] `t6404-recursive-merge.sh` — 5/6 passing, 1 failing.
- [ ] `t6424-merge-unrelated-index-changes.sh` — 18/19 passing, 1 failing.
- [ ] `t6435-merge-sparse-directory.sh` — 1/2 passing, 1 failing.

Completed merge files include `t6060`, `t6401`, `t6403`, `t6405`, `t6406`, `t6407`, `t6408`,
`t6409`, `t6412`, `t6413`, `t6414`, `t6417`, `t6425`, `t6426`, `t6428`, `t6429`, `t6431`,
`t6432-merge-recursive-space-options`, `t6433`, `t6434-merge-recursive-rename-options`,
`t6435-merge-sparse`, and `t6437`.

## Pathspec

- [ ] `t6135-pathspec-with-attrs.sh` — 7/37 passing, 30 failing.
- [ ] `t6131-pathspec-icase.sh` — 1/9 passing, 8 failing.
- [ ] `t6136-pathspec-in-bare.sh` — 1/3 passing, 2 failing.
- [ ] `t6133-pathspec-rev-dwim.sh` — 5/6 passing, 1 failing.

Completed pathspec files: `t6130`, `t6132`, `t6133-pathspec-toplevel`, `t6134-*`, and
`t6137-pathspec-wildcards-literal`.

## Describe

- [ ] `t6120-describe.sh` — 54/103 passing, 49 failing.

Completed describe/name files: `t6120-name-rev`.

## Bundle, Object Reachability, And GC

- [ ] `t6020-bundle-misc.sh` — 13/37 passing, 24 failing.
- [ ] `t6501-freshen-objects.sh` — 36/42 passing, 6 failing.
- [ ] `t6500-gc.sh` — 34/35 passing, 1 failing.

## Bisect

- [ ] `t6030-bisect-porcelain.sh` — 85/96 passing, 11 failing.
- [ ] `t6041-bisect-submodule.sh` — 7/14 passing, 7 failing.

## Tracking And Ref Formatting

- [x] `t6040-tracking-info.sh` — 44/44 passing.
- [x] `t6200-fmt-merge-msg-extra.sh` — 23/23 passing.
- [x] `t6300-for-each-ref.sh` — 429/429 passing.
- [x] `t6301-for-each-ref-errors.sh` — 6/6 passing.
- [x] `t6304-for-each-ref-detached-head.sh` — 10/10 passing.

Skipped rows: `t6050-replace`, `t6200-fmt-merge-msg`, `t6302-for-each-ref-filter`.
