# t6421-merge-partial-clone — FINAL (3/3 passing)

Ticket: e29c55. Subsystem: merge-ort partial-clone blob prefetch.

## Starting state

Prior agent left 2/3 passing (B-single, B-dir) at 68c1e0246. The remaining
failure was B-many ("Objects downloaded with lots of renames and
modifications"): expected fetch batches `12, 5, 3, 2` (= 22 blobs in 4
`fetch.negotiationAlgorithm` children); grit produced `6, 9` and the merge
failed with spurious modify/delete CONFLICTs on `general/leap1` and
`general/leap2`.

## Root cause of the remaining failure

`maybe_prefetch_partial_clone_merge_blobs` in `grit/src/commands/merge.rs`
only modeled one rename-detection batch per side plus a content-merge batch,
and its relevance/rename logic was too narrow:

1. **No general (non-basename) rename phase.** Git's `diffcore_rename_extended`
   runs a basename-matching phase (`basename_prefetch`) then a general
   full-matrix phase (`inexact_prefetch`), each its own lazy-fetch child. grit
   collapsed them, so it never fetched the leap1/leap2 -> jump1/jump2 general
   renames (leap1_O, leap2_O, jump1_A, jump2_A, newfile.rs = batch 2 of 5).

2. **Wrong relevance model.** grit only treated a deleted source as relevant
   when the other side modified it *in place*. merge-ort's `add_pair`
   (`content_relevant = (match_mask & filemask) == 0`) makes a deleted source
   content-relevant whenever the other side does NOT keep it byte-identical at
   the original path — including when the other side also deleted/renamed it
   (e.g. `basename/{numbers,sequence,values}` renamed on both sides). Those were
   being missed from the basename phase.

3. **Modified-in-place paths mis-classified as rename sources.** The `deleted`
   (rename-source) list included base paths still present on the side (modified
   in place), so on the theirs(B) side `general/leap1` (modified in place on B)
   was wrongly treated as a rename source, contaminating the Side2 basename
   batch.

4. **Content merge missed cross-renamed both-modified files.** `general/leap1`
   is renamed-and-modified on A (leap->jump, basename differs) and modified in
   place on B; `content_merge_blob_oids` only resolved rename targets by
   basename, so it never recognized leap as both-sides-modified and didn't fetch
   leap1_B/leap2_B (batch 4 of 2).

## Fix (grit/src/commands/merge.rs only)

- `relevant_rename_blob_oids` now returns `RenameDetectionBlobs { basename,
  general }` — two phases per side, each pushed as its own batch (so a side can
  emit up to two `fetch.negotiationAlgorithm` children, matching git).
- Relevance is split into `Content` vs `Location`:
  - Content-relevant = the other side does not keep the file unchanged at the
    original path. Participates in basename phase AND, if unpaired, the general
    phase.
  - Location-relevant = under a directory this side renamed and the other side
    added into. Participates in basename phase ONLY; unpaired location-relevant
    sources are culled (directory rename already known) and are NOT read in the
    general phase. This is why `dir/subdir/tweaked/f` (deleted, location-only) is
    correctly never fetched.
- The general phase fires only when content-relevant sources remain unpaired;
  it then fetches every still-unpaired destination + every remaining
  content-relevant source (mirrors `inexact_prefetch`).
- Rename-source list now requires the base path to be ABSENT on the side
  (modified-in-place paths are handled by content merge, not rename detection).
- `content_merge_blob_oids`: when a base path is renamed away on a side with a
  non-basename (general) rename, fall back to any added non-exact path in the
  same parent directory as the rename target. That target blob was already
  fetched in the general phase, so it adds no extra fetch, but it lets the
  both-sides-modified detection fire and pull the in-place-modified side's blob
  (leap1_B, leap2_B).

## Result

B-many now fetches exactly `12, 5, 3, 2` (4 children, 22 blobs, no new missing)
and merges cleanly. `./scripts/run-tests.sh t6421-merge-partial-clone.sh` => 3/3.

## Regression checks (no regressions)

- t6402-merge-rename: 46/46
- t6423-merge-rename-directories: 80/80
- t6422-merge-rename-corner-cases: 16 pass / 4 fail (unchanged baseline)
- grit-lib unit tests: pass except the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures (unrelated).
