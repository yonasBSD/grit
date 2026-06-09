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
`write_v2_fetch_request` (grit/src/file_upload_pack_v2.rs) did single-round
`want <oid> + done`; never sent `want-ref <refname>`, never consumed
`wanted-refs`, never emitted `negotiation_v2/total_rounds=2`.

## Client-side fix implemented — now 22/26 (9, 10, 11, 13 fixed)

Threaded want-ref + a real multi-round have/ACK loop through the `file://`
(subprocess) v2 fetch path:

- `write_v2_fetch_request` gained `want_refs: &[String]` and `send_done: bool`.
  Emits `want-ref <name>` lines and can omit `done` for a non-final round. All 5
  call sites updated; the 4 secondary ones pass `&[]`/`true` (behavior unchanged).
- `v2_fetch_supports_ref_in_want(caps)` added (file_upload_pack_v2.rs).
- `cli_want_refs_and_oids` (fetch_transport.rs): classifies CLI refspec sources —
  named/wildcard sources that resolve to an advertised ref become `want-ref`,
  exact-OID sources stay `want <oid>` (mirrors `fetch-pack.c add_wants`).
- `read_v2_acknowledgments`: reads one v2 `acknowledgments` section, reporting
  `ready`/`seen_ack` (mirrors `process_ack`).
- `local_negotiation_haves` now expands ref tips into a committer-date-ordered
  commit walk (`date_ordered_have_walk`, max 1024), so round 1 offers the newest
  commits first. Without this only ref tips were offered and a single round always
  sufficed (test 9 wanted `total_rounds=2`).
- Main v2 path in `fetch_via_upload_pack_skipping`: when there are local haves and
  it is not a shallow request, do round 1 (wants/want-refs + first 16 haves, no
  `done`) -> read acknowledgments -> if `ready`/no-ack-section read pack now
  (total_rounds=1), else round 2 (remaining haves + `done`) and read pack
  (total_rounds=2). Emits `negotiation_v2.total_rounds`.

Regression sweep (all unchanged from baseline): t5510 215/215, t5601 112/115,
t5516 106/124, t5552 6, t5616 46. grit-lib lib tests: only the 2 pre-existing
`ignore::gitignore_glob_tests` failures (another agent's area; I touched no
gitignore code).

## smart-HTTP server validation + ERR propagation — now 23/26 (22 fixed)

- serve_v2.rs `cmd_fetch`: validate every `want <oid>` before serving (mirrors
  `upload-pack.c parse_want`/`check_non_tip`). Unknown/forbidden OID -> emit
  `ERR upload-pack: not our ref <oid>` and exit 128. Honors
  `uploadpack.allow{Tip,Reachable,Any}SHA1InWant`. Added helpers
  `serve_our_ref_oids`, `serve_is_reachable_from_our_refs`,
  `serve_reject_not_our_ref`. This is what makes test 22 (server changed the
  advertised main OID under the client) fail correctly.
- Client ERR propagation: http_smart.rs v2 response loops and fetch_transport.rs
  `read_v2_fetch_pack_response`/`read_v2_acknowledgments` now detect an `ERR `
  pkt-line and `bail!("fatal: remote error: <msg>")`, which main.rs prints as
  `fatal: remote error: ...` (test greps exactly that).

Regression re-check (unchanged): t5510 215, t5601 112, t5616 46.

## smart-HTTP want-ref — now 24/26 (26 fixed)

http_smart.rs `http_fetch_pack`: when the server advertises `ref-in-want`, send
`want-ref <name>` for named/wildcard-matched advertised refs (helper
`http_want_refs_and_plain_wants`), emit recognized fetch features as standalone
arg lines when no plain `want` carries them, and consume the `wanted-refs` section
(`apply_wanted_refs_section`) to override head/tag OIDs with the server's current
resolution. This fixes test 26 (`unknown ref refs/heads/rain` now surfaced).

Regression re-check (unchanged): t5551 29, t5601 112, t5616 46, t5701 25.

## FULLY PASSING — 26/26 (23, 25 fixed)

Root-caused 23/25 with packet tracing: the `wanted-refs` override WAS reaching the
returned `heads` (correct OID), but the fetch caller builds its ref-update map from
the *advertised* ref list (`fetch.rs` `refs_for_mapping` uses `remote_advertised`
when non-empty), which still held the stale/fake OID from the rewritten ls-refs
response. `fetch.rs` is another agent's in-flight file, so the fix lives entirely
on my side: `apply_wanted_refs_section` now also overrides the matching entry in
`all_advertised` (made mutable), so the OID the caller maps from is the server's
authoritative `want-ref` resolution.

Final regression sweep (all == baseline): t5510 215, t5551 29, t5601 112, t5616
46, t5701 25, t5552 6, t5516 108, t5615 9, t5604 34, t5705 17, t5500 47.
grit-lib lib: only the 2 pre-existing `ignore::gitignore_glob_tests` failures
(another agent's `ignore.rs`; untouched here). No new clippy warnings in my files.

Files changed (this ticket): grit-lib/src/unpack_objects.rs and
grit/src/commands/unpack_objects.rs + grit/src/commands/upload_pack.rs +
grit/src/receive_ingest.rs (build-unblocking conflict resolution);
grit/src/file_upload_pack_v2.rs, grit/src/fetch_transport.rs (file:// want-ref +
multi-round); grit/src/commands/serve_v2.rs (server want validation);
grit/src/http_smart.rs (HTTP want-ref + wanted-refs + ERR). `fetch.rs` NOT touched
(reverted an exploratory edit to respect the concurrent agent).
