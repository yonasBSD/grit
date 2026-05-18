# 2026-05-18 — Phase 1.1 worktree library

## Goal

Move worktree discovery/listing from `grit/src/commands/worktree.rs` into `grit-lib`.

## Work

- Add `grit-lib/src/worktree.rs`: registry scan, `WorktreeEntry`, linked HEAD resolution.
- Thin CLI `worktree list` to call the library API.
- Validate with `cargo test -p grit-lib --lib` and `t2402-worktree-list.sh`.

## Follow-up (same day)

- `t2402-worktree-list` flipped to `in_scope=yes` in `data/test-files.csv`.
- Fixed `rev-parse --git-path objects` for linked worktrees: use Git
  `DEFAULT_RELATIVE_IF_SHARED` (absolute path when no `--prefix`).
- Harness: **26/27** (`rev-parse --git-path` passes); remaining: broken-HEAD
  porcelain list (`test 24`).

## t2402 full pass

- `resolve_head`: symref to non-`refs/heads/*` target that does not resolve →
  `HeadState::Invalid` (Git `worktree list` `(error)` + porcelain `ZERO_OID`).
- `test-tool ref-store main create-symref` wired in `grit/src/main.rs`.
- `./scripts/run-tests.sh t2402-worktree-list.sh`: **27/27**.
