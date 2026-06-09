# t5705-session-id-in-capabilities

Ticket: 4c6789 — tests/t5705-session-id-in-capabilities.sh
Subsystem: protocol-server (thread C)

## Starting state

Fresh run: 9/17 passing. Failing subtests:
- 3, 5, 7  — session IDs NOT advertised by default (push v0/v1/v2)
- 10, 13, 16 — session IDs advertised (push v0/v1/v2)
- 15 — session IDs advertised (fetch v2)
- 17 — client & server log negotiated version (v2 fetch)

## Root causes

### 1. All push subtests (3,5,7,10,13,16) — `--receive-pack` push was a stub

`grit push` has a fast in-process local-file path that applies ref updates directly
without ever spawning a receive-pack program. When `--receive-pack <cmd>` was given,
`push_to_url` ran the native path and then unconditionally `bail!("failed to push some
refs to '{url}'")` (grit/src/commands/push.rs, old line ~2458). So every push in the test
exited non-zero (failing the `&&` chain) AND no server-side trace2 events were written
(no receive-pack subprocess ran, so `client-sid` never appeared in the server's events).

Fix: when an explicit, non-default `--receive-pack` program is given for a local-file push,
delegate to the existing complete `grit send-pack` implementation, which actually spawns the
receive-pack program (so the server emits `client-sid`/`negotiated-version`) and emits the
matching client-side `server-sid`. Guarded to only the simple-refspec case (no
mirror/all/delete/tags/follow-tags/set-upstream/atomic/force-with-lease) so all the other
native-path features (denyCurrentBranch policy, hooks, output, lease checks) are untouched.
Default `--receive-pack` values ("git receive-pack", "grit receive-pack", etc.) still use
the native fast path. New helpers: `is_default_receive_pack_program`,
`delegate_local_push_to_send_pack`.

### 2. fetch v2 subtests (15, 17) — client sent `session-id=` in the wrong place

`write_v2_fetch_request` (grit/src/file_upload_pack_v2.rs) emitted the `session-id=<id>`
pkt-line AFTER the `0001` delimiter, i.e. as a per-command fetch argument. In protocol v2,
`session-id` is a capability (git serve.c lists it with a `.receive` handler, like
`object-format`), so it must be sent BEFORE the delimiter in the request's capability list.
The grit v2 server's `cmd_fetch` arg parser then rejected it with "unexpected line:
'session-id=...'", aborting the whole fetch (upload-pack exit 1) and never emitting
`client-sid`.

Fix: move the client's `session-id=` line to before `write_delim`, alongside agent/object-format.
When SID is not advertised (the default) `session_id_on_wire` is None, so normal clone/fetch
requests are byte-for-byte unchanged.

## Result

17/17 passing.

## Files changed
- grit/src/commands/push.rs — delegate explicit `--receive-pack` local push to send-pack
- grit/src/file_upload_pack_v2.rs — send v2 `session-id` as a capability (before delim)

## Regression checks
- t5701-git-serve: 25/25 (unchanged)
- t5702-protocol-v2: SKIP (needs git daemon, unrelated)
- Manual: v2 clone+fetch (no SID) work; normal local push + default --receive-pack push work
- 2 failing grit-lib lib tests are in ignore.rs (gitignore globs) — pre-existing, unrelated
  to this ticket; grit-lib/src/fetch_negotiator.rs is another agent's in-flight edit (left alone).
