# t7063-status-untracked-cache

Ticket: 6e7f3a

## 2026-06-06 06:15

- Claimed ticket after closing t7512.
- Starting from a clean GitButler workspace on `sc-branch-1`.
- Investigating `tests/t7063-status-untracked-cache.sh`, `grit-lib/src/untracked_cache.rs`, and the status/update-index wiring.
- Baseline run: `./scripts/run-tests.sh t7063-status-untracked-cache.sh` reported 26/58 passing.

## 2026-06-06 06:19

- Fixed `status` to persist untracked-cache extension changes after creating, removing, or refreshing UNTR state.
- This moved the harness to 52/58; the remaining failures were sparse-checkout setup and `/done/` tracked `.gitignore` cache OID expectations.
- Fixed `checkout main` while already on `main` to re-run tree checkout when sparse checkout is enabled, so changes to `.git/info/sparse-checkout` are applied.
- Verified the direct sparse reproduction removes `done/.gitignore` and marks it skip-worktree.
- Final run: `./scripts/run-tests.sh t7063-status-untracked-cache.sh` reported 58/58 passing.
- Validation:
  - `cargo fmt --check` passed.
  - `cargo check -p grit-cli` passed with the known `grit/src/commands/diff.rs` `ext_total` warning.
  - `cargo clippy --fix --allow-dirty` completed with the existing warning backlog.
  - `cargo test -p grit-lib --lib` still fails only the two known ignore glob tests.
