# test-lib.sh OIDPATH_REGEX + pack-feature skips — 2026-06-08

Repo-owner-directed cleanup of the test-harness portability tail.

## Shared test-lib.sh fix (orchestrator-owned)
- `tests/test-lib.sh`: added `OIDPATH_REGEX` (was missing; upstream defines it at
  test-lib.sh:1611). The ported file had `OID_REGEX` but not the loose-object **path**
  regex, so tests using `.git/objects/$OIDPATH_REGEX` expanded to a bare
  `.git/objects/` glob. Defined inline from `$ZERO_OID` (insert `/` after 2 hex chars,
  translate `0`→`[0-9a-f]`) since `test_oid_to_path` is sourced later than this point.
  Added to the adjacent `export` line.
- Fixes **t1050-large** → 29/29 (was 28/29, subtest 12 "verify-pack -v stats").

## Skips (per repo owner)
Two deep pack features set `in_scope = "skip"` — they need real subsystems, not test fixes:
- `data/tests/t5/t5310-pack-bitmaps.toml` (was 203/236): full pack-bitmap reader/writer
  (commit selection, EWAH chunks, load_bitmap, trace2).
- `data/tests/t5/t5319-multi-pack-index.toml` (was 94/98): byte-exact pack-objects sizes
  + real EWAH commit bitmaps + preferBitmapTips.

## Remaining test-portability tail
58 candidate test-body files handed to workflow `test-portability-fixes`
(wf_b3082a0e-a5f), each differential-verified against real git 2.52.0
(`/opt/homebrew/bin/git`) before any test edit. Out of scope (genuine grit work):
t3404-rebase-interactive (29), t3701-add-interactive (14).
