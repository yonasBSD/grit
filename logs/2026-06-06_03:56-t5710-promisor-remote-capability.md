# t5710-promisor-remote-capability

Ticket: 188720

## Work log

- Claimed the ticket and reproduced the harness at 21/22.
- Added strict unpack support for promisor packs by allowing missing references that are known
  promisor objects.
- Routed receive-pack promisor ingestion through in-process unpacking when strict validation would
  otherwise reject promised missing references.
- Added upload-pack/pack-objects plumbing to distinguish accepted promisor delegation from cases
  where the server must backfill from its own promisor remote.
- Investigated the remaining subsequent-fetch case with `promisor.advertise=false`. The remaining
  failure is subtest 22: after `client2` pulls the new `bar` blob, the server still reports both
  `foo` and `bar` missing; Git expects only the original `foo` to remain missing.

## Validation

- `cargo fmt`
- `cargo check -p grit-cli` passes with existing warnings in `commit_graph_file.rs`, `diff.rs`,
  and `pull.rs`.
- `cargo build --release -p grit-cli` passes with the same warnings.
- `./scripts/run-tests.sh t5710-promisor-remote-capability.sh --timeout 180`: 21/22.
