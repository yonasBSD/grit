# t5324-split-commit-graph — work log

Ticket: fba897. Subsystem: pack-storage (commit-graph machinery).

## Starting state
17/42 passing.

## Root causes found
1. **No split merge strategy.** `commit-graph write --split` never merged base
   layers into the new tip (split_graph_merge_strategy / merge_commit_graphs in
   commit-graph.c). Implemented size-multiple + max-commits + no-merge logic in
   `grit/src/commands/commit_graph.rs::cmd_write`.
2. **Generation-data chunk always written.** Added `write_generation_data`
   parameter to `build_commit_graph_bytes` and conditional GDA2/GDO2 emission.
   Driven by `commitGraph.generationVersion` and the topmost kept base layer.
3. **read-graph read the base layer, not the tip.** `test-tool read-graph` reads
   the last (tip) line of the chain file, not the first. Fixed in main.rs.
4. **Local clone didn't copy `info/commit-graphs`.** `copy_objects` in clone.rs
   skipped the commit-graphs subdir; now recurses into it (real local clone
   copies the whole objects tree). Unblocks all the `git clone . X` verify tests.
5. **Verify was single-file only.** Rewrote `cmd_verify` to load+validate the
   split chain: chain-file size, hash-line validity, missing-layer detection,
   BASE-chunk match, per-layer checksum, ODB cross-check, --shallow + progress.

## Helpers added
- `CommitGraphChain`: `num_layers`, `layer_commit_counts_tip_first`,
  `layer_has_generation_data_tip_first`, `layer_hashes_tip_first`,
  `layer_object_dirs_tip_first`, `layer_oids`, `sub_chain_tip_first`.
- `cmd_write`: `hex_to_hash20`, `parse_expire_time`, `parse_tz_offset`.

## After first commit: 26/42 passing.

## Remaining failures (to diagnose)
13, 15, 17, 18, 19, 22, 25, 26, 31, 33, 34, 37, 38, 39, 40, 42.
