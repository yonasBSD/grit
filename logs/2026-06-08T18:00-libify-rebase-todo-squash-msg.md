# Libify: rebase todo-command model + squash/fixup message assembly

Target: `rebase` (Phase 7). Source `grit/src/commands/rebase.rs` (12,312 lines).
Destination: new `grit_lib::porcelain::rebase`.

## What moved

The rebase command is a ~12k-line stateful sequencer entangled with editor
prompts, hook dispatch, revision resolution against a live `Repository`, and
worktree mutation. I extracted the cleanest self-contained slice: the
presentation-free, repository-free **todo-command vocabulary** plus the **pure
string transforms** that build a rebase's squash/fixup message buffer.

Moved to `grit-lib/src/porcelain/rebase.rs` (all now `pub`):

- `RebaseTodoCmd` enum + impl (`as_str`, `parse_word`) — pick/reword/fixup/squash.
- `FixupMessageMode` enum (UseCommit / EditCommit).
- Autosquash subject matching: `commit_subject_single_line`,
  `skip_fixupish_prefix`, `strip_fixupish_chain`,
  `format_autosquash_subject_for_match`.
- Todo-display: `rebase_todo_command_for_display`,
  `rebase_todo_command_for_display_abbrev`.
- Squash/fixup message buffer builders (faithful ports of `sequencer.c`):
  `message_body_after_subject`, `skip_blank_lines`, `fixup_replacement_message`,
  `first_line_len`, `squash_comment_subject_prefix`, `append_commented`,
  `update_squash_message_for_fixup`, `copy_section`,
  `append_skipped_squash_message`, `append_nth_squash_message`.

These compute results from text alone — no clap, no stdout/ANSI, no `std::env`,
no `crate::`, no `Repository`/`ConfigSet`. The only lib dependency added is
`crate::interpret_trailers::complete_line` (already in lib) and
`crate::objects::CommitData`.

## What stayed in the CLI (deferred)

Everything entangled: argv preprocessing, onto/upstream computation against a
live repo, the `parse_interactive_rebase_todo_line` /
`parse_rebase_replay_step` parsers (depend on `resolve_revision(repo, …)`), the
`SequencerStep`/replay enums, `--rebase-merges` script generation, editor/hook
dispatch, todo-file state I/O, worktree writes, and exit-code mapping. These are
left in place for a future pass. Extracting a small clean slice and deferring
the entangled remainder is the intended outcome.

## CLI wiring

`grit/src/commands/rebase.rs`: deleted the moved defs (net -315 lines); added
`use grit_lib::porcelain::rebase::{…}` so bare-name call sites resolve. Dropped
three names from the import (`copy_section`, `rebase_todo_command_for_display`,
`squash_comment_subject_prefix`) that only had in-lib (transitive) callers — the
surviving CLI calls the rest directly.

## Verification (byte-exact)

- `cargo build --release -p grit-cli -j 4`: clean, no warnings in my files.
- `cargo test -p grit-lib --lib`: 289 passed, only the 2 known
  `ignore::gitignore_glob` failures.
- Harness (recorded baseline → result):
  - t3404-rebase-interactive: 103 → **107** (recorded baseline was STALE; I
    stashed my changes, rebuilt the ORIGINAL tree, and re-measured t3404 — the
    untouched tree also yields 107/132, confirming byte-exact, not a regression).
  - t3406-rebase-message: 32/32 → 32/32 (fully_passing).
  - t3415-rebase-autosquash: 28/28 → 28/28 (fully_passing).
  - t3421-rebase-topology-linear: 63/63 → 63/63 (fully_passing).
  - t3430-rebase-merges: 34/34 → 34/34 (fully_passing).
  - No `fully_passing` flipped true→false; every passed count >= baseline.

## Files

- new `grit-lib/src/porcelain/rebase.rs`
- `grit-lib/src/porcelain/mod.rs` (+`pub mod rebase;`)
- `grit/src/commands/rebase.rs` (delete moved defs + import-back)
