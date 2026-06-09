# Libify: revert-specific pick core → grit_lib::porcelain::revert

## Target
`grit/src/commands/revert.rs` — revert is the inverse pick. The shared pick
engine already landed in `grit_lib::porcelain::cherry_pick` (strategy-option
parsing, directory-rename detection, index staging) and `grit_lib::porcelain::merge`
(`tree_to_index_entries`, `tree_to_map`, fast-forward/octopus index composition).
The CLI revert flow already routes the tree/index transforms it shares with
cherry-pick through those modules (via `super::cherry_pick` / `super::merge`),
so what remained to extract was the residual *pure, revert-specific* core.

## What moved
Two self-contained, presentation-free clusters that compute results from
repo/object data alone (no clap, no `println!`/`eprintln!`, no `std::env`, no
tty/exit-code/editor/state-file work) into a new
`grit-lib/src/porcelain/revert.rs` (all `pub`, the surviving CLI calls them):

- `revision_set_newest_first` + its helper `committer_timestamp` — the
  `git rev-list <include> --not <exclude>` ordering used when reverting an
  `A..B` range, returned newest-first (descending committer timestamp). Called
  by the CLI's `expand_revert_specs`, which keeps the argv `^X` / `A..B`
  spec parsing (`resolve_revision`) and stays in the CLI.
- `merge_commit_message_for_revert` — the revert commit-message template
  (`Revert "..."` / `Reapply "..."` subject + `This reverts commit ...` body,
  plus the `--reference` form via `crate::commit_pretty::format_reference_line`).
  Returns `(title_line, body_suffix)`; the CLI assembles the final template and
  owns editor/cleanup/output.

`grit_lib::` → `crate::` (`commit_pretty::format_reference_line`, `objects`,
`repo`, `error::Result`). No `anyhow` text crossed the boundary: the moved
graph walk only propagates an odb read/parse error (already `crate::error::Result`),
and the message builder is infallible.

The CLI imports the moved items via
`use grit_lib::porcelain::revert::{merge_commit_message_for_revert,
revision_set_newest_first};` and the bare-name call sites resolve unchanged.

### Deliberately NOT moved
- `walk_commit_range` in revert.rs is dead code (defined, never called; only
  compiles silently under the crate-level `#![allow(dead_code)]` in `main.rs`).
  Moving it to lib would warn, so it was left in place.
- The revert sequencer/state-file machinery, dirty-index/clobber preflights,
  editor launch, conflict-hint output, signoff/identity resolution, and the
  commit writer are all CLI shell (state files, `eprintln!`, `std::process::exit`,
  `$EDITOR`/`Command`, `crate::ident`) and stayed.
- `tree_to_index_entries` / `tree_entries_to_map` already have lib equivalents
  in `porcelain::merge`, but the CLI revert copy is entangled with the local
  `checkout_merged_index` worktree-mutation path; collapsing it onto the lib
  versions is a separate, larger change and was left for a follow-up.

## Result
- `grit/src/commands/revert.rs`: -92 / +1 lines (two function clusters removed,
  one `use` added; remainder is rustfmt of the touched import block).
- New `grit-lib/src/porcelain/revert.rs` (~140 lines); registered in
  `grit-lib/src/porcelain/mod.rs` (`pub mod revert;`, alphabetical).

## Verification (byte-exact gate, baselines from data/tests/{t3,t7}/*.toml)
- t3501-revert-cherry-pick: 21/21 (baseline 21, fully_passing) ✓
- t3508-cherry-pick-many-commits: 14/14 (baseline 14, fully_passing) ✓
- t7106-reset-unborn-branch: 7/7 (baseline 7, fully_passing) ✓
- All three TOMLs keep `fully_passing = true`, `expect_failure = 0`.
- `cargo test -p grit-lib --lib`: 289 passed, 2 failed (only the known
  `ignore::gitignore_glob_tests` failures).
- `cargo build --release -p grit-cli`: clean (residual warnings are pre-existing
  in diff.rs/merge.rs/repack.rs, not touched here).
