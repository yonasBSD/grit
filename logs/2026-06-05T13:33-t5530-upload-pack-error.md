# t5530-upload-pack-error — work log (2026-06-05)

Ticket 8afe05. Upstream `git/t/t5530-upload-pack-error.sh` — "errors in upload-pack".

## Starting state
Fresh run: 6/11 (an earlier ticket in the group had already advanced it from 4/11).
Failing: 5, 6, 8, 10, 11.

## Root causes and fixes (all in grit/src/commands/)

### Subtests 5 & 6 — bad `want` ("no object" / "not tip"), protocol v0
grit's v0 `upload-pack` never validated that each `want` was a tip of an advertised
ref; it forwarded any OID to `pack-objects`, which died with "bad tree object" /
"pack-objects died" instead of the expected `not our ref` + `ERR` packet.

Fix (`upload_pack.rs`): after parsing the wants, compute the "our ref" tip set
(`our_ref_oids` = HEAD + everything under `refs/`) and validate each want, honoring
`uploadpack.allow{Tip,Reachable,Any}SHA1InWant` (default deny). A want for a missing
object, or for a non-tip the policy forbids, now sends `ERR upload-pack: not our ref
<oid>` on stdout, an `error: git upload-pack: not our ref <oid>` on stderr, and exits
128 (`reject_not_our_ref`). Mirrors `upload-pack.c` `parse_want` / `check_non_tip`.

### Subtest 8 — EOF just after stateless client wants
A stateless client sent want/shallow/deepen + flush then closed stdin (no have/done).
grit generated a pack anyway. Upstream (`upload_pack` in upload-pack.c) tolerates EOF
here: it emits the shallow list and exits without a pack.

Fix: in stateless mode, emit the v0 shallow-list response (`emit_v0_shallow_list`,
modeled on serve_v2 + `rev_list::shallow_grafts_for_upload_pack_deepen`) before
negotiating; track `saw_negotiation` and, if the client closed without any have/done,
flush and return with no pack. The shallow-list's `unshallow` emission was also fixed:
a client-shallow commit that remains the depth boundary (parents still cut) must NOT be
unshallowed — only commits that became interior (all parents within the deepened depth,
computed by local `commits_within_depth`) get an `unshallow` line. Without this the test
got a spurious `unshallow <head>` instead of the expected bare `0000`.

### Subtest 10 — repeated non-commit `have`, protocol v0
Two `have <tree>` lines crashed grit: it parsed every `have` as a commit
(`merge_ancestors_into`) and died "does not name a commit". Also the ACK rule was wrong.

Fix: only run commit-ancestor logic when the `have` object is actually a commit; for
trees/blobs just record them. Track distinct server-known haves (`they_have` /
`have_obj_count`) mirroring `do_got_oid`'s `THEY_HAVE` flag, and (non-multi-ack) ACK
whenever the distinct count is exactly 1 — so a single repeated object is ACKed each
time. Also: a flush during stateless negotiation now exits without a pack (matching
`get_common_commits`' stateless `exit(0)`), writing NAK only when no server-known have
was received.

### Subtest 11 — repeated non-commit `have`, protocol v2
serve_v2's `acknowledgments` section always wrote `NAK` and never ACKed haves.

Fix (`serve_v2.rs`): ACK each `have` the server already has, de-duplicated in
first-seen order (so repeated objects are ACKed once); emit `NAK` only when there are
no acks. Matches `send_acks`.

## Result
11/11 passing (official `./scripts/run-tests.sh t5530-upload-pack-error.sh`).

## Regression checks
- t5537-fetch-shallow 16/16, t5552-skipping-fetch-negotiator 6/6,
  t5536-fetch-conflicts 7/7 — still fully pass (these exercise the shared
  upload-pack/serve_v2 negotiation + shallow paths).
- t5551-http-fetch-smart: subtests 36/37 (clone empty SHA-256 repo over HTTP) fail, but
  they ALSO fail with my two files reverted to HEAD — pre-existing regression from
  another agent's in-flight fetch.rs/pack_objects_upload.rs work, NOT this change.
- The 2 failing grit-lib unit tests (ignore::gitignore_glob_tests) are pre-existing on
  committed HEAD and unrelated to transport. The 3 clippy errors (hash_object.rs,
  rebase.rs) are in other agents' files. No new warnings in my two files.

## Notes for next agent
The shared `tests/grit` binary is swapped by concurrent builds; subtest 2 ("error in
pack-objects packing") flaked once mid-run when another agent's build overwrote the
binary, but is fine with a clean binary copy.
