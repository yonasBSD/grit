# libify: extract `git apply` patch-parse core into `grit_lib::apply`

Phase 5 (medium commands). Target: `grit/src/commands/apply.rs` (6476 lines).

## What moved

The self-contained **parse core** — unified/`git`-diff text → structured
`FilePatch`/`Hunk` data — moved to a new `grit-lib/src/apply.rs`
(`pub mod apply;` added alphabetically in `lib.rs`). This layer has no I/O, no
environment access, and no CLI dependencies.

Moved (≈1.4k lines):
- Data types: `Hunk`, `HunkLine`, `FilePatch` (+ `impl`), `BinaryPatchPayload`.
- `parse_patch`, `parse_hunk`, `parse_hunk_header`, `parse_range`.
- `parse_binary_patch`/`parse_binary_literal`/`decode_binary_patch_line`/
  `decode_binary_line_len`/`inflate_binary_payload`.
- All path/name/timestamp helpers (`split_diff_git_paths`, `git_header_def_name`,
  `find_name_*`, `skip_tree_prefix_*`, `unquote_c_style_diff_prefix`,
  `diff_timestamp_len`/`has_epoch_timestamp`, `parse_traditional_patch_pair`, …).

Public surface (what the CLI engine still calls, so it is `pub`):
`parse_patch`, `inflate_binary_payload`, and the four types with `pub` fields
(`Hunk`/`HunkLine`/`BinaryPatchPayload` are constructed by the surviving
reverse/ws engine; `FilePatch` methods are read everywhere).

## Stayed in CLI (`grit/src/commands/apply.rs`)

clap `Args`, all config lookups, `ApplyWhitespaceMode`/`WsCliMode`, the
worktree/index application engine (`apply_hunks`, `apply_to_worktree`,
`apply_to_index`, three-way merge, reject files, stat/numstat/summary
rendering), `ExplicitExit`, `std::env`, and `apply_setup_prefix()`.

## Key boundary decision

`parse_patch` previously called `apply_setup_prefix()` internally
(`Repository::discover` + `std::env::current_dir`). That environment concern is
now a `setup_prefix: Option<&str>` parameter the CLI computes and passes in,
keeping the lib pure.

## Error fidelity

`grit-lib` does not depend on `anyhow`, so `bail!`/`anyhow!`/`.context()` became
`crate::error::Error::Message(..)`. Context chains were folded into the message
to reproduce anyhow's `{:#}` rendering exactly (the CLI's `main` prints
`error: {e:#}`). Verified against the asserted texts
`error: corrupt patch at patch:4` (t4100) and `invalid digit found in string`.

Four genuinely-dead helpers (`header_line_file_path` cluster) were dropped
rather than carried into lib (the CLI hid them under crate-level
`#![allow(dead_code)]`; lib has no such allow).

## Verification (byte-exact)

Ran all 41 `t41xx` apply harness files before and after; pass counts are
**identical** for every file. Spot-checked pre vs post on a stashed pre-change
binary for the partial-fail files — byte-for-byte equal (e.g. t4124 70/85,
t4137 8/28, t4108 8/18, t4103 5/24, t4100 0/25, t4120 6/12 all unchanged).

`cargo test -p grit-lib --lib`: 289 passed, 2 failed (only the pre-existing
`ignore::gitignore_glob` ignores). Added 5 unit tests under `apply::tests`.
`cargo build --release -p grit-cli` clean (only 3 pre-existing warnings in
diff/merge/repack, none in my files).
