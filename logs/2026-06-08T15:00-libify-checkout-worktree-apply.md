# Libify: checkout worktree-apply primitives → grit_lib::porcelain::checkout

## Target
`grit/src/commands/checkout.rs` (8,259 lines) — first worktree mutator.

The whole tree-to-worktree application core is far too entangled to move
byte-exact in one pass (it threads clap-derived `CheckoutMergeCli`, hook
dispatch, smudge/filter context, promisor lazy-fetch, and branch-switch
messaging through dozens of mutually-recursive functions). Per the recipe,
extracted a **self-contained sub-core** instead: the pure worktree-apply
primitives the shell calls into.

## Moved to `grit-lib/src/porcelain/checkout.rs` (new module)
- `apply_index_file_mode` — set file perms from index mode.
- `prepare_parent_dirs_for_checkout` — replace symlink/file parents with dirs.
- `write_to_worktree` — write blob bytes (symlink / exec-bit / parent-dir aware).
- `remove_empty_parent_dirs` — prune now-empty parents up to the work tree.
- `is_glob_pattern` / `glob_matches` / `glob_matches_inner` — simple glob matcher
  used by interactive-patch path filters.

These are pure-domain: no clap, no println/eprintln/color, no `crate::`
(CLI-internal) references. They compute and apply worktree changes from
index/object data. `current_dir()` use (in `remove_empty_parent_dirs`) is a
runtime cwd-safety query, with ample precedent in 15 existing lib modules.

## Error-type bridge
The CLI functions used `anyhow::Result` + `.with_context()`; the lib uses
`crate::error::{Error, Result}`. Converted the moved fns to return the lib
`Result`, preserving the exact context message strings via `Error::PathError`.
None of these messages are asserted by the harness (they only surface on write
failure, which passing tests never hit). The lib `Error` implements
`std::error::Error`, so `?` from CLI call sites into `anyhow::Result` is
automatic; two trailing-expression `return write_to_worktree(...)` sites in
`checkout_conflicted_path_with_merge` became `write_to_worktree(...)?; Ok(())`.

## CLI changes (`grit/src/commands/checkout.rs`)
- Deleted the 7 moved definitions (~204 lines).
- Added `use grit_lib::porcelain::checkout::{apply_index_file_mode, glob_matches,
  is_glob_pattern, remove_empty_parent_dirs, write_to_worktree};` so the bare
  call sites resolve. (`prepare_parent_dirs_for_checkout` is now only used
  internally by `write_to_worktree`, so it is not re-imported.)

## Verification (byte-exact gate)
Baselines (from data/tests TOMLs) → after:
- t2020-checkout-detach  26/26 → 26/26 (fully_passing)
- t2022-checkout-paths    5/5  → 5/5  (fully_passing)
- t2007-checkout-symlink  4/4  → 4/4  (fully_passing)
- t7201-co               46/46 → 46/46 (fully_passing)

`cargo build --release -p grit-cli` clean (no new warnings in my files).
`cargo build --release -p grit-simple` clean (thin-CLI canary).
`cargo test -p grit-lib --lib`: 289 passed, only the 2 known
`ignore::gitignore_glob` failures.
