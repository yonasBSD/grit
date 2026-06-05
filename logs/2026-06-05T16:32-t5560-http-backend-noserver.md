# t5560-http-backend-noserver — work log

Ticket: b6fcc7
Date: 2026-06-05T16:32Z

## Starting state
- `./scripts/run-tests.sh t5560-http-backend-noserver.sh` → 2/14 (only setup + the
  `expect_aliased` test passed). Subtests 2–13 returned `500 Internal Server Error`
  for nearly everything.

## Root cause
`grit/src/commands/http_backend.rs` was written narrowly for
`t5562-http-backend-content-length.sh` (the smart-HTTP POST content-length paths).
It had no routing table, no dumb-protocol static file serving, no export policy,
and no `http.getanyfile` / `http.uploadpack` / `http.receivepack` access policy.
Every unrecognized request fell into "unable to determine requested smart HTTP
service" → 500. t5560 only checks the CGI `Status:` line, which is determined by
those policies.

## Fix
Rewrote `http_backend.rs` to mirror upstream `git/http-backend.c`:
- `getdir()` from `PATH_TRANSLATED` (or `GIT_PROJECT_ROOT`+`PATH_INFO` with an
  `daemon_avoid_alias` check).
- A routing table (`ROUTES`) matching the request path tail: `/HEAD`,
  `/info/refs`, `/objects/info/{alternates,http-alternates,packs}`, loose objects,
  pack/idx files, and the `POST /git-{upload,receive}-pack` RPCs. The repo dir is
  the path prefix before the matched tail. No match → `404`.
- Method mismatch → `bad_request` (405 for HTTP/1.1, else 400).
- Not a git repo → `404`. Export check (`GIT_HTTP_EXPORT_ALL` or
  `git-daemon-export-ok`) → `404`.
- `HttpPolicy` from repo-local config: `http.getanyfile` (default true),
  `http.uploadpack` (default on), `http.receivepack` (default by `REMOTE_USER`).
- Static handlers gated by `select_getanyfile` → `403`; smart RPC and smart
  `info/refs` gated by `service_enabled` → `403`.
- Smart `info/refs?service=...` runs `<svc> --http-backend-info-refs`, prepending
  the `# service=git-<svc>\n` pkt-line banner + flush (v0).

## Preserving t5562 (was 16/16, must not regress)
t5562's `verify_http_result` treats a `fatal:` on stderr as failure, and expects
empty/truncated request bodies to produce `fatal:` while valid pushes create the
branch. Kept request-body validation (`validate_{upload,receive}_pack_request`)
but relaxed it so a flush-only `0000` body is accepted (needed for t5560 subtests
8–13). The RPC runs the service non-statelessly (as the previous code did, so
receive-pack still creates the branch), and a service failure surfaces as a
`fatal:` 500.

## Result
- t5560: **14/14** passing.
- t5562: **16/16** (no regression).
- `cargo build --release -p grit-cli` clean (only the pre-existing diff.rs
  `ext_total` warning).
- 2 failing grit-lib unit tests (`ignore::gitignore_glob_tests::*`) are unrelated
  (gitignore globbing, `ignore.rs` untouched by me, pre-existing).

Note: the build was briefly blocked by another agent's in-flight broken `fetch.rs`
(`branch_label` not in scope); waited for it to be fixed, then built cleanly.
