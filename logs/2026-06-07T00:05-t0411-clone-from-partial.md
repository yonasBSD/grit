# t0411-clone-from-partial

## Status
- Start: 5/7 (subtests 4 and 6 failing)
- End: 7/7 fully passing

## Failing subtests (first run)
- `4 - fetch from file://... must not fetch from promisor remote and execute script`
- `6 - clone from promisor remote does not lazy-fetch by default`

## Root cause
The server-side upload-pack (v0/v1: `grit/src/commands/upload_pack.rs`; v2:
`grit/src/commands/serve_v2.rs`) computed a `force_lazy_fetch` flag that, for an
**unfiltered** clone/fetch against a partial-clone server that does NOT advertise a
promisor remote, called `hydrate_upload_pack_blobs_missing_from_client`. That helper
unconditionally set `GIT_NO_LAZY_FETCH=0` and lazy-fetched the missing blobs from the
promisor remote — i.e. it ran the `evil` repo's promisor `fake-upload-pack`.

Upstream `git upload-pack` pins `GIT_NO_LAZY_FETCH=1` by default (grit already does this
in `upload_pack::run`), so the server-side `pack-objects` must NOT lazily fetch missing
objects; it just fails to read them and dies. The `force_lazy_fetch`/hydrate machinery
(added for t5710) ignored that pinned env.

## Fix
Gate `force_lazy_fetch` (and therefore the hydrate call and the `GIT_NO_LAZY_FETCH=0`
override passed to the spawned `pack-objects`) on lazy-fetch actually being enabled in the
environment, via `promisor_hydrate::git_no_lazy_fetch_env_disables_lazy()`. With the
default pinned `GIT_NO_LAZY_FETCH=1` the server no longer hydrates and `pack-objects`
fails cleanly (subtest 6 then emits "lazy fetching disabled" from
`try_lazy_fetch_promisor_object`). When the operator re-enables lazy fetching
(`GIT_NO_LAZY_FETCH=0`, subtest 7) hydration/lazy-fetch still happens as before.

Files:
- grit/src/commands/serve_v2.rs
- grit/src/commands/upload_pack.rs

## Regression checks
- t0411-clone-from-partial: 7/7
- t5710-promisor-remote-capability: 22/22 (introduced this machinery)
- t0410-partial-clone: 38/38
- t5601-clone: 112/115 (unchanged from committed baseline — pre-existing failures, not mine)
- grit-lib unit tests: only the 2 known ignore::gitignore_glob_tests failures
