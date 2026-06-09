# t5554-noop-fetch-negotiator

Ticket: 035619f1-cb3f-4522-a6a2-da9cae5cc5e5

## Problem

`t5554` configures `fetch.negotiationAlgorithm=noop` on the client then fetches and
asserts that NO `fetch> have <oid>` pkt-lines are emitted (only `fetch> done`).

Initial run: 0/1. grit emitted `fetch> have <oid>` because the fetch transport only
special-cased the `skipping` algorithm (forcing protocol v0); the `noop` value fell
through to the default `SkippingNegotiator`, which advertises local ref tips as haves.

## Upstream reference

`git/negotiator/noop.c`: the noop negotiator's `add_tip`, `known_common`, and `next`
are all no-ops, so the client offers no `have` lines and the server always streams a
full pack. Selected via `fetch-negotiator.c` `fetch_negotiator_init` on
`FETCH_NEGOTIATION_NOOP`.

## Fix

`grit/src/fetch_transport.rs`:

- Added `fetch_negotiation_is_noop(local_git_dir)` helper (reads
  `fetch.negotiationalgorithm`, case-insensitive compare to `noop`).
- v0/v1 path (`fetch_upload_pack_negotiate_pack_bytes_with_streams`): OR the new check
  into `suppress_haves`, so the whole have/ACK exchange is skipped and the client goes
  straight to `done`.
- v2 path: `local_negotiation_haves` returns an empty Vec for noop, so no haves are
  offered in the v2 request either.

## Result

- t5554-noop-fetch-negotiator: 1/1 (was 0/1).
- t5552-skipping-fetch-negotiator: 6/6 (no regression; transient 5/6 was a binary
  swap by a concurrent agent, resolved by re-copying the binary).
- t5510-fetch: 215/215 (no regression).
- cargo test -p grit-lib --lib: only the 2 known pre-existing ignore::gitignore_glob
  failures.
