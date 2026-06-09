# t6421-merge-partial-clone

Ticket: e29c55. Subsystem: merge-ort (thread C).

## Problem

`git merge` on a partial clone (`clone --sparse --filter=blob:none`) must prefetch
the missing blobs it needs (for rename detection and 3-way content merge) in a
small number of batches, matching upstream Git's merge-ort. The test validates,
via `GIT_TRACE2_PERF`, the exact `fetch_count:N` per batch, the number of
`fetch.negotiationAlgorithm` child processes (= number of batches), and that
exactly the expected blobs move from missing -> present.

## Root cause

grit's merge reads blobs via `repo.odb.read`, which does NOT lazy-fetch missing
promisor blobs. On a `--filter=blob:none` clone the rename-source / rename-target
/ content-merge blobs are genuinely absent locally (only in the promisor remote),
so similarity rename detection silently failed (`detect_renames` got `None` for
the blob content) and the merge produced a wrong modify/delete CONFLICT (e.g.
`dir/subdir/Makefile`) instead of applying the `dir/ -> folder/` directory rename.

The previous code (`maybe_simulate_partial_clone_fetch`) faked the trace counts
with hardcoded batch sizes but never actually hydrated any blob, so the merge
itself still failed.

## Fix (grit/src/commands/merge.rs)

Replaced the fake simulation with `maybe_prefetch_partial_clone_merge_blobs`,
which actually hydrates the needed missing blobs from the promisor remote (via
`promisor_hydrate::try_lazy_fetch_promisor_objects_batch`) before the merge, in
the same phased batches Git uses, emitting the `child_start` + `fetch_count` perf
lines per batch:

- Phase 1/2 (rename detection, ours then theirs): `relevant_rename_blob_oids`
  computes basename-matched rename pairs restricted to *relevant* sources (the
  merge-ort optimization): a renamed-and-modified source is fetched only when the
  other side content-modifies it in place, or it lives under a directory this side
  renamed and the other side added a new path into. Pairing uses longest-shared-
  suffix matching so `dir/subdir/Makefile` pairs with `folder/subdir/Makefile`,
  not `folder/subdir/tweaked/Makefile`.
- Phase 3 (content merge): `content_merge_blob_oids` returns base+ours+theirs for
  paths modified on both sides (resolving rename targets by best-suffix match).

Helpers added: `path_under_dir`, `renamed_dir_prefix`.

## Status: 2/3 passing

- PASS: "Objects downloaded for single relevant rename" (B-single: 2,1 -> 3 blobs)
- PASS: "Objects downloaded when a directory rename triggered" (B-dir: 6 -> 6 blobs)
- FAIL: "Objects downloaded with lots of renames and modifications" (B-many)

### B-many remaining work

Expected 22 blobs in batches 12,5,3,2. Current impl fetches 15 (6,9) and the
merge fails. Missing pieces:
1. The **general (non-basename) rename detection** batch (leap1_O, leap2_O,
   jump1_A, jump2_A, newfile.rs = 5): renames where basename differs
   (leap1 -> jump1) are not paired by the basename phase; Git then runs full
   similarity over remaining relevant sources x all unpaired destinations,
   fetching those blobs. Not yet implemented.
2. Content merge of `general/leap{1,2}` (leap1_B, leap2_B = 2): these are
   renamed-on-A (leap->jump) AND modified-on-B, a rename+both-modified case the
   current content_merge_blob_oids does not catch (it only matches by base path /
   basename, and basename leap1 vs jump1 differ).
3. My basename Side1 batch yields fewer than Git's 12 (missing numbers/seq/values
   pairs in the right phase) — needs the general phase to fill in.

Implementing Git's two-phase relevant-rename algorithm (basename phase, then
general phase over remaining relevant sources) is required to get B-many exact.
