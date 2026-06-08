# Phase 8 — Transport libify survey

Date: 2026-06-08
Scope: conservative pass over `grit/src/file_upload_pack_v2.rs` and
`grit/src/http_smart.rs`, moving only *purely self-contained* protocol-v2
parsing/building logic into `grit-lib`. No in-process serve() overhaul.

## What was moved this pass

New module `grit_lib::protocol_v2` (`grit-lib/src/protocol_v2.rs`), holding pure
v2 capability parsing/echo helpers — no I/O, no `std::env`, no `crate::` refs,
no trace, no transport backend:

- `server_advertises_bundle_uri(caps)` — bare or valued `bundle-uri` cap.
- `cap_lines_for_command_request(caps)` — forward `agent=` / `object-format=`
  for a follow-up `command=…` request (was triplicated as
  `cap_lines_for_bundle_request` in `file_upload_pack_v2.rs`,
  `cap_lines_for_bundle_request` in `http_bundle_uri.rs`, and
  `cap_lines_for_client_request` in `http_smart.rs` — three byte-identical
  copies, now one).
- `fetch_features(caps)` — set of `fetch=` feature tokens (was
  `v2_fetch_features` in `http_smart.rs`).
- `fetch_supports_feature` + the three thin wrappers `fetch_supports_sideband_all`
  / `fetch_supports_ref_in_want` / `fetch_supports_filter` (were three near-
  identical `v2_fetch_supports_*` fns in `file_upload_pack_v2.rs`, plus one
  inline copy in `clone_preflight_file_v2_if_needed`, now all one helper).

CLI call sites delegate via `pub(crate) use … as <old_name>` re-exports, so no
caller churn. 6 unit tests added in the new module.

### Verification (byte-exact, in_scope=yes only)

Baselines recorded before the change, identical after:

| test | result |
|---|---|
| t5701-git-serve | 25/25 (unchanged) |
| t5601-clone | 112/115 (unchanged; 3 pre-existing fails) |
| t5703-upload-pack-ref-in-want | 26/26 (unchanged) |
| t5704-protocol-violations | 3/3 (unchanged) |
| t5705-session-id-in-capabilities | 17/17 (unchanged) |
| t5555-http-smart-common | 10/10 (unchanged) |
| t5551-http-fetch-smart | 31/37 (unchanged; pre-existing fails) |
| t5558-clone-bundle-uri | 37/37 (unchanged) |
| t5730-protocol-v2-bundle-uri-file | 8/8 (unchanged) |
| t5732-protocol-v2-bundle-uri-http | 9/9 (unchanged) |
| t5616-partial-clone | 47/47 (unchanged) |

t5700 (protocol-v1), t5702 (protocol-v2), t5500 (fetch-pack) are
`in_scope = "skip"` and were not run.

## What transport logic is ALREADY in grit-lib

Most pure protocol logic is already libified:

- `grit_lib::pkt_line` — pkt-line read/write framing.
- `grit_lib::protocol` — `protocol.<name>.allow` / `GIT_ALLOW_PROTOCOL` policy,
  client/server protocol-version selection, `GIT_PROTOCOL` merge.
- `grit_lib::protocol_v2` — (new this pass) v2 capability/feature parsing.
- `grit_lib::fetch_negotiator` — negotiation algorithms.
- `grit_lib::ls_remote` — ls-refs v2 line parsing / output formatting.
- `grit_lib::connectivity` — connectivity / want-have checks.
- `grit_lib::receive_pack`, `grit_lib::push_cert`, `grit_lib::push_report` —
  receive-pack command/report parsing and push-cert handling.
- `grit_lib::transport_path` — URL/path classification.
- `grit_lib::shallow`, `grit_lib::fetch_head`, `grit_lib::rev_list`
  (`expand_object_filter_for_protocol`) — supporting pure logic.

## What stays in the CLI by design

The remaining transport code in `file_upload_pack_v2.rs` / `http_smart.rs` /
`http_bundle_uri.rs` / `fetch_transport.rs` / `send_pack.rs` /
`ext_transport.rs` is *not* pure and stays in `grit-cli`:

- **I/O backends.** HTTP via `crate::http_client`, git:// via
  `crate::git_daemon_url`, ssh/`ext::` spawns, socket/child-stdio plumbing.
- **Subprocess orchestration.** `spawn_upload_pack_readonly` shells out to
  `grit upload-pack` (`grit_executable()` / `strip_trace2_env`), drives stdin/
  stdout, waits on exit status. This is the Phase-3 spawn (see below).
- **Trace.** `wire_trace` / `crate::trace_packet` / `crate::trace2_transfer`
  packet-line tracing is woven through every request builder
  (`write_ls_refs_*`, `write_v2_fetch_request`, the `read_*`/`drain_*`/`skip_*`
  readers). These functions interleave `trace_packet_git(...)` with each
  `pkt_line::write_line`, so they are not separable without either threading a
  trace sink or losing byte-exact `GIT_TRACE_PACKET` output. Left in place.
- **Process-global config/env reads.** `client_wants_protocol_v2`,
  `transfer_bundle_uri_enabled`, `should_use_source_head_symref_fallback`,
  `std::env::var("GIT_DEFAULT_HASH")` — read ambient state, so kept at the CLI
  edge (the underlying decisions already live in `grit_lib::protocol`).

## Explicit remaining work — KILL_SPAWNS Phase 3 (NOT done here)

Per `KILL_SPAWNS.md` "Phase 3 — in-process protocol endpoints for local
transport (medium-high)": for `file://`/local fetch and push, grit still spawns
its own `upload-pack` / `receive-pack` purely to get a pkt-line conversation
over pipes (`fetch_transport.rs`, `file_upload_pack_v2.rs`, `send_pack.rs`,
`ext_transport.rs` fallback, `commands/http_backend.rs`). KILL_SPAWNS prescribes:

```rust
grit_protocol::upload_pack::serve(repo, &mut impl Read, &mut impl Write, Opts)
grit_protocol::receive_pack::serve(repo, &mut impl Read, &mut impl Write, Opts)
```

with local transport connecting client and server over an in-memory duplex, a
`GRIT_INPROC_PROTOCOL` strangler toggle, and the same `serve()` functions reused
for ssh/http wired to sockets/child stdio (also collapsing the nested
re-invocation at `receive_pack.rs:981`). SSH and `ext::` keep spawning (far side
is genuinely another machine/program).

That is a large strangler effort that must NOT be rushed — it would risk the
t5xxx transport suite — so it was deliberately left untouched this pass.
