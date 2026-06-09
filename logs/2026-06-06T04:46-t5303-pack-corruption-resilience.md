# t5303-pack-corruption-resilience.sh — MOP-UP ROUND 1

## Status
- Before: 35/36 (test 23 failing)
- After: 36/36 fully passing

## Failing test
Test 23: "... and a redundant pack allows for full recovery too"

The scenario: a pack (282feba) has blob_2 stored as a delta whose delta-base
reference is corrupted. blob_2 and blob_3 (which deltas on blob_2) become
unreadable from that pack. The test then creates a *redundant* pack (b3dc)
holding good copies of blob_1 and blob_2, runs prune-packed, and restores the
corrupt pack's .idx. All three blobs must then be readable: blob_1/blob_2 from
the good redundant pack, blob_3 from the corrupt pack (its delta base blob_2 now
resolvable from b3dc).

## Root cause
`grit_lib::pack::read_object_from_packs` (grit-lib/src/pack.rs) iterated the
local pack indexes and, for the FIRST pack whose index named the OID, returned
`read_object_from_pack(idx, oid)` immediately — even when that returned an
ERROR (corrupt delta base / zlib failure). It never tried the other packs that
also contained the object.

So for blob_2, which lived in BOTH the corrupt pack (282feba) and the good
redundant pack (b3dc), grit hit the corrupt pack first, got an `Error::Zlib`,
and returned it. `cat-file` then surfaced that as
"error: inflate: data stream error (incorrect header check)" / exit 128 — even
though an intact redundant copy existed.

(Diagnosis note: the error message displays a LOOSE object path, which is
misleading — it comes from `cat_file.rs` mapping `LibError::Zlib` to a loose
"unable to unpack header" message. The actual failure was a pack read. Confirmed
by a temporary debug log in odb.rs's loose-read path that never fired.)

## Fix
Made `read_object_from_packs` keep scanning when a pack read fails: on
`ObjectNotFound` it continues; on any other (corruption) error it records the
error and continues to the next pack that names the OID. Only if every pack
fails does it surface the last error. This matches Git, which retries the
remaining object sources before giving up so a redundant intact pack satisfies
the read.

File: grit-lib/src/pack.rs (read_object_from_packs)

## Validation
- t5303-pack-corruption-resilience: 36/36
- No regressions: t5300-pack-object 63/63, t5302-pack-index 36/36,
  t1006-cat-file 291/291, t5303-pack-corruption 25/25
- grit-lib pack/odb unit tests pass (the 2 failing lib tests are in ignore.rs
  gitignore-glob, unrelated and pre-existing — ignore.rs untouched).
- No new clippy warnings in pack.rs.
