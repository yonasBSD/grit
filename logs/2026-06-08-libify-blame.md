# Libify: blame attribution engine -> grit_lib::blame

## Target
Continue the blame extraction: move the per-commit attribution engine out of the
CLI (`grit/src/commands/blame.rs`) into the existing `grit_lib::blame` module
(line-mapping already lived there).

## What moved
The domain core (~1700 lines) now in `grit-lib/src/blame.rs`:
- `compute_blame` and its full transitive closure: `resolve_path_in_tree_entry`,
  `read_blob_content_for_blame`, `read_object_for_blame`, `find_copy_source_blame`,
  `find_path_by_oid_in_tree`, `collect_tree_file_entries`, `get_commit`,
  `commit_parents_for_blame`, `apply_annotate_huge_graft_fixup`, `load_graft_parents`,
  `peel_to_commit_oid`, `content_lines`, the textconv helpers (`resolve_textconv_command`,
  `diff_attr_pattern_matches`, `run_textconv_command`, `create_temp_textconv_file`,
  `shell_quote`, `load_attr_rules`, `load_diff_attr_rules`, `parse_diff_attr_file`,
  `parse_diff_attr_content`, `is_regular_mode`), and the reverse/overlay/uncommitted
  paths (`build_uncommitted_blame`, `read_commit_lines_for_blame`, `compute_reverse_blame`,
  `apply_final_content_overlay`, `apply_worktree_overlay`, `read_worktree_content_for_blame`).
- Types `BlameLine` (pub, all fields pub), `TrackedLine`, `BlameTextconvContext`
  (pub, `config`/`attrs` fields pub, `new` pub), `DiffAttrRule`, `DiffAttrValue`.

Items the surviving CLI still calls are `pub`: `BlameLine`, `BlameTextconvContext`,
`compute_blame`, `compute_reverse_blame`, `build_uncommitted_blame`,
`apply_annotate_huge_graft_fixup`, `apply_final_content_overlay`, `apply_worktree_overlay`,
`load_graft_parents`, `peel_to_commit_oid`, `read_object_for_blame`.

## Boundary handling
- **Promisor hydration**: `read_object_for_blame`'s on-miss lazy fetch is CLI-coupled
  (`crate::commands::promisor_hydrate` -> transport/trace). The lib cannot reference it.
  Added a CLI-installed hook (`set_promisor_hydrate_hook`, `OnceLock<fn(&Repository, ObjectId)>`)
  mirroring the existing `set_blame_indent_heuristic` pattern. `run()` installs
  `blame_promisor_hydrate` which calls `try_lazy_fetch_promisor_object`.
- **Error model**: grit-lib has no `anyhow` dependency (uses `crate::error::{Error, Result}`).
  Converted the moved code's small anyhow surface (6 `bail!`, 3 `with_context`, 2 `anyhow!`)
  to `crate::error::Error::Message` with byte-identical message text. No target test greps
  these messages; `test_must_fail` cases tolerate the exit code.

## CLI stays
`run()`, arg/clap parsing, all output formatting (`AuthorInfo`, `parse_author_field`,
`format_time`, `blame_summary_unicode`, `BlameMetaEncoding`, `write_porcelain/annotate/default`),
color (`BlameColorStyle`, `apply_blame_color_config`, `parse_blame_highlight_recent`,
`age_color_for_timestamp`), `-L` line-range parsing, progress-to-stderr, exit codes.

## Verification (byte-exact gate)
Per-file harness, before -> after (all fully_passing, no regression):
- t8001-annotate 117 -> 117
- t8002-blame 135 -> 135
- t8003-blame-corner-cases 30 -> 30
- t8011-blame-split-file 10 -> 10
- t8012-blame-colors 120 -> 120
Also re-ran the rest of the blame engine surface, all unchanged:
t8004 3, t8005 5, t8006 16, t8008 5, t8009 2, t8013 19, t8014 16, t8015 7,
t8016 5, t8017 4, t8018 6.
`cargo test -p grit-lib --lib`: 284 passed, 2 failed (known ignore::gitignore_glob only).
`cargo build --release -p grit-cli`: clean (no new warnings in changed files).
