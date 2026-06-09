# t3905 stash include untracked

## 2026-06-05

- Claimed ticket `34f9ec`.
- Target test: `tests/t3905-stash-include-untracked.sh`.
- Ticket snapshot says 31/34 pass; remaining failures are around `stash show` handling of
  `--include-untracked`, `--only-untracked`, and `--no-include-untracked`.
- Fresh harness reproduction: 31/34.
- First direct failure was in `stash show -p --include-untracked`: the tracked empty-file add
  printed an `index ... 100644` suffix and `---`/`+++` headers, while Git prints only the diff
  header, `new file mode`, and bare `index` line for empty added files in this path.
- Patch: adjust the stash tree-diff formatter for added/deleted files to omit the index mode suffix
  and skip the file-header/body patch when the blob is empty.
- Harness improved to 32/34.
- Next direct failure was `--only-untracked --no-include-untracked`: tracked-only stat output was
  empty for staged-only changes because it diffed the stash index parent against the WIP tree.
- Patch: make tracked-only `stash show` stat compare the HEAD parent tree against the WIP tree.
- Final targeted harness: `t3905-stash-include-untracked.sh` passed 34/34.
- Related incomplete harness: `t3903-stash.sh` improved to 110/142; its TOML was left out because
  that ticket remains open.
- Validation:
  - `cargo fmt` passed.
  - `cargo check -p grit-cli` passed with the existing `diff.rs` `ext_total` warning.
  - `cargo clippy --fix --allow-dirty` exited 0 with the repository's existing warning backlog and
    failed unrelated autofix report.
  - `cargo test -p grit-lib --lib` reported the known baseline: 252 passed, 2 ignore glob tests
    failed.
