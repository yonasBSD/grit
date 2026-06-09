# t5703-upload-pack-ref-in-want

Ticket 991205. Upstream test: protocol v2 `fetch` ref-in-want (server + client).

## Starting state
7/26 passing (fresh re-run).

## Root causes found & fixed (server side: `grit serve-v2`)

1. `cmd_fetch` ignored `want-ref` lines entirely (no-op match arm). Implemented
   full `want-ref` handling in `grit/src/commands/serve_v2.rs`:
   - Reject when `uploadpack.allowRefInWant` is off (`unexpected line`).
   - Resolve the ref relative to `GIT_NAMESPACE` via the *storage* refname
     (`ref_namespace::storage_ref_name`), resolved with NO DWIM fallback —
     `refs::resolve_ref(git_dir, &storage)`. Passing the storage name avoids the
     double-try fallback in `resolve_ref_depth` that would otherwise resolve a
     top-level `refs/heads/ns-no` when only the namespaced lookup should match
     (test 16).
   - Reject hidden refs via `hide_refs::ref_is_hidden(logical, storage, patterns)`
     using `hide_ref_patterns_uploadpack` (tests 18/20).
   - On failure: write `ERR unknown ref <ref>` pkt-line to stdout, print
     `fatal: unknown ref <ref>` to stderr, `exit(128)` (tests 3/16/18 grep
     "unknown ref" on either stream).
   - Collect resolved `(refname, oid)` and emit the `wanted-refs` section
     (`wanted-refs` line, `<oid> <ref>` lines, delim) just before
     `shallow-info`/`packfile`, matching `upload-pack.c send_wanted_ref_info`
     order. Also push resolved oids into `wants`.

2. `pack-objects --stdout` produced a **0-byte** output when the object set was
   empty after exclusion, but upstream `write_pack_file()` is always called and
   streams a valid 32-byte empty pack. Fixed in
   `grit/src/commands/pack_objects.rs`: in the empty-enumeration early return,
   when `args.stdout` (and not `--non-empty`) write `build_pack(&[], ...)` to
   stdout. This is what made test 7 ("want-ref with ref we already have commit
   for") fail with `index-pack: error: pack too small`.

## Result
18/26 passing. Newly passing: 3,4,5,6,7,15,16,17,18,19,20.

## Remaining failures (all one shared root cause — client side)
9, 10, 11, 13, 22, 23, 25, 26.

grit's protocol-v2 **client** fetch (`write_v2_fetch_request` in
`grit/src/file_upload_pack_v2.rs`, driven from `fetch_transport.rs`
`fetch_via_upload_pack_skipping`) uses "skipping negotiation": it sends
`want <oid>` + `done` in a single round with no `have` lines and never sends
`want-ref`. Consequences:
- 9, 11: assert `grep '"key":"total_rounds","value":"2"' trace2` — grit emits no
  `total_rounds` trace2 data event and only does 1 round. Needs real multi-round
  v2 negotiation (`negotiation_v2`/`total_rounds` trace2, category data event)
  with `have` lines so a 33-commit local repo takes 2 rounds.
- 10, 13: fetch works and writes correct refs, but assert
  `grep "want-ref refs/heads/main" log` — client must send `want-ref <refname>`
  (not `want <oid>`) for refspecs naming a ref when the server advertised
  `ref-in-want`, then consume the `wanted-refs` section to map ref->oid.
- 22, 23, 25, 26: PERL_TEST_HELPERS + httpd "server changes ref during
  negotiation" cases. Depend on the same negotiation/want-ref machinery and on
  the client re-resolving via want-ref; grit trusts the advertised OID and does a
  single-round fetch so it gets the stale/fake OID or wrongly succeeds.

These four-plus failures need a client-side v2 negotiation rewrite (multi-round
have/ACK loop + want-ref + wanted-refs consumption + total_rounds trace2). That
is broad shared fetch machinery; deferred to avoid regressing other t5xxx fetch
tests. Server-side ref-in-want is complete.
