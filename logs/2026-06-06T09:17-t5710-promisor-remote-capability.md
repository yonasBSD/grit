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

## Status
20/22 (subtests 17, 18, 20 now passing; 18 was already passing in some runs). Remaining:
21, 22 — subsequent fetch/pull from client when promisor.advertise true/false. Under
investigation.

Note: 2 pre-existing `grit-lib ignore::gitignore_glob_tests` unit-test failures are
unrelated (ignore.rs untouched by this work; in another agent's domain).
