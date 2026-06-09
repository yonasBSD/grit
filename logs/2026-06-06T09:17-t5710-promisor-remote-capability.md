# t5710-promisor-remote-capability — mop-up round 1

Ticket: 188720. Prior agent reached 20/22 per comments; fresh re-run scored 18/22
(flaky binary-swap variance), failing subtests 17, 20, 21, 22.

## Fixes

### Subtest 17 (clone with promisor.storeFields=partialCloneFilter)
Root cause: the `git fetch --filter=… ../server` step in the test fetches against an
already-up-to-date server (only the advertised filter changed). In
`grit/src/fetch_transport.rs::fetch_via_upload_pack_skipping`, the protocol-v2
`promisor-remote` capability — including `promisor.storeFields` persistence — was only
evaluated inside the `client_proto == 2` pack-building block, which is reached *after*
the `if wants.is_empty()` early return. With no wants, `store_promisor_fields` never ran,
so the client never updated `remote.lop.partialCloneFilter` (stayed blob:limit=8k instead
of blob:limit=7k).
Fix: evaluate `evaluate_promisor_remote_advertisement` in the `wants.is_empty()`
early-return branch too (the capability is part of the handshake, not the pack round).

### Subtest 20 (setup for subsequent fetches)
Root cause: `git -C server fetch origin` where `origin` is configured with only a `url`
(no `remote.origin.fetch` refspec). Git's `builtin/fetch.c get_ref_map` default case
(lines 572-578) fetches the remote's HEAD ref and marks it FETCH_HEAD_MERGE when the
remote has no fetch refspec and no matching `branch.*.merge`. Grit only mapped refs through
configured refspecs, so with none it wrote no branch line to FETCH_HEAD (only auto-followed
tags), and `git -C server update-ref HEAD FETCH_HEAD` left HEAD on the old commit ->
`rev-parse HEAD:bar` failed.
Fix: in `grit/src/commands/fetch.rs`, after the refspec-mapping loop, when
`refspecs.is_empty() && union_refspecs.is_empty() && !has_merge_cfg && !prefetch &&
!user_passed_cli_refspecs && !implicit_path_fetch`, emit the remote HEAD branch as the
single for-merge FETCH_HEAD entry.

### Subtests 21 & 22 (subsequent fetch/pull from a partial-clone server)
The client (a partial clone, `extensions.partialclone=origin`) pulls from a local-path
server that is itself a partial clone with promisor objects missing (on its lop). Grit's
local-path pull fast path copies the reachable object closure directly
(`copy_reachable_objects_*` in fetch.rs) and aborted with "missing object ... while
copying from remote" on the server's omitted promisor objects.

Fix (grit/src/commands/fetch.rs + pull.rs):
- New `copy_reachable_objects_skipping_missing_promisor`, used by the local pull copy when
  the destination treats promisor packs (`grit_lib::promisor::repo_treats_promisor_packs`).
- The reachable-copy walk now (only in this promisor mode):
  - Prunes at objects the destination already has (`exists_local`) — its "haves" — so a
    pull introducing only a new commit does not touch unrelated promisor objects the
    destination already had (this was the 22 over-fetch bug: both `foo` and `bar` blobs
    were lazy-fetched instead of just the new `bar`).
  - On a source read miss, branches on the *source's* `promisor.advertise`:
    - advertise=true  -> skip + record the oid in the destination promisor markers
      (client lazy-fetches it from its own lop later) — subtest 21.
    - advertise=false -> lazy-fetch the object onto the *source* from the source's own
      promisor remote (`try_lazy_fetch_promisor_objects_batch`), then copy the now-present
      full object, mirroring a non-advertising server's pack-objects that must serve every
      requested object — subtest 22.
- Refactored the child-pushing match into `push_object_children` (shared by the normal and
  lazy-fetch copy paths).

## Status
22/22 — t5710 fully passing (stable across multiple runs).

Regression checks (all green): t5572-pull-submodule 69/69, t5604-clone-reference 34/34,
t5510-fetch 215/215, t5520-pull 80/80, t5616-partial-clone 47/47.

Notes:
- 2 pre-existing `grit-lib ignore::gitignore_glob_tests` unit-test failures are unrelated
  (ignore.rs untouched by this work; another agent's domain).
- t0410-partial-clone subtest 14 fails with `error: unexpected line: 'filter blob:none'`
  from serve_v2.rs:328 (server-side v2 `filter` command-arg parsing) — NOT touched by this
  work and not on this change's code path; another agent's in-flight serve_v2 work.
- t5500-fetch-pack failures are from a concurrent agent actively editing t5500-fetch-pack.sh
  and httpd contention, not this change.
