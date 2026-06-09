# Phase 2 — test-tool scaffolding behind a default-ON `test-tools` cargo feature

## Goal

Let the product binary be built **without** the `test-tool` scaffolding while
normal builds and the test harness keep the full surface (the harness invokes
75 distinct `test-tool <sub>` commands).

## Cargo features

- `grit/Cargo.toml`: added `[features]` with `default = ["test-tools"]` and
  `test-tools = ["grit-lib/test-tools"]`. `grit-cli` had no prior features, so
  `--no-default-features` drops only the test-tools-adjacent code.
- `grit-lib/Cargo.toml`: added `[features]` with `test-tools = []` (empty by
  default; enabled transitively by `grit-cli`'s default feature).

## Code gated behind `#[cfg(feature = "test-tools")]`

- `grit-lib/src/lib.rs`: `parse_options_test_tool` and `test_tool_progress`
  module decls (both referenced only from main.rs's test-tool dispatch).
- `grit/src/commands/mod.rs`: `test_tool_reach`, `test_tool_ref_store`,
  `test_tool_rot13_filter` module decls.
- `grit/src/main.rs`:
  - mod decls `bundle_uri_test_tool`, `test_tool_pack_deltas`,
    `test_tool_run_command`.
  - the entire `"test-tool" => { ... }` dispatch match arm (so with the feature
    off, `grit test-tool` falls through to the unknown-command `_` arm).
  - all `run_test_tool_*` fns, the `TEST_TOOL_EXAMPLE_TAP_OUTPUT` const,
    `test_tool_usage`, `preprocess_test_tool_args`, and every helper used
    *only* by test-tool code: `run_ref_store_rename`, `dir_iterator_error_name`,
    `walk_dir_iterator`, `parse_ulong_str`, `collect_custom_userdiff_drivers`,
    `BUILTIN_USERDIFF_DRIVERS`, `parse_find_pack_count_arg`,
    `display_find_pack_path`, the lazy-init-name-hash timing/analyze helpers,
    the json-writer helpers + `JsonWriter*` enums, the `bloom_*` helpers +
    `BloomSettings`/`TEST_BLOOM_SETTINGS`, `normalize_path_simple`,
    `posix_basename`, `posix_dirname`, the gitmodules-for-test-tool helpers,
    and the `test_tool_config_*` / `test_tool_git_config_parse_key` /
    `test_tool_parse_git_bool_strict` config helpers + `TestToolConfigParseKeyErr`.

`pkt_line` is **not** gated (it re-exports `grit_lib::pkt_line` used by real
transport). `parse_bool_str` is **not** gated — it is shared with non-test-tool
code (env-helper early load, config parsing). All edits are additive (attribute
insertions only); `git diff --numstat` shows insertions and zero deletions, so
the moves are faithful.

## Verification

(a) Default build (`test-tools` ON):
- `cargo build --release -p grit-cli -j 4` — clean (pre-existing warnings only).
- `grit test-tool getcwd` works (exit 0).
- Harness (isolated `--data-dir`, no regression vs baseline TOMLs):
  - `t1405-main-ref-store` 16/16 (baseline 16)
  - `t0061-run-command` 24/24 (baseline 24)
  - `t0040-parse-options` 94/94 (baseline 94)

(b) Product build (`--no-default-features`, test-tools OFF):
- `cargo build --release -p grit-cli --no-default-features -j 4` — compiles
  (warnings only: now-unused imports `Context`, `grit_lib::git_path`, `Read`).
- `grit test-tool getcwd` → `git: 'test-tool' is not a git command.` (exit 1):
  no test-tool scaffolding present.

The committed state is the **default** build (test-tools ON) so the harness keeps
working.
