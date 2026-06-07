# t3404-rebase-interactive (ticket 9e2eff) — mop-up round 1 (claude-t5)

Started at 92/132. Continuing from prior agent's notes
(`logs/2026-06-06T18:42-t3404-rebase-interactive.md`).

## Fix A: --update-refs todo generation (decoration ordering) — test 122

Rewrote the interactive todo generation to mirror Git's exact ordering in
`complete_action` (sequencer.c). New `build_autosquash_with_update_refs`
(grit/src/commands/rebase.rs) builds a typed `TodoBuildItem` list where
`update-ref`/`# Ref … checked out` decorations are inserted *after each pick*
(Git's `todo_list_add_update_ref_commands`) *before* the autosquash rearrange
(`todo_list_rearrange_squash`). The fixup/squash threading uses the same
`next`/`tail` linked list keyed on the combined item array; decoration items
keep their original positions, so trailing fixups land between a pick and its
update-ref lines. Decoration order is reverse-alphabetical (Git prepends each
decoration in `add_name_decoration`); the current HEAD branch is skipped.
`# Ref … checked out at '<path>'` now quotes the path (Git's format).

Result: test 122 passes (was failing). 92 -> 93.

## Fix B: --update-refs actually applies refs — test 124

`update-ref` todo lines were parsed as `Noop` and never applied. Added:
- `RebaseReplayStep::UpdateRef(String)` + `ParsedRebaseTodoLine::UpdateRef`
  (parser now also accepts the `u` abbreviation).
- `UpdateRefRecord` + `read_update_refs_state` / `write_update_refs_state`
  for the `rebase-merge/update-refs` state file (`ref\nbefore\nafter\n` triples).
- `write_rebase_update_refs(git_dir, todo_body)` now initializes the state from
  the finalized todo's `update-ref` lines (before = ref's current oid, after = 0),
  called after the post-edit todo is written (not from the raw commit list).
- `do_update_ref` (records current HEAD as a ref's pending `after` when the
  `update-ref` command runs in the replay loop).
- `apply_update_refs` at rebase completion (Git's `do_update_refs`): moves each
  ref from `before`→`after` when `after` is set and differs, prints
  "Updated the following refs with --update-refs:" (alphabetical, stderr) after
  the "Successfully rebased" line and before clearing rebase state.

Result: test 124 passes.

## Fix C: `git commit --amend --fixup=<ref>` keeps message (root cause of 124)

`prepare_commit_message` returned HEAD's existing message early for
`--amend && !use_editor`, *before* the fixup branch, so
`commit --amend --fixup=L` kept the old "extra2" message instead of "fixup! L".
Guarded that early return with `&& fixup.is_none()`. Now `--amend --fixup`
produces the `fixup!`/`amend!`/`squash!` subject (commit.rs).

Net: 92 -> 94 / 132.

## Remaining (38) — clusters
- 123 (--update-refs + --rebase-merges todo generation: the rebase-merges path
  builds its own todo and doesn't yet interleave update-ref decorations).
- 125-129 (--update-refs edit-todo paths: respect user edits / removed lines /
  re-added / edit-todo with no update-ref / failed ref update — need
  `todo_list_filter_update_refs` on edit-todo + check-failed-ref reporting).
- 130, 131 (bad labels/refs + non-merge reject merge commits: todo validation msgs).
- 70, 107 (exec-after-autosquash + abbreviateCommands), 91 (collision abbrev),
  79/80 (--root untracked conflict), 85 (commentchar=auto deprecation),
  86-89, 94-96, 100-104, 117-120, 18/35/54/57 (cascade + misc).
