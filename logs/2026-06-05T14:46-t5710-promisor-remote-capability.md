# t5710-promisor-remote-capability

Ticket 188720. Implement the protocol v2 `promisor-remote` capability (large-object promisor
advertisement) and fix clone packfile storage so the test suite passes.

## Root causes found

1. **Test 2**: `git clone --bare --no-local` stored objects loose instead of as a packfile.
   `git clone` always sets `TRANS_OPT_KEEP` (builtin/clone.c), so the received pack is kept rather
   than unpacked (the `transfer.unpackLimit` heuristic is fetch-only). Fix: set
   `GRIT_FETCH_KEEP_PACK=1` at the top of clone's `run()`.

2. **Tests 4/7/8/...**: grit didn't implement the `promisor-remote` protocol v2 capability at all.
   Without it, the server's filtered `pack-objects` lazily fetched the missing large blob (to
   measure its size in `omit_prefiltered_blobs`), back-filling the server's ODB so the object was
   no longer "missing". Implemented the full capability:
   - New module `grit-lib/src/promisor_remote.rs` mirroring `git/promisor-remote.c`
     (`promisor_remote_info` server advertisement; `promisor_remote_reply` client accept logic
     with acceptFromServer None/KnownName/KnownUrl/All + checkFields; urlencode/decode).
   - Server (`serve_v2.rs`): advertise `promisor-remote=<info>`; parse the client's
     `promisor-remote=<accepted>` reply; when non-empty + filter active, pass
     `GRIT_OMIT_MISSING_PROMISOR=1` to spawned `pack-objects`.
   - `pack-objects` `omit_prefiltered_blobs`: when that env is set on a promisor repo, drop
     locally-absent objects (`!odb.exists`) without lazy-fetching instead of reading them.
   - Client (`fetch_transport.rs` `evaluate_promisor_remote_advertisement` +
     `file_upload_pack_v2.rs write_v2_fetch_request`): evaluate accept policy, send the reply,
     resolve `--filter=auto` to the combined advertised filter, and apply `promisor.storeFields`.
   - clone `run()`: apply `-c` config overrides to the dest repo BEFORE the fetch (git writes them
     early), guarded so the later apply does not duplicate multi-valued entries.

## Status

14/22 -> (see below). Remaining work: storeFields/sendFields/checkFields/filter=auto trace
expectations (15/16/17/18), KnownName missing-URL + KnownUrl server-side lazy fetch (10/11/19/22).
