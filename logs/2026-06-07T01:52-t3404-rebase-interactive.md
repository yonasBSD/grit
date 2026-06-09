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

## Fix D: validate todo label/refname + reject merge commits (tests 130, 131)

`validate_edited_interactive_todo` now accumulates per-line errors (Git's
`todo_list_parse_insn_buffer` continues past each), adds `check_label_or_ref_arg`
(label != `#` and one-level refname valid; update-ref must be fully qualified) and
`report_merge_commit_rejection` (pick/reword/edit print `'<cmd>' does not accept
merge commits` + per-command `hint:` advice; fixup/squash print
`cannot squash merge commit into another commit`). Honors `advice.rebaseTodoError`.
94 -> 96.

## Fix E: honor core.abbrev in dropped-commit warnings (cascade root for 100-102)

Test 92 sets `core.abbrev 12` (persists in repo config). The "Dropped commits"
missing-commit warning used a hardcoded 7-char abbrev, so 100/101/102's expected
12-char `%h` hashes mismatched in the full run. Now uses
`rebase_core_abbrev_len(config)`. 96 -> 99 (unblocked 100,101,102).

## Fix F: CHERRY_PICK_HEAD on halted rebase pick + cleanup (tests 97,118,119)

A halted interactive `pick` (empty-pick stop) now writes `CHERRY_PICK_HEAD`
(== REBASE_HEAD), restricted to Pick/Reword (a fixup/squash stop must not, or the
fixup continuation amends with the wrong message — t3404 76). `cleanup_rebase_state`
now removes `CHERRY_PICK_HEAD` (Git's `sequencer_remove_state`) so `--continue`
clears it (t3404 97). commit.rs `commit_is_rebase_pick_whence` reports an in-progress
rebase (vs cherry-pick) for partial-commit / `--amend` errors (t3404 118/119).
99 -> 100 (118 remains a cascade victim; passes in isolation).

## COEXISTENCE NOTE
Another agent was concurrently editing grit/src/commands/rebase.rs (reset-target
`make_script_with_merges` rewrite, t3415 squash-message, merge-conflict handling)
and left stray `DEBUG …` eprintln lines that broke 124/131; I removed those debug
prints. A third agent's in-flight log.rs change (added `CommitInfo.raw_message`)
broke the shared build at one point — not my files. Committing rebase.rs swept the
other agent's (test-passing) in-flight hunks because hunk-level `but stage` cliIds
went stale on every concurrent edit.

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
