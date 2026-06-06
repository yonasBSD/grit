# t5324-split-commit-graph — mop-up round 1 (ticket fba897)

## Starting state
Prior agent reported 39/42 (failing 13, 25, 40), but a fresh run showed 29/42 — a
regression introduced by the shared `target/release/grit` binary being swapped by another
agent mid-effort. Rebuilt and confirmed the genuine failures.

## Root cause of the 29→39 regression (the big find)
Tests 15–26 each `git clone . <dir>` (local clone, hardlinked objects tree). grit's local
clone faithfully **hardlinks** `info/commit-graphs/commit-graph-chain` (and the layer
`*.graph` files), matching Git's `copy_or_link_directory`. The commit-graph writer then
rewrote the chain with `fs::write(&chain_path, ...)`, which truncates the existing file
**in place** — mutating the shared inode and corrupting the clone *source's*
`commit-graph-chain`. After test 15's `merge-2` clone wrote a 3rd layer, the main repo's
chain gained a bogus 3rd line whose `.graph` file did not exist there, so every later clone
of `.` verified with `warning: unable to find all commit-graph files`.

## Fix
`grit/src/commands/commit_graph.rs`: added `write_file_atomic(path, contents)` (temp file in
the same dir + `fs::rename` over the target). `rename` creates a fresh inode, so a hardlink
shared with the clone source is left untouched (this is what Git does — it always renames its
lockfile into place). Routed all fixed-name writes through it:
- split chain file write (the corrupting one)
- split layer-file write (defensive; layer names are content-addressed so rarely collide)
- split-migration chain write
- non-split single `info/commit-graph` write (was `File::create` truncate-in-place)

Dropped the now-unused `BufWriter` import (kept `Write` — still used by verify writeln!s).

## Result
39/42 (failing 13, 25, 40) — recovered the prior agent's count. Committed.

## Cross-alternate chain (t13, t14) — FIXED
Added `CommitGraphChain::try_load_across(objects_dir, alt_dirs)` in
`grit-lib/src/commit_graph_file.rs`: loads the split chain owned by the local dir (resolving
its layer files across alternates) or, when the local dir has no graph at all, from an
alternate's *chain file* — but never from an alternate's single non-split `commit-graph`
(Git refuses to base a chain on a plain graph file; t6/t29 enforce this). Wired it into:
- `cmd_write` (`grit/src/commands/commit_graph.rs`): `existing_chain` now loads across the
  alternate, so a fork whose alternate has a 2-layer split chain writes 1 local tip layer and
  a 3-line chain referencing the alternate's base layers.
- log.rs commit-graph validation (`grit/src/commands/log.rs:~5167`): was
  `try_load(local)` which I/O-errored on alternate-resident base layers; now `try_load_across`.

Care taken not to regress t6/t14/t29 (all cross-alternate variants): the local-single vs
alternate-single distinction is the crux.

## Remaining (t40 — deep multi-clone mixed gdat)
- t40: clone chain mixed -> mixed-no-gdat -> mixed-merge-no-gdat -> mixed-merge-gdat. The 5th
  layer split write (`--split --size-multiple 1`) should merge down to a 2-line chain with
  num_commits 47, but grit reports num_commits 8 (only the new layer's commits). The merge
  strategy is not absorbing the right base layers when the chain spans gdat/non-gdat layers
  that were themselves produced across a clone boundary. Needs deeper investigation of the
  merge-strategy commit counting vs the gdat gating.
