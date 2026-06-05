# t5539-fetch-http-shallow

Ticket: 39a832 — tests/t5539-fetch-http-shallow.sh

## Starting state
4/8 passing. Failing: 3 (no shallow lines after ACK ready, v0), 5 (fetch shallow
since), 7 (fetch exclude tag one), 8 (fetching deepen).

## Root causes found & fixed

### 1. Negotiator walked past shallow boundary -> "object not found" (tests 5,7,8)
`SkippingNegotiator` (grit-lib/src/fetch_negotiator.rs) read parents of every
commit it visited. In a shallow client repo the boundary commit's parents are not
present, so `read_parents` -> `odb.read` failed with "object not found" before the
fetch request was even sent. Git's `register_shallow()` grafts those parents away so
the rev-list walk treats boundary commits as parentless.
Fix: load `$GIT_DIR/shallow` into the negotiator at construction; new `parents_of`
helper returns an empty parent list for boundary commits. All three `read_parents`
call sites (`mark_common`, `next_have`) now go through `parents_of`.

### 2. v0/v1 ref advertisement of a shallow server has `shallow <oid>` trailer (test 3 partial)
A shallow server's protocol-v0 `git-upload-pack` advertisement appends
`shallow <oid>` lines after the refs (upstream `advertise_shallow_grafts`). grit's
`parse_v0_v1_advertisement` (grit/src/http_smart.rs) fed `shallow` to the OID parser
=> "bad oid in v0/v1 advertisement: shallow". Fix: skip `shallow `/`unshallow `
trailer lines in the advertisement parser. (Fixes the parse crash; test 3 still
needs the multi-round ACK-ready negotiation — see below.)

### 3. deepen-since sent the raw date string instead of a Unix timestamp (test 5)
Git's `fetch-pack` runs `approxidate()` on `--shallow-since`/`--deepen-since` and
sends the resulting integer; `upload-pack` parses `deepen-since` with
`parse_timestamp` and rejects trailing text. grit sent the raw `"200000000 +0700"`,
which makes a real `git upload-pack` (the test server is system git-http-backend)
die with NO output, so the fetch silently transferred nothing.
Fix: convert via `grit_lib::git_date::approx::approxidate_careful` before writing the
`deepen-since` wire line. Applied at all four send sites:
- grit/src/http_smart.rs (v0/v1 + v2 extension builders) via `deepen_since_wire_value`
- grit/src/file_upload_pack_v2.rs
- grit/src/fetch_transport.rs

## Result
7/8 passing. Remaining: test 3 "no shallow lines after receiving ACK ready"
(v0/protocol-0). It exercises multi-round stateless-RPC negotiation where the server
returns `ACK <oid> ready` with `no-done` negotiated, the client must trace
`fetch-pack< ACK .* ready` and must NOT send `fetch-pack> done`. grit's v0 http
fetch path (`fetch_pack_v0_v1_stateless_http`) currently sends all haves + done in a
single shot and does not emit `fetch-pack<`/`fetch-pack>` labelled trace lines nor
honor the `no-done` + ACK-ready short-circuit. Substantial work; left for follow-up.

## Server confound note (for the next agent)
The test HTTP server is the SYSTEM `git-http-backend` (test-httpd's
`find_git_http_backend` falls through GIT_EXEC_PATH, which only holds git-p4, to
/opt/homebrew/.../git-http-backend). So the *server* in this test is real git; the
failures are all in grit's *client*. To repro manually you must export
`GUST_BIN=target/release/grit` when launching test-httpd, and the server is still
real git upload-pack.
