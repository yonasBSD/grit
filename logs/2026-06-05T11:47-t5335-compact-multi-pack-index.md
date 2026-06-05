# t5335-compact-multi-pack-index — MIDX compaction

Ticket: c8e5f7. Goal: make `tests/t5335-compact-multi-pack-index.sh` pass.

## Result
1/10 -> 10/10.

## What was missing
1. `git multi-pack-index compact <from> <to>` was a stub that just rewrote the
   whole MIDX, ignoring the endpoints. It now performs real incremental
   compaction.
2. `test-tool read-midx <object-dir> <checksum>` and
   `test-tool read-midx --show-objects <object-dir> <checksum>` ignored the
   trailing checksum and always read the chain tip. They now read the specific
   layer named by the checksum.
3. Incremental MIDX layers did not filter objects already present in a base layer
   (only base *packs* were filtered). t5335 test 8 builds a fresh pack with all
   objects via `--revs`, so OID-level filtering is required.

## Changes
- `grit-lib/src/midx.rs`:
  - `build_midx_bytes` -> `build_midx_bytes_filtered` with an `exclude_oids`
    parameter; the incremental write path now passes the base OIDs.
  - `compact_multi_pack_index(pack_dir, from, to, write_bitmaps, write_rev, version)`
    plus a `CompactError` enum mapping to git's exact diagnostics. It merges the
    inclusive chain range `[from..to]` (oldest->newest, `from`=argv[0]) into one
    new layer, preserving pack order (no lexical sort), excluding base OIDs, and
    rewrites the chain as `[base] + [compacted] + [upper]`. Upper layers keep
    their files/checksums because grit's layers are self-contained.
  - `resolve_midx_layer_path` and `format_midx_dump_layer` /
    `format_midx_show_objects_layer` for per-layer reads by checksum. The dump
    now reads pack names from the resolved layer file, not the tip.
- `grit/src/commands/multi_pack_index.rs`: `CompactArgs` gains
  `--incremental/--bitmap` and two positional endpoints; `cmd_compact` calls the
  new lib function and prefixes errors with `fatal: ` (git `die()` semantics, the
  top-level handler strips a leading `fatal:` and exits 128).
- `grit/src/main.rs`: `read-midx` passes the optional checksum to the new
  per-layer formatters; missing-checksum prints `error: could not find MIDX with
  checksum <hash>` and exits 1.

## Notes / non-regressions
- grit's chain layers are self-contained (each `.midx` lists only its own packs,
  byte7/num_packs_in_base == 0), so compaction did not need to rewrite upper
  layers or store base counts — much simpler than git's format.
- t5334 test 3 ("convert incremental to non-incremental") was ALREADY failing on
  the committed binary (verified by stashing my files and rebuilding); its TOML
  `passed_last=16` was stale. Not a regression from this work. The real bug there
  is grit's non-incremental write `remove_dir_all`ing `multi-pack-index.d/` while
  git keeps the empty directory — left for its own ticket.
- The shared build was transiently broken by the other agent's in-flight
  `fetch.rs`/`pull.rs` `no_all` field; grit-lib compiles clean in isolation.
