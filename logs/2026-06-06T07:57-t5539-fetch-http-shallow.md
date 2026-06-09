# t5539-fetch-http-shallow — MOP-UP ROUND 1 (ticket 39a832)

Continuing prior agent (commit c4cf729 / fae10c64c, 7/8). Remaining: test 3
"no shallow lines after receiving ACK ready" (v0 protocol, multi_ack_detailed + no-done).

## Correction to prior note
Prior agent believed the test server was real `git upload-pack`. It is NOT:
`lib-httpd.sh` (the in-repo port) wires `git-upload-pack` so that ONLY
`--advertise-refs` runs REAL git; **negotiation + pack generation run grit's own
`upload-pack --stateless-rpc`**. So BOTH client and server here are grit. The real
git advertisement DOES include `multi_ack_detailed no-done` (verified manually).

## What test 3 asserts
`GIT_TRACE_PACKET=trace GIT_TEST_PROTOCOL_VERSION=0 git fetch --depth=2`, then:
- `grep "fetch-pack< ACK .* ready" trace`  (server must emit ACK ready, traced with
  the `fetch-pack` packet identity)
- `! grep "fetch-pack> done" trace`        (client must NOT send `done` — no-done path)

## Root causes
1. CLIENT (`grit/src/http_smart.rs::fetch_pack_v0_v1_stateless_http`): sent all
   wants+haves+done in ONE POST and never traced negotiation packets with the
   `fetch-pack` identity. No multi-round, no no-done handling.
2. SERVER (`grit/src/commands/upload_pack.rs`): did not parse the `no-done` feature
   and, on the stateless flush after emitting `ACK <oid> ready`, returned without a
   pack. Per `upload-pack.c get_common_commits`, with `no_done && sent_ready` the
   server sends `ACK <oid>` and proceeds straight to pack generation in the same RPC.

## Fix plan
- Server: parse `no-done`; in stateless flush, if `ACK ready` was sent and no-done
  negotiated, send `ACK <last_hex>` and fall through to pack generation.
- Client: request `no-done` (+ multi_ack_detailed); run multi-round stateless
  negotiation (find_common): replay want-state each POST, batch haves with
  INITIAL_FLUSH/next_flush, parse ACK common/ready, replay un-was-common haves,
  stop on ready; if got_ready && no-done skip `done`, read pack from the same response;
  trace each sent/received negotiation pkt with the `fetch-pack` identity.
