# 2026-05-18 — Phase 1.1 worktree library

## Goal

Move worktree discovery/listing from `grit/src/commands/worktree.rs` into `grit-lib`.

## Work

- Add `grit-lib/src/worktree.rs`: registry scan, `WorktreeEntry`, linked HEAD resolution.
- Thin CLI `worktree list` to call the library API.
- Validate with `cargo test -p grit-lib --lib` and `t2402-worktree-list.sh`.
