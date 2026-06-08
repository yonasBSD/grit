# Libification Phase 4 (step 1) — status data model — 2026-06-08

The high-value reference extraction: turning `git status` into a structured
library operation (compute a model, render in the CLI). This first step lands
the **data contract**; the computation move follows.

## Seam identified
`grit/src/commands/status.rs` already has an implicit model: its three
formatters (`format_porcelain_v2`, `format_short`, `format_long`) all consume the
same computed core — HEAD + head tree, the staged (index-vs-HEAD) and unstaged
(index-vs-worktree) `DiffEntry` lists, the untracked and ignored path lists, the
in-progress `WtStatusState`, the sparse-expanded `Index`, and the stash count.
Everything else those formatters take (`args`, `config`, `colopts`, the
relativize/quote closures) is presentation, not model.

## This step
- New `grit-lib/src/porcelain/status.rs`: `StatusOptions` (compute-driving inputs:
  untracked/ignored mode, rename detection, pathspecs, ahead-behind — no
  presentation flags) and `StatusModel` (the computed result above), built from
  existing lib types (`HeadState`, `WtStatusState`, `DiffEntry`, `Index`,
  `ObjectId`). Declared `pub mod status;` in `porcelain/mod.rs`.

## Remaining Phase 4 steps (each harness-gated on the status test files)
1. Add `pub fn status(repo, &StatusOptions, &mut dyn ProgressSink) -> Result<StatusModel>`
   by moving the model-assembly out of `run()` (diffs, untracked/ignored walk,
   rename detection, state, stash, sparse handling). CLI side-effects (untracked
   cache index write, trace2) stay in the CLI wrapper.
2. Convert the three formatters to take `&StatusModel` (+ `&Args`/`&ConfigSet`
   for presentation), shrinking `run()` to: Args+config → `StatusOptions` →
   `status()` → render.
3. Verify no byte-exact regression across t7508/t7525/t9130/t2xxx status files.

Verified: `cargo build --release -p grit-cli` clean; `cargo test -p grit-lib --lib`
green (the model compiles, 1 porcelain test).
