# Libify: stash-apply core → grit_lib::porcelain::stash

## Target
`grit/src/commands/stash.rs` — extract the stash-apply domain-logic core into
`grit_lib::porcelain::stash` (Phase 5, medium command).

## What moved
The stash-*apply* core and the tree-flattening / worktree-mutation primitives it
is built on. Stash *create/push/show/list/export* are largely arg-parsing,
output formatting, and orchestration, so they stayed in the CLI; the apply path
is the one self-contained computation-plus-mutation engine.

Moved into `grit-lib/src/porcelain/stash.rs` (all `pub`, since the surviving CLI
show/create/branch paths still call several of them):

- `FlatTreeEntry` (pub fields) + `flatten_tree_full` — recursive tree flatten
- `add_stage_entry` — push a conflict (non-zero) stage entry
- `worktree_bytes_for_index_mode` — read worktree bytes honoring symlink mode
- `write_regular_file_replacing_symlink`
- `remove_empty_dirs` — prune empty dirs up to (not incl.) the worktree root
- `stash_worktree_change_paths`
- `check_stash_apply_would_overwrite_local_changes`
- `apply_stash` (was `apply_stash_impl`) — the 3-way-merge-on-moved-HEAD apply
  engine returning `bool` had_conflicts; CLI keeps the `Dropped …` / conflict
  messaging and exit codes.

`grit_lib::` paths became `crate::`; `anyhow::{bail,anyhow,Context}` became
`crate::error::{Error::Message, Result}` (the apply functions call only
lib functions that already return `crate::error::Result`). The CLI imports the
five still-referenced items via `use grit_lib::porcelain::stash::{…}` and two
tail-position calls were wrapped `Ok(…?)` to bridge `grit_lib::Error` →
`anyhow::Error`.

## Result
- `grit/src/commands/stash.rs`: -699 / +12 lines.
- New `grit-lib/src/porcelain/stash.rs` (~30 KB); registered in
  `grit-lib/src/porcelain/mod.rs` (mirrors the `status` precedent — porcelain
  submodules register in `porcelain/mod.rs`, not `lib.rs`).

## Verification (byte-exact gate, baselines from data/tests/t3/*.toml)
- t3903-stash: 142/142 (baseline 142) ✓
- t3904-stash-patch: 10/10 (baseline 10) ✓
- t3905-stash-include-untracked: 34/34 (baseline 34) ✓
- t3907-stash-show-config: 10/10 (baseline 10) ✓
- `cargo test -p grit-lib --lib`: 289 passed, 2 failed (only the known
  `ignore::gitignore_glob_tests` failures).
- `cargo build --release -p grit-cli`: clean (residual warnings are pre-existing
  in diff.rs/merge.rs/repack.rs, not touched here).
