# Libification Phase 4 (step 2) — move the untracked/ignored walk to lib — 2026-06-08

Moved the self-contained domain core of `git status` out of the CLI into
`grit_lib::porcelain::status`: the untracked/ignored work-tree walk plus the
status pathspec-matching helpers (15 functions, ~540 lines).

## What moved
`collect_untracked_and_ignored` (+ `visit_untracked_node`/`visit_untracked_directory`,
`traditional_normal_directory_only`, `directory_contains_only_dot_git`,
`has_tracked_under`, `relative_path`, `dir_is_nested_submodule_worktree`) and the
pathspec matchers (`status_path_matches`, `status_path_matches_worktree`,
`pathspecs_use_attr_magic`, `worktree_path_mode`, `is_executable_file`,
`pathspec_may_match_directory`, `directory_pathspec_matches_self`). The CLI
`IgnoredMode` enum was replaced by `grit_lib::porcelain::status::IgnoredMode`.

The closure was fully self-contained (only `grit_lib::{pathspec,crlf,ignore,index}`
+ std, no env / tty / trace / clap), so it moved verbatim. Four entry points are
`pub` (`collect_untracked_and_ignored`, `status_path_matches`,
`status_path_matches_worktree`, `dir_is_nested_submodule_worktree`,
`pathspec_may_match_directory`) because surviving CLI code (the formatters,
`run()`, `find_untracked`) still calls them via a `use` import; the rest are
private to the lib module.

What stays in the CLI (correctly): the fsmonitor query, untracked-cache refresh
(mutates the index + writes it back), trace2 emission, and cwd-relative display —
IPC / env / optimization / presentation concerns, not status computation.

## Effect
`grit/src/commands/status.rs`: 4338 → 3796 lines (−542). The walk is now a
reusable library API.

## Verified (byte-identical, no regression)
`cargo build --release -p grit-cli` clean; `cargo test -p grit-lib --lib` green.
Status harness files all fully pass: t7508-status 126/126, t7061-wtstatus-ignore
25/25, t7063-status-untracked-cache 58/58, t7066-status-ignored 32/32,
t9130-status-porcelain-v2 26/26.

## Remaining Phase 4
Step 3: add `status(repo, &StatusOptions, &mut ProgressSink) -> StatusModel`
assembling the model, and convert the three formatters to take `&StatusModel`,
shrinking `run()` to options→status()→render.
