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

18/22 passing (was 10/22). Additional fixes:
- `config_info_list` treats a remote as promisor if `.promisor=true` OR `.partialCloneFilter` set
  (matches `promisor_remote_config`), in config order — needed otherLop advertisement (15/16).
- Lazy fetch from a promisor remote now registers `remote.<lop>.partialCloneFilter=blob:none`
  (Git `partial_clone_register`); makes the server advertise `partialCloneFilter=blob:none` for lop.
- Clone no longer inherits the source's `blob:none` promisor filter when an explicit `--filter`
  was given (`args.filter` non-empty) — was downgrading a `blob:limit=5k` clone to `blob:none`,
  stripping small blobs (15/16/17/19).

Remaining (10, 11, 18, 22): a `blob:limit` `--filter` clone does not set up `extensions.partialclone
=origin`, so when the client cannot lazy-fetch from the accepted LOP (no URL: test 10; KnownUrl
reject: test 11) it cannot fall back to origin and checkout fails. Test 18 (`--filter=auto`) needs
the wire filter resolved to the combined advertised filter AND persisted; test 22 is the
advertise=false subsequent-fetch case. These share the missing `origin` promisor fallback.
