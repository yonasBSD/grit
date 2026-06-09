# Libify: merge octopus-reduce + fast-forward index compose → grit_lib::porcelain::merge

## Target
`grit/src/commands/merge.rs` (13,176 lines). The full merge sequencer
(`run`, `do_real_merge`, `do_octopus_merge`, `merge_trees`) is deeply entangled
with CLI concerns — argv/clap parsing, `println!`/`eprintln!` progress
("Trying merge strategy ...", "Already up to date."), the merge-message editor
launch, hook dispatch, `ExplicitExit`/exit-code mapping, and worktree writes via
`checkout_entries`. Per the target's escape hatch ("if the whole sequencer is
too entangled ... extract the cleanest self-contained sub-core (octopus
reduction + ff compose) and DEFER the rest in place"), I extracted exactly that
self-contained, presentation-free algorithm core and left the sequencer in the
CLI. The `MergeOutcome` enum / strategy-trial loop are **deferred** — they own
mutable repo+index+worktree state plus interleaved prints/editor/hooks and are
not cleanly separable in one byte-exact slice.

## Moved to new `grit-lib/src/porcelain/merge.rs`
- `tree_to_index_entries` (pub) — flatten a tree object into a recursive
  `Vec<IndexEntry>` (stage 0, zeroed stat fields). The shared building block the
  rest of the cluster (and ~30 CLI merge call sites) are written against.
- `tree_to_map` (pub) — index a flat entry list by path, last-wins.
- `compose_fast_forward_index` (pub) — build the post-fast-forward index from
  the target tree, carrying forward staged additions absent from both target and
  HEAD trees; dedups duplicate-path trees (t4058).
- `compose_octopus_final_index` (pub) — fold pre-octopus staged paths the merge
  result doesn't touch back into the final index.
- `reduce_octopus_merge_heads` (pub) — Git's "reduce parents": drop any merge
  head that is an ancestor of another listed head, preserving input order (t7603).

These are pure-domain: they touch only `Index`/`IndexEntry`, `ObjectId`,
`ObjectKind`, `parse_tree`, `Repository` (read-only `odb`), and
`merge_base::is_ancestor` — all already in lib. No clap, no
`println!`/`eprintln!`/color, no env/tty, no `crate::` (CLI-internal) refs, no
`ExplicitExit`. Added `pub mod merge;` to `grit-lib/src/porcelain/mod.rs`
(alphabetically after `log`).

## Error-type bridge (byte-exact equivalence)
The CLI's `tree_to_index_entries` returned `anyhow::Result` and used
`bail!("expected tree, got {}", obj.kind)`. The lib version returns
`crate::error::Result` and produces
`Error::Message(format!("expected tree, got {}", obj.kind))` — identical text
(`ObjectKind`'s `Display` yields the same string). The only call site that
inspected the error type, `tree_to_index_entries_for_merge_tree`
(merge.rs:~11621), previously did
`e.downcast_ref::<grit_lib::error::Error>()` on an `anyhow::Error`; it now
matches `grit_lib::error::Error::ObjectNotFound(hex)` directly on the returned
lib error and falls through with `e.into()` (lib `Error` → `anyhow::Error`).
Observable behavior — the merge-tree "Could not read <oid>" path and every other
error — is unchanged.

## CLI changes (`grit/src/commands/merge.rs`)
- Deleted the 5 functions above (~138 lines).
- Added `use grit_lib::porcelain::merge::{compose_fast_forward_index,
  compose_octopus_final_index, reduce_octopus_merge_heads, tree_to_index_entries,
  tree_to_map};` so the ~35 bare-name call sites resolve unchanged.
- Removed now-unused `parse_tree` from the `grit_lib::objects` import.
- Rewrote the `tree_to_index_entries_for_merge_tree` error mapping per the bridge
  above.

## Verification (byte-exact gate)
Target harness baselines (from data/tests TOMLs) → after:
- t7600-merge               83/83 → 83/83 (fully_passing)
- t7601-merge-pull-config   65/65 → 65/65 (fully_passing)
- t7602-merge-octopus-many   5/5  → 5/5   (fully_passing)
- t7607-merge-state          1/1  → 1/1   (fully_passing)
- t6402-merge-rename        46/46 → 46/46 (fully_passing)

All TOMLs byte-identical to HEAD after the harness rewrote them (no diff).
`cargo build --release -p grit-cli` clean (no new warnings in my files; the 3
remaining warnings — merge.rs:4013 `push_batch`, diff.rs:7561 `ext_total`,
repack.rs:1559 `push` — are pre-existing ambient noise in untouched code).
`cargo test -p grit-lib --lib`: 289 passed, only the 2 known
`ignore::gitignore_glob` failures.

## Deferred (left in place, byte-exact, in the CLI)
The `MergeOutcome` enum (AlreadyUpToDate/FastForward/Merged/Conflicts/Stopped),
the strategy-trial loop (`try_merge_strategies`), `do_real_merge`,
`do_octopus_merge`, and `merge_trees` — these carry the editor launch, progress
prints, hook dispatch, and mutable worktree/index/state writes interleaved with
the algorithm, and could not be sliced into a result-returning core in one
byte-exact move. A future pass can inject `ProgressSink`/`HookRunner` and split
the ff/up-to-date path first, as the plan's Phase 7 outlines.
