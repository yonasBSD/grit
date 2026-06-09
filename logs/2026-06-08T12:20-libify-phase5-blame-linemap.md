# Libification Phase 5 — blame line-mapping → grit_lib::blame — 2026-06-08

Extracted the pure line-mapping/similarity algorithm out of `grit/src/commands/blame.rs`
into a new `grit_lib::blame` module (~430 lines).

## What moved
`BlameDiffAlgorithm` (+ `to_similar`), `parse_diff_algorithm_name`,
`should_drop_tail_match_for_myers`, `build_line_map` (exact map via the configured
diff algorithm honoring the indent heuristic), `build_fuzzy_line_map` +
`fuzzy_match_segment` + `line_similarity_and_lcs` (fuzzy fallback for rewritten
lines), and the `BLAME_INDENT_HEURISTIC` toggle (`set_blame_indent_heuristic`).
These are pure functions over `&[&str]` line slices — strings in, line mappings
out, no repository/IO. Only dependency is `grit_lib::diff_indent_heuristic` + the
`similar` crate.

The per-commit blame walk (`compute_blame`), the `BlameLine`/`TrackedLine` data,
the textconv/diff-attribute handling, and all output formatting stay in the CLI;
`compute_blame` now calls the lib via a `use grit_lib::blame::{…}` import.

## Why this slice
The full blame engine (~1,635 lines) has a wide CLI/engine boundary (the CLI calls
21 engine internals directly). The line-mapping algorithm is the cleanest
self-contained, reusable piece with a crisp boundary — extracted first, following
the Phase-4 discipline of moving a verified core rather than a sprawling whole.

## Verified
`cargo build --release -p grit-cli` clean (no warnings); 3 new `grit_lib::blame`
unit tests pass; blame harness files all fully pass: t8002-blame 135/135,
t8003-blame-corner-cases 30/30, t8011-blame-split-file 10/10, t8012-blame-colors
120/120. Byte-identical, no regression.

## Remaining
The rest of the blame engine (`compute_blame` + commit-walk helpers + textconv)
can follow into `grit_lib::blame` later; it needs a wider pub API or moving the
CLI's `build_uncommitted_blame` too.
