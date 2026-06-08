# libify: log — ref-decoration data into grit_lib::porcelain::log

## Target

`grit/src/commands/log.rs` (15,011 lines). Destination: new
`grit_lib::porcelain::log`.

## What moved

`log.rs` is enormous and the rev-walk/pretty-format/pager/graph machinery is
deeply entangled with the clap `Args` struct (58 `&Args` sites), ANSI colour, and
stdout. A full `log_entries()` extraction was not safely achievable in one
byte-exact step. Instead I extracted the one genuinely **colour-free, structured**
slice: the `--decorate` / `%d` ref-decoration model.

Moved to `grit-lib/src/porcelain/log.rs` (transform `grit_lib::` → `crate::`):

- `DecorationKind` (enum), `DecorationItem` (struct), `DecorationMap` (type alias)
- `DecorationFilter` (struct + `is_empty`/`matches` impl)
- `normalize_glob_ref`, `decoration_pattern_matches` (private)
- `replace_ref_base`, `prepend_decoration`, `peel_to_commit_hex` (helpers)
- `collect_decorations`, `collect_decorations_inner` (the decoration walk)
- `current_branch_decoration_index` (`HEAD -> branch` fold predicate)

Made `pub` exactly what the CLI still calls, including the `DecorationItem`
fields (`refname`/`display`/`kind`) and `DecorationFilter` fields
(`include`/`exclude`/`exclude_config`) that the surviving CLI `build_decoration_filter`
constructs and the formatters read.

## What stayed in the CLI (correctly)

`DecorationPaint`, `load_decoration_paint`, `color_for_decoration_kind` (ANSI),
`build_decoration_filter` / `ordered_decorate_ref_patterns` /
`decorations_initial_set_all` (read clap `Args` + `eprintln!`), and all
`format_decoration*` / `oneline_decoration_for_hex` formatters. The rev-walk,
pretty-format engine, graph drawing, pager, and `Args` parsing remain in
`commands/log.rs`. The `rev_list::render_commit_with_color` colour leak was not
touched or extended.

## CLI wiring

Added `use grit_lib::porcelain::log::{collect_decorations, collect_decorations_inner,
current_branch_decoration_index, normalize_glob_ref, DecorationFilter,
DecorationKind, DecorationMap};` so the bare-name call sites resolve. Net
`commands/log.rs`: +4 / −349.

## Verification (byte-exact gate)

`cargo build --release -p grit-cli` clean (no warnings in my files).
`cargo test -p grit-lib --lib`: 289 passed, only the 2 known
`ignore::gitignore_glob` failures.

Harness (per-file passed/total), before → after:

| file | baseline (TOML) | actual original tree | after my change |
|------|-----------------|----------------------|-----------------|
| t4202-log | 146/149 | 146/149 | 146/149 |
| t4203-mailmap | 74/74 | — | 74/74 |
| t4205-log-pretty-formats | 123/123 (stale) | 122/125 | 122/125 |
| t4207-log-decoration | 23/23 | — | 23/23 |
| t4207-log-decoration-colors | 4/4 | — | 4/4 |

The recorded t4205 TOML baseline (123) was stale: building the **unmodified** tree
reproduces 122/125 exactly, so my change causes zero regression (122 → 122). No
file regressed; no `fully_passing` flipped true→false because of this change
(t4205 was already false on the unmodified tree).
