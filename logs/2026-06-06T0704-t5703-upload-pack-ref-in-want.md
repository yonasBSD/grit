# t5703-upload-pack-ref-in-want — mop-up round 1

Ticket: 991205. Prior agent reached 18/26 (server-side want-ref complete).

## Build was broken on entry

`cargo build --release -p grit-cli` failed: the GitButler workspace integration
had committed UNRESOLVED 3-way merge conflict markers into several source files
(two agents added overlapping features — promisor in-process unpack vs. shallow
`--shallow-file` boundaries). This blocked ALL builds for every agent.

Resolved the conflicts by taking the semantic UNION of both sides (the
already-merged function bodies referenced symbols from both branches):

- `grit-lib/src/unpack_objects.rs`: `strict_verify_packed_references_map` now
  takes `allowed_missing`, `allow_promisor_missing_references`, AND
  `shallow_boundaries`; call site passes all three.
- `grit/src/commands/unpack_objects.rs`: merged imports (`ObjectId`+`ObjectKind`,
  promisor helpers), kept both promisor-allowed-missing and shallow-boundaries
  computation; restored the lost `let quiet = args.quiet || !io::stderr().is_terminal();`.
- `grit/src/commands/upload_pack.rs`: merged imports (`parse_tag`/`parse_tree` +
  `ref_namespace`).
- `grit/src/receive_ingest.rs`: merged imports; `ingest_via_unpack_objects_subprocess`
  and `ingest_promisor_pack_in_process` both take `shallow_boundaries`; kept
  `write_temp_shallow_file` helper.

Build green after fix. `cargo test -p grit-lib --lib unpack_objects` = 12/12.
(Two unrelated failures remain in `ignore::gitignore_glob_tests` — another
agent's in-flight area, not touched here.)

t5703 re-run after fix: 18/26 (unchanged — matches prior agent's state, confirms
the conflict resolution preserved both features correctly).

## Remaining 8 failures (all client-side)

9, 10, 11, 13 (file://), 22, 23, 25, 26 (httpd, PERL_TEST_HELPERS).

Root cause (per prior agent, confirmed): grit's v2 fetch client
`write_v2_fetch_request` (grit/src/file_upload_pack_v2.rs) does single-round
`want <oid> + done`; never sends `want-ref <refname>`, never consumes
`wanted-refs`, never emits `negotiation_v2/total_rounds=2` (multi-round have/ACK
loop). Adding this is a broad rewrite of shared `fetch_via_upload_pack_skipping`
machinery in grit/src/fetch_transport.rs that many other passing t5xxx tests flow
through — high regression risk. Deferred by prior agent for that reason.
