# Libify: diff orderfile + rotate/skip reordering → grit_lib::diff

## Target
`grit/src/commands/diff.rs` (11,465 lines). The diff compute core already lives
in `grit_lib::diff`; the CLI keeps colored unified-diff emission, pager, ANSI,
column layout, and argv flag parsing. Per the recipe ("small if anything"),
extracted the one self-contained pure-domain cluster that remained: the
`-O<orderfile>` and `--rotate-to`/`--skip-to` entry-reordering family.

## Moved to `grit-lib/src/diff.rs` (appended to the existing module)
- `apply_orderfile_entries` (pub) — sort `DiffEntry`s by first matching orderfile
  glob pattern; unmatched last (stable).
- `apply_orderfile` (private) — internal helper behind the above.
- `read_orderfile_patterns` (pub) — read non-empty/non-comment glob patterns from
  a `-O` file, resolving relative paths against `cwd`.
- `apply_rotate_skip_entries` (pub) — `git diff` rotate/skip over changed paths.
- `apply_rotate_skip_log_entries` (pub) — `git log` rotate/skip using the commit
  tree's blob order (wraps `crate::merge_diff::all_blob_paths_in_tree_order`).
- `apply_rotate_skip_ordered_paths` (private) — tree-order reorder helper.
- `orderfile_pattern_matches` (pub) + `orderfile_glob_match` (private, renamed
  from the CLI's local `glob_match` to avoid a generic name in the lib API).

These are pure-domain: they only touch `DiffEntry` (already in lib), `Odb`,
`ObjectId`, `Path`, and `crate::merge_diff`. No clap, no println/eprintln/color,
no env, no `crate::` (CLI-internal) references.

## Error-type bridge (byte-exact equivalence)
The CLI functions returned `anyhow::Result` and produced
`ExplicitExit { code: 128, message }` for fatal cases ("could not read orderfile
…", "fatal: No such path '…' in the diff"). The lib versions return
`crate::error::Result` and produce `Error::Message(message)` with the identical
strings. `main.rs` maps both to the same observable behavior: it prints the
message verbatim to stderr and exits 128 (`ExplicitExit` branch at main.rs:127;
`Error::Message` via `verbatim_lib_error_message` at main.rs:134). So output and
exit code are byte-for-byte unchanged. `?` from the CLI call sites into
`anyhow::Result` is automatic (lib `Error` implements `std::error::Error`).

## CLI changes
- `grit/src/commands/diff.rs`: deleted the ~241-line cluster; updated the four
  in-file call sites to `grit_lib::diff::{read_orderfile_patterns,
  orderfile_pattern_matches, apply_orderfile_entries, apply_rotate_skip_entries}`.
  (The combined-diff path keeps using `read_orderfile_patterns` +
  `orderfile_pattern_matches`; the diff `run()` path now calls
  `apply_orderfile_entries` instead of the private `apply_orderfile`.)
- `grit/src/commands/log.rs`: 3 call sites `crate::commands::diff::…` →
  `grit_lib::diff::…` (`apply_orderfile_entries`, `apply_rotate_skip_log_entries`).
- `grit/src/commands/format_patch.rs`: 1 call site `crate::commands::diff::
  apply_orderfile_entries` → `grit_lib::diff::apply_orderfile_entries`.

## Verification (byte-exact gate)
Target harness baselines (from data/tests TOMLs) → after:
- t4013-diff-various    230/230 → 230/230 (fully_passing)
- t4015-diff-whitespace 136/136 → 136/136 (fully_passing)
- t4018-diff-funcname   287/287 → 287/287 (fully_passing)
- t4045-diff-relative    39/39  → 39/39   (fully_passing)

Extra coverage of the moved code (not in the target set):
- t4056-diff-order       23/23  → 23/23   (fully_passing) — orderfile + rotate/skip
- t7800-difftool         95/95  → 95/95   (fully_passing) — rotate-to/skip-to
- t4014-format-patch    215/215 → 215/215 (fully_passing) — apply_orderfile_entries
- t4205-log-pretty-formats 122  → 122     (fully_passing already false; unchanged)

`cargo build --release -p grit-cli` clean (no new warnings in my files).
`cargo test -p grit-lib --lib`: 289 passed, only the 2 known
`ignore::gitignore_glob` failures.
