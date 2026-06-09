# t4215-log-skewed-merges — colored graph rendering

Ticket: 11b36f. Subgroup: log-graph (thread B).

## Starting state
9/10 passing. Failing subtest #9 "log --graph with multiple tips and colors".

## Root cause
`grit log --graph --color=always` rendered the graph edge characters with **no
color codes at all**. The `AsciiGraph` port in `grit/src/commands/log.rs` was a
faithful port of upstream `graph.c` *except* it had dropped all column-color
support: `GraphColumn` carried only `oid` (no `color` field), and the
`output_*_line` functions emitted plain chars via `line.push(...)`.

Upstream `graph.c` colors each graph character with the owning column's color
(`graph_line_write_column`): an index into `column_colors` (default ANSI palette,
overridable by `log.graphColors`). The default column color starts at
`column_colors_max - 1` and increments mod `column_colors_max` for every merge
parent / new childless column.

## Fix (all in grit/src/commands/log.rs)
- Added `color: usize` to `GraphColumn` (index into the palette; `>= colors_max`
  means "draw uncolored", matching Git's sentinel).
- Added `colors`/`colors_max`/`default_column_color`/`use_color` to `AsciiGraph`,
  plus `set_colors`, `current_column_color`, `increment_column_color`,
  `find_commit_color`, and `write_column`/`add_char` helpers that mirror
  `graph_line_write_column` and track *visible* width (excluding escape codes).
- `insert_into_new_columns` now assigns each new column `find_commit_color`.
- `update_columns` increments the column color for merges / new childless
  columns (`num_parents > 1 || !is_commit_in_columns`), as in `graph_update_columns`.
- Every output line function (`padding`, `pre_commit`, `commit`, octopus dashes,
  `post_merge`, `collapsing`) now wraps each column char in its color; `next_line`
  pads to width using the visible width rather than byte length.
- Added `load_graph_colors()` to parse the `log.graphColors` config (comma list,
  each via `grit_lib::config::parse_color`, invalid entries skipped) into a
  palette, and wired `graph.set_colors(use_color, palette)` into BOTH graph render
  paths (`run_graph_log` at the real call site, and the `run_log` graph block).

The default ANSI palette (`GRAPH_COLUMN_COLORS_ANSI`) matches `column_colors_ansi`
in color.c (red..cyan, then bold red..bold cyan), with RESET appended.

## Result
- t4215: 10/10 (was 9/10). Verified colored output byte-for-byte against the
  test's expect.colors.
- No regressions in shared graph machinery: t4214-log-graph-octopus 17/17,
  t4205-log-pretty-formats unchanged (only its 2 `test_expect_failure` known
  breakages remain), t4202-log 129/149 (was 128, +1).
- `cargo test -p grit-lib --lib`: only the 2 known `ignore::gitignore_glob_tests`
  failures remain.
