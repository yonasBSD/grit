# Libify: cherry-pick pure pick-engine helpers -> grit_lib::porcelain::cherry_pick

## Target
`grit/src/commands/cherry_pick.rs` (3421 lines). The plan asked to extract the
pure pick/apply-commit core. On inspection the file is overwhelmingly
orchestration: `CHERRY_PICK_HEAD`/`sequencer/*` state files, editor/hook
subprocess dispatch, `eprintln!`/`println!` conflict hints, `std::process::exit`
exit-code mapping, and `crate::ident::*`/`super::*` CLI-internal calls thread
through nearly every function. The actual three-way merge already delegates to
`grit_lib::merge_trees::merge_trees_three_way`. So the genuinely pure, used,
self-contained slice is small — this is the "DEFER if mostly orchestration"
case, and I extracted the clean slice rather than forcing the entangled whole.

## What moved (new file `grit-lib/src/porcelain/cherry_pick.rs`)
The pure, presentation-free pick-engine helpers (no clap, no I/O, no print, no
env, no exit, no state files):

- `WhitespaceStrategyOptions` (struct, pub fields) + `parse_strategy_options` —
  translate `-X<option>` merge-strategy options into a `MergeFavor`, whitespace
  flags, and an optional diff algorithm.
- The directory-rename detection cluster used to surface Git's transitive
  "file location" conflicts: `same_blob`, `parent_dir`,
  `remap_path_by_directory_renames`, `same_blob_renames`,
  `directory_renames_from_file_renames`, `stage_entry_at`,
  `path_has_unmerged_entry`.

All exported `pub`. anyhow was not involved (these helpers are infallible / pure
transforms), so no `bail!`/`context` message translation was needed.

## Deduplicated
The CLI's local `tree_to_index_entries` and `tree_to_map` were byte-identical
copies of the already-extracted `grit_lib::porcelain::merge::{tree_to_index_entries,
tree_to_map}`. Deleted the local copies; the CLI now `use`s the lib versions.

## What stayed in the CLI (correctly)
- `apply_transitive_file_location_conflicts` — emits a `CONFLICT (file location)`
  line via `println!`, so the presentation stays in the CLI; it now calls the
  moved pure helpers (`directory_renames_from_file_renames`, `same_blob_renames`,
  `remap_path_by_directory_renames`, `stage_entry_at`, `path_has_unmerged_entry`,
  `same_blob`).
- All sequencer-state, `CHERRY_PICK_HEAD`/`MERGE_MSG` bookkeeping, hook/editor
  dispatch, `--continue/--skip/--abort/--quit`, exit-code mapping, and the
  cwd-obstruction preflights (`bail_if_df_merge_would_remove_cwd`,
  `preflight_cherry_pick_cwd_obstruction`, both also called by `rebase.rs`).
- The dead `three_way_merge_with_content` / `content_merge_or_conflict` /
  `same_blob_content_modulo_trailing_newline` / `stage_entry` cluster was left in
  place: it is unreachable (the live flow uses `merge_trees_three_way`, and
  rebase has its own copies) and was already dead before this change. Removing it
  is an orthogonal cleanup, kept out of this focused extraction.

## CLI changes
- Added `use grit_lib::porcelain::cherry_pick::{...}` and
  `use grit_lib::porcelain::merge::{tree_to_index_entries, tree_to_map}`.
- Deleted the moved defs and the duplicate tree helpers.
- Dropped the now-unused `parse_tree` from the `grit_lib::objects` import.
- Net: grit/src/commands/cherry_pick.rs 8 insertions, 217 deletions.

## Verification (byte-exact gate)
Baselines (all fully_passing before): t3500-cherry 4/4, t3501-revert-cherry-pick
21/21, t3502-cherry-pick-merge 12/12, t3505-cherry-pick-empty 17/17 — all still
pass at full count after the change. Also re-ran the wider family with no
regression: t3503 6/6, t3504 9/9, t3506 11/11, t3507 44/44, t3508 14/14,
t3509 9/9, t3510-sequence 55/55, t3511-x 22/22, and t3404-rebase-interactive
107/132 (matches its non-fully-passing baseline; rebase imports two untouched
cwd helpers from cherry_pick).

`cargo build --release -p grit-cli -j 4`: clean (the 3 remaining warnings are
pre-existing and in files I did not touch: commit_graph_file.rs, diff.rs,
merge.rs, repack.rs). `cargo test -p grit-lib --lib`: 289 passed, only the 2
known `ignore::gitignore_glob` failures.

## Notes for the next agent
A pre-existing rustfmt-only diff in `grit-lib/src/porcelain/status.rs` was sitting
in the working tree (ambient noise, not mine) and was deliberately excluded from
the commit.
