# Libification Phase 4 (step 3) — the status() operation — 2026-06-08

Added `grit_lib::porcelain::status::status()`, the clean library entry point that
computes a [`StatusModel`] for a repository — completing `status` as a structured
library operation.

## What it does
`status(repo, &StatusOptions, &mut dyn ProgressSink) -> Result<StatusModel>`:
load + sparse-expand the index, resolve HEAD and in-progress state, compute the
staged (index-vs-HEAD) and unstaged (index-vs-worktree) diffs with optional
rename/copy detection (`apply_status_renames`, mirroring git's candidate-count
guards), walk the work tree for untracked/ignored via the step-2 lib walk, and
count stash entries. Reports through the `ProgressSink`; never touches the
terminal.

The CLI's performance/diagnostic layers — fsmonitor query, untracked cache,
trace2 — are deliberately **not** in `status()`; they remain in `grit`'s `run()`
wrapper. A library consumer (a GUI, a server, `grit-simple`) that just wants a
repo's status now calls one function.

## Verified
- Two integration tests build a real minimal repo (tempdir + `.git`) and assert
  the model: untracked detection on an unborn branch, and `untracked=No` skipping
  the walk. Both pass.
- `cargo build --release -p grit-cli` clean; `cargo test -p grit-lib --lib`
  281 passed (the 2 new tests included), only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures remain.

## Phase 4 status
The library status operation is complete: `StatusOptions`, `StatusModel`, the
untracked/ignored walk + pathspec matching, and `status()`. **Deferred (optional,
CLI-internal tidy, not libification):** rewiring `grit`'s `run()` to call
`status()` and converting the three formatters to take `&StatusModel`. That was
left out on purpose — the formatters stay CLI presentation either way, and
delegating `run()`'s computation risks byte-exact output regressions because of
the fsmonitor/untracked-cache entanglement, for little libification gain.
