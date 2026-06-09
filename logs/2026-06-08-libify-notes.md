# Libify: `notes` — extract the notes-tree read/modify core into `grit_lib::notes`

## Target
- Command file: `grit/src/commands/notes.rs`
- Destination: new module `grit-lib/src/notes.rs` (`grit_lib::notes`)

## What moved (the domain-logic core)
The notes feature stores blobs in a fanout tree (`object hex -> note blob`) under
`refs/notes/commits` (or a `--ref` namespace). The pure tree operations over the
odb/refs were extracted as a self-contained library module, separate from editor
launch, output, stdin, clap, and exit-code mapping (all of which stayed in the CLI).

Moved to `grit_lib::notes`:
- Tree model + read: `NotesTreeEntry` (pub fields), `NotesTreeChild`,
  `note_object_name`, `display_note_path`, `collect_notes_tree_entries`,
  `read_notes_tree`.
- Fanout + write: `notes_fanout`, `path_with_fanout`, `write_notes_subtree`,
  `write_notes_commit`, `write_notes_commit_with_parents`, `build_ident_role`
  (Git `GIT_{AUTHOR,COMMITTER}_*` env ident — same precedent as
  `ident_resolve.rs`/`ident_config.rs`).
- Ref expansion: `expand_notes_ref`.
- Blob combine: `combine_notes_concatenate`, `combine_notes_cat_sort_uniq`,
  `note_blob_lines`, `read_blob_bytes`, `blob_to_lines`, `upsert_note_entry`.
- Notes-merge data model + engine: `NotesMergeStrategy`, `LocalNoteState`,
  `NotesMergePair`, `parse_notes_merge_strategy_value`, `notes_tree_blob_by_object`,
  `diff_note_blob_changes`, `build_merge_pairs`, `merge_note_blobs_conflict_markers`,
  `write_note_conflict_file`, `merge_one_note_change`, `merge_changes_into_entries`,
  `remote_unchanged`, `same_change_local_remote`, `no_local_change`,
  `adopt_remote_note`, `resolve_commit_tree`, `resolve_notes_commit_optional`,
  `notes_merge_inner`, plus the worktree-layout helpers `notes_merge_git_dir`,
  `notes_merge_worktree_path`, `notes_merge_worktree_nonempty` and the
  `NOTES_MERGE_WORKTREE` constant.

Public surface (items the surviving CLI calls): `NotesTreeEntry` (+ its `mode`/
`path`/`oid` fields), `NotesMergeStrategy`, `read_notes_tree`, `write_notes_commit`,
`write_notes_commit_with_parents`, `note_object_name`, `expand_notes_ref`,
`display_note_path`, `upsert_note_entry`, `combine_notes_concatenate`,
`combine_notes_cat_sort_uniq`, `parse_notes_merge_strategy_value`,
`notes_merge_git_dir`, `notes_merge_worktree_path`, `notes_merge_worktree_nonempty`,
`notes_merge_inner`. Internal-only helpers stayed private.

## What stayed in the CLI (`grit/src/commands/notes.rs`)
clap `Args`/`Subcommand`, argv reconstruction (`ordered_note_fragments_from_argv`,
`last_ordered_note_fragment_is_reuse_message`), editor launch (`launch_editor`,
`resolve_editor`), stdin/file reading, all `eprintln!`/`println!`/output, the
`std::process::exit(...)` exit-code mapping, the per-subcommand orchestration
(`add_note`, `append_or_edit_note`, `copy_notes`, `remove_notes`,
`merge_notes_dispatch`/`abort`/`commit`, `prune_notes`, list/show), the
notes-rewrite config (`RewriteCfg`, `load_rewrite_cfg`, `apply_rewrite_copy`,
`copy_notes_for_rewrite` — the cross-command API used by `commit.rs`), the
worktree-discovery helper `find_other_worktree_with_notes_merge_ref`, and the
`active_notes_ref`/`ensure_*` ref-policy guards.

## Mechanics
- `anyhow` `bail!`/`anyhow!` -> `crate::error::Error::Message` with BYTE-IDENTICAL
  text; redundant `.map_err(|e| anyhow::anyhow!(e))` over odb writes dropped (odb
  already returns `crate::error::Error`).
- `grit_lib::` -> `crate::` throughout the moved body.
- Added `merge3.workspace = true` to `grit-lib/Cargo.toml` (already a workspace dep,
  used by `merge_note_blobs_conflict_markers`).
- Added `pub mod notes;` to `grit-lib/src/lib.rs` (alphabetical, after `name_rev`).
- CLI imports trimmed; added `use grit_lib::notes::{...}`.

## Verification (byte-exact gate)
Baselines (all `fully_passing = true`) and post-change results — identical:
- t3301-notes: 153/153 (baseline 153)
- t3302-notes-index-expensive: 12/12 (baseline 12)
- t3303-notes-subtrees: 23/23 (baseline 23)
- t3304-notes-mixed: 6/6 (baseline 6)

`cargo build --release -p grit-cli` clean (no new warnings in either notes.rs).
`cargo test -p grit-lib --lib`: 289 passed, 2 failed — only the known
`ignore::gitignore_glob` failures. No data/tests TOML deltas.

## Files
- new: `grit-lib/src/notes.rs`
- modified: `grit/src/commands/notes.rs` (−928/+7), `grit-lib/src/lib.rs` (+1),
  `grit-lib/Cargo.toml` (+1), `Cargo.lock` (+merge3 under grit-lib).
