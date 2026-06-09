# t6427-diff3-conflict-markers — 5d3ab2

Date: 2026-06-06T14:26 UTC
Result: 9/9 passing (was 7/9).

## Failing subtests
- 7: `rebase --merge describes parent of commit being picked` — expected `||||||| parent of` in conflict marker.
- 8: `rebase --apply describes fake ancestor base` — expected `||||||| constructed fake ancestor`.

## Root cause
Non-interactive rebase replays each commit through `cherry_pick_for_rebase`
(`grit/src/commands/rebase.rs`). For a non-root commit it ran the three-way
merge via `merge_trees_for_single_cherry_pick` (replay.rs), which always passed
`short_oid(parent_oid)` as the diff3 ancestor (base) label. `resolve_conflict_labels`
in `grit/src/commands/merge.rs` then turned that hex prefix into `<oid>:content`,
so the marker read e.g. `||||||| d3702f9:content` for BOTH the merge and apply
backends. Upstream Git instead uses, per backend (git/sequencer.c get_message /
replay.c / builtin/am.c):
- merge backend: `parent of <abbrev> (<subject>)` (sequencer `out->parent_label`)
- apply backend: `constructed fake ancestor` (git-am `o.ancestor`)

The struct `RebaseConflictContext::label_base()` already knew these strings but
was only consumed by the root-commit / whitespace-fix paths (`three_way_merge_with_content`),
never by the common non-root cherry-pick path. Also `label_base()` for the merge
backend was missing the abbreviated oid (`parent of <subject>` rather than
`parent of <abbrev> (<subject>)`).

## Fix
- `RebaseConflictContext`: added `picked_short_oid`; `label_base()` for the Merge
  backend now formats `parent of <abbrev> (<subject>)` to match the sequencer.
- Threaded an explicit `base_label_override: Option<&str>` through
  `merge_trees_for_single_cherry_pick` → `merge_trees_for_replay` → `merge_trees`
  → `resolve_conflict_labels`. When set, the override is used verbatim as the
  base label (no `:content` suffix), since these labels are human-readable, not
  oid prefixes.
- `cherry_pick_for_rebase` now passes `Some(conflict_ctx.label_base())` into
  `merge_trees_for_single_cherry_pick`.
- All other callers of the touched functions pass `None` (unchanged behavior).

Files: grit/src/commands/rebase.rs, grit/src/commands/replay.rs,
grit/src/commands/merge.rs, grit/src/commands/merge_recursive.rs,
grit/src/commands/am.rs.

## Regression checks (isolated --data-dir, all unchanged vs baseline)
t3404 76/132, t6437 20/22, t6406 13/13, t3501 21/21, t4200 34/36, t3406 32/32,
t4301 42/44, t6404 6/6, t3430 4/34. grit-lib unit tests pass modulo the 2 known
ignore::gitignore_glob_tests failures.
