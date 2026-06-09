# t6112-rev-list-filters-objects — ticket c1bff7

Date: 2026-06-06T23:36 (UTC)
Result: 54/54 passing (was 52/54).

## Failing subtests addressed

### 30 — "verify skipping tree iteration when not collecting omits"
The second half of this subtest runs `rev-list --objects --filter=combine:tree:1+tree:3 HEAD`
with `GIT_TRACE=1` and expects exactly one `Skipping contents of tree dir1/...` trace line.

Root cause: grit's filtered object walk uses a `parent_union` optimization
(`union_parent_reachable_objects`) that precomputes every object reachable from a commit's
parents and short-circuits any tree/blob already in that set (rev_list.rs ~6135, ~6258, ~6289).
For `r3` the `dir1` tree is shared between HEAD and HEAD~1. With the union optimization, when
HEAD is walked first, `dir1` is found in HEAD~1's reachable set and skipped silently, so it is
only ever entered once (under HEAD~1) and never re-visited — so the per-filter `seen_at_depth`
re-visit detection (which is what produces the `Skipping contents of tree` trace for `tree:`
filters) never fires.

Upstream Git instead visits every reachable tree per commit and relies on each filter's own
seen state (`filter_trees_depth`'s `seen_at_depth` oidmap / the combine subfilters) to detect
re-visits and emit `LOFR_SKIP_TREE` (list-objects.c:203, list-objects-filter.c).

Fix: disable the `parent_union` short-circuit whenever a filter is active. Dedup of shown
objects is already handled by the `emitted` HashSet, and the final omitted set is deduped via a
BTreeSet (rev_list.rs ~1456), so the union optimization is purely a (filter-incompatible)
performance shortcut. With it disabled for filtered walks, grit visits trees per-commit exactly
like Git and the skip-tree trace matches. (rev_list.rs: 2 call sites updated —
`collect_objects_segmented` and `collect_root_object`'s commit branch.)

### 53 — "expand blob limit in protocol"
`clone --filter=blob:limit=1k` over `file://` must send the canonical/expanded spec
`blob:limit=1024` on the wire (Git's `expand_list_objects_filter_spec`,
list-objects-filter-options.c:333). grit sent the raw `blob:limit=1k`.

A helper `expand_object_filter_for_protocol` already existed in rev_list.rs but was never
called. Wired it into all four client-side wire `filter <spec>` emission sites:
- grit/src/file_upload_pack_v2.rs:459 (the local `file://` protocol-v2 path — the one this test exercises)
- grit/src/fetch_transport.rs:2271
- grit/src/http_smart.rs:532 and :642

Config storage of the partial-clone filter is unchanged (still the original spec); only the
wire line is canonicalized, matching Git.

## Regression checks
- t6112: 54/54
- t6113-rev-list-bitmap-filters: 14/14
- t5616-partial-clone: 47/47
- t0410-partial-clone: 37/38 (unchanged from recorded baseline 37/38 — pre-existing failure, not mine)
- grit-lib --lib: only the 2 known ignore::gitignore_glob_tests failures (unrelated)
