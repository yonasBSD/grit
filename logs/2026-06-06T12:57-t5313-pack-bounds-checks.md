# t5313-pack-bounds-checks

Ticket: 5e3952 (t5313-pack-bounds-checks: subtests failing)

## Starting state
8/9 passing. Failing subtest 4 'matched bogus object count'.

## Diagnosis
The test munges the v2 `.idx` fanout table at byte offset `255 * 4 = 1020`. For a
v2 index the fanout starts at byte 8 (after magic+version), so byte 1020 lands on
fanout entry `(1020 - 8) / 4 = 253`, setting `fanout[253] = 0xff000000` (big-endian).
This makes the fanout non-monotonic: entry 253 is huge while entries 254/255 (the
true object count) are small (1).

Upstream git's `load_idx` (`git/packfile.c`) walks all 256 fanout entries and rejects
the index with `error("non-monotonic index %s")` if any entry is smaller than its
predecessor. grit only read `fanout[255]` as the object count and never validated the
full fanout table, so it happily parsed/enumerated the single object and
`git cat-file --batch-all-objects --batch-check` produced `<oid> missing` instead of
empty output, failing `test_must_be_empty actual`.

## Fix
grit-lib/src/pack.rs: added `check_fanout_monotonic(&fanout, idx_path)` helper and
call it in both `read_pack_index_v1` and `read_pack_index_v2` right after reading the
256-entry fanout, before computing `object_count`. It returns
`Error::CorruptObject("non-monotonic index ...")` for a decreasing fanout, matching
git's behavior. A corrupt index is then rejected, the object is not enumerated, and
grit falls back to the restored base copy exactly as the test expects.

## Result
9/9 passing. fully_passing = true.

Unit tests: only the 2 known pre-existing `ignore::gitignore_glob_tests` failures
(unrelated to this change).
