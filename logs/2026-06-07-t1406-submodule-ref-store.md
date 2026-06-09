# t1406-submodule-ref-store

Ticket: 6c28fb
Date: 2026-06-07

## Starting state
8/15 passing, 7 failing. All 7 failures returned:
`error: test-tool ref-store: unsupported backend (only 'main' and 'worktree:*' are implemented)`
for the `submodule:sub` backend used by `RUN="test-tool ref-store submodule:sub"`.

Failing subtests: for_each_ref(refs/heads/), resolve_ref(main), verify_ref(new-main),
for_each_reflog(), for_each_reflog_ent(), for_each_reflog_ent_reverse(), reflog_exists(HEAD).

## Root cause
`grit/src/main.rs::run_test_tool_ref_store()` routed only `worktree:*` specs to the full
implementation `commands::test_tool_ref_store::run` and bailed with "unsupported backend"
for everything that was not `main`. The full implementation in
`grit/src/commands/test_tool_ref_store.rs` already supported `submodule:<name>` backends
(open_store resolves `.git/modules/<name>` or `<name>/.git`) — the dispatcher just never
reached it for submodule specs.

The 6 `*_not_allowed` / setup subtests that were already passing did so because the full
impl correctly bails (and `test_must_fail` expects failure) — but the failure reason was
the wrong "unsupported backend" message rather than reaching the submodule store; still,
they passed because the test only checks for non-zero exit.

## Fix
One-line dispatcher fix in `grit/src/main.rs`: route `submodule:*` (in addition to
`worktree:*`) to `commands::test_tool_ref_store::run`. Updated the bail message to mention
`submodule:*`. Also updated the now-inaccurate `#![allow(dead_code)]` comment in
`test_tool_ref_store.rs` (the harness IS now dispatched from main; the allow remains because
some helper paths are only exercised by specific subcommands).

## Result
15/15 passing. cargo test -p grit-lib --lib: 276 pass, only the 2 known pre-existing
`ignore::gitignore_glob_tests` failures (not in scope). No clippy warnings on changed files.

## Files changed
- grit/src/main.rs
- grit/src/commands/test_tool_ref_store.rs
- data/tests/t1/t1406-submodule-ref-store.toml
- logs/2026-06-07-t1406-submodule-ref-store.md
