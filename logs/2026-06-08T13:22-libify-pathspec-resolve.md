# Libification — pathspec resolution → grit_lib::pathspec — 2026-06-08

Moved the pure pathspec-resolution helpers out of `grit/src/pathspec.rs` into the
existing `grit_lib::pathspec` module (~297 lines appended before its test module).

## What moved
`resolve_pathspec`, `resolve_magic_pathspec` (+ private `resolve_magic_pathspec_parts`,
`has_magic_prefix_token`, `inject_magic_prefix_token`, `normalize_relative_path_str`,
`prepend_cwd_to_short_exclude_pathspec`), `pathdiff`, `resolve_pathspec_in_worktree`,
`normalize_worktree_file_path`, and the `PathOutsideRepository` error type
(struct + `Display` impl). These take an explicit `work_tree`/`prefix` and depend
only on `grit_lib::pathspec` (the two intra-module references
`literal_pathspecs_enabled` and `pathspec_is_exclude` already lived there, so the
`grit_lib::pathspec::` qualifiers became bare names — the only logic-line change).

## What stays in the CLI
`grit/src/pathspec.rs` is now a thin shell: a `pub use grit_lib::pathspec::{…}`
re-export for the five moved fns + `PathOutsideRepository`, plus the CLI-local
short-magic parser `PathspecMagic` (struct) + `parse_magic` — kept in the CLI
because they collide with the lib's own private `PathspecMagic` and are used only
by `git clean` (`commands/clean.rs`). The ~32 `crate::pathspec::*` call sites
across add/blame/clean/commit/diff/grep/log/ls_files/reset/rm/stash/update_index
keep working unchanged via the re-export.

## Verified
Every moved item is byte-identical to the committed CLI version (confirmed by
diffing each function body) except the two expected qualifier transforms above.
`cargo build --release -p grit-cli` clean — no new warnings (the 3 pre-existing
CLI warnings and 1 pre-existing grit-lib warning are unrelated). `cargo test
-p grit-lib --lib`: 284 passed, only the 2 known `ignore::gitignore_glob`
failures. Harness byte-exact vs baseline: t4010-diff-pathspec 17/17,
t6130-pathspec-noglob 21/21, t6132-pathspec-exclude 31/31,
t3705-add-sparse-checkout 20/20, t7508-status 126/126 — all fully_passing, no
regression. Smoke (status/add/diff/log with pathspecs + the
`PathOutsideRepository` fatal) renders identically.
