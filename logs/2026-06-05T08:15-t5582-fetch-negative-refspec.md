# t5582-fetch-negative-refspec — negative refspecs in fetch/push/prefetch

Ticket: aa7f52. Start: 10/16. End: 16/16.

## Failing subtests fixed
6, 7 (fetch negative pattern refspec), 11 (fetch --prune negative refspec),
12, 13 (push matching `:`/`+:` + negative refspec), 14 (--prefetch modifies refspecs).

## Root causes and fixes (all in grit/src/commands/)

### Tests 6 & 7 — FETCH_HEAD missing negative-refspec filtering (fetch.rs)
The CLI glob refspec loop emitted a FETCH_HEAD line only for refs whose local
tracking ref actually changed: it `continue`d on an up-to-date ref *before* the
FETCH_HEAD push. When every matched ref was already current (e.g. only `alternate`
needed updating and it was unchanged from a prior fetch), `fetch_head_entries`
stayed empty and the generic fallback (around the old `if fetch_head_entries.is_empty()`)
listed *all* `refs/heads/`, ignoring `^refs/heads/m*` — so `main` leaked into FETCH_HEAD.
Fix: push the FETCH_HEAD line for every non-excluded matched ref *before* the
up-to-date early `continue`; only the ref write + progress line are skipped for no-ops.

### Test 11 — fetch --prune deleted negatively-excluded tracking refs (fetch.rs)
`prune_stale_refs` deleted any ref under the prune prefix not in `updated_refs`,
with no negative-refspec awareness. Added `local_ref_protected_by_negative`, which
mirrors Git's `refspec_find_negative_match` (refspec.c): reverse-map the local dst
through positive refspecs to candidate remote srcs, and if any candidate matches a
negative refspec, the ref is protected from pruning. Threaded a `prune_protection_refspecs`
list into `prune_stale_refs` and the will-prune precheck.

### Tests 12 & 13 — matching push `:`/`+:` + negative wrongly errored (push.rs)
`collect_matching_push_updates` skipped negatively-excluded branches *before*
counting them as matched, so excluding the only shared branch (`^main`) returned
`matched == 0` and the caller bailed "No refs in common". Git treats an all-excluded
matching push as a successful no-op. Fix: count a branch present on both sides as
matched, then apply the negative exclusion afterward.

### Test 14 — --prefetch produced refs/prefetch/heads/... (fetch.rs)
`collect_refspecs` eagerly normalizes an unqualified dst `bogus/*` to
`refs/heads/bogus/*`. Applying the prefetch rewrite after that yielded
`refs/prefetch/heads/bogus/*` instead of `refs/prefetch/bogus/*`. Git's
`filter_prefetch_refspec` runs on the raw dst (strips only a leading `refs/`).
Added `collect_refspecs_for_prefetch`, which builds refspecs from the raw config
value with the prefetch rewrite applied, and used it for both the config `refspecs`
path and the coalesced `union_refspecs` build.

## Regression check (baseline failing -> after)
t5510-fetch 11 -> 11 (no change); t5505-remote 62 -> 61 (improved 1);
t5516-fetch-push 118 -> 118 (no change). No regressions.

## Notes
- `cargo test -p grit-lib --lib`: 245 pass, 2 pre-existing fails in
  `ignore::gitignore_glob_tests` (ignore.rs unchanged vs HEAD; unrelated to this work).
- No new clippy warnings in my files; added one `#[allow(clippy::too_many_arguments)]`
  on `prune_stale_refs` (gained one arg).
