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

## t40 (mixed gdat deep clone) — FIXED
Root cause was NOT the merge strategy. It was auto-maintenance: after every `git commit`,
grit runs `maintenance run --auto`, whose commit-graph task triggers when the count of
commits *not yet in the graph* reaches `maintenance.commit-graph.auto` (default 100). The
counter helper `graph_oids` in `grit/src/commands/maintenance.rs` only read the single
`info/commit-graph` file and ignored the split chain — so a repo with a split chain looked
like it had NO graph, every commit was counted, and once the mixed-merge-gdat clone crossed
111 commits during its setup `test_commit`s, a spurious `commit-graph write --split` fired
and rewrote the chain (collapsing it to a bad 2-line [8,103] state). The later explicit
write then produced num_commits 8 instead of 47.

Fix: `graph_oids` now loads `CommitGraphChain::load(objects_dir)` (which covers both the
single file and the split chain) and returns all its OIDs, falling back to the raw single-file
reader only if the chain fails to load. Now only the genuinely-new commits are counted, the
threshold is not crossed, and no spurious auto-write corrupts the chain.

Verified t6500-gc (35/35) and t7900-maintenance (71/72, unchanged — the 1 failure is the
pre-existing 'geometric repacking task', not commit-graph related) for regressions.

## FINAL: 42/42 — fully passing.
